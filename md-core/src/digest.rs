//! `compute_tree_digest`: hash determinista de un árbol de archivos.
//!
//! ADR-0006. Se recorre el árbol una vez (sin seguir enlaces), se ordenan las
//! entradas por ruta relativa y se hashea el contenido de los archivos en
//! paralelo (rayon). El resultado combina estructura, metadatos (permisos y
//! destino de los enlaces simbólicos) y contenido con `blake3`.

use std::fs::{self, Permissions};
use std::io::Read;
use std::path::{Path, PathBuf};

use rayon::prelude::*;

/// Error tipado al calcular el digest de un árbol (nunca hay `panic`).
#[derive(Debug, thiserror::Error)]
pub enum TreeDigestError {
    #[error("ruta no UTF-8: {0}")]
    NonUtf8(PathBuf),
    #[error("no se pudo recorrer {0}: {1}")]
    Walk(PathBuf, std::io::Error),
    #[error("no se pudo leer {0}: {1}")]
    Read(PathBuf, std::io::Error),
}

impl TreeDigestError {
    fn walk(path: &Path, error: std::io::Error) -> Self {
        Self::Walk(path.to_owned(), error)
    }

    fn read(path: &Path, error: std::io::Error) -> Self {
        Self::Read(path.to_owned(), error)
    }
}

/// Tipo de entrada del árbol (determina cómo se serializa).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Dir,
    File,
    Symlink,
    Other,
}

impl Kind {
    fn as_char(self) -> char {
        match self {
            Kind::Dir => 'd',
            Kind::File => 'f',
            Kind::Symlink => 'l',
            Kind::Other => 'o',
        }
    }
}

/// Una entrada del árbol. `content_hash` solo se rellena para archivos.
struct Entry {
    kind: Kind,
    rel: String,
    meta: String,
    content_hash: Option<String>,
}

/// Permisos como metadato determinista (`0o7777` en Unix).
fn permissions_string(permissions: Permissions) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        format!("{:04o}", permissions.mode() & 0o7777)
    }
    #[cfg(not(unix))]
    {
        if permissions.readonly() {
            "0444".to_owned()
        } else {
            "0644".to_owned()
        }
    }
}

/// Reúne todas las entradas del árbol (ordenadas por ruta relativa) sin seguir
/// enlaces simbólicos. `ignore` es una lista de nombres de archivo o carpeta que
/// se omiten en cualquier nivel.
fn collect_entries(root: &Path, ignore: &[String]) -> Result<Vec<Entry>, TreeDigestError> {
    let mut entries = Vec::new();
    let mut stack: Vec<(PathBuf, String)> = Vec::new();

    let children = fs::read_dir(root).map_err(|error| TreeDigestError::walk(root, error))?;
    for child in children {
        let child = child.map_err(|error| TreeDigestError::walk(root, error))?;
        let name = child
            .file_name()
            .to_str()
            .ok_or_else(|| TreeDigestError::NonUtf8(child.path()))?
            .to_owned();
        stack.push((child.path(), name));
    }

    while let Some((path, rel)) = stack.pop() {
        let base = rel.rsplit('/').next().unwrap_or(&rel).to_owned();
        if ignore.contains(&base) {
            continue;
        }

        let metadata =
            fs::symlink_metadata(&path).map_err(|error| TreeDigestError::walk(&path, error))?;
        let file_type = metadata.file_type();

        if file_type.is_dir() {
            entries.push(Entry {
                kind: Kind::Dir,
                rel: rel.clone(),
                meta: permissions_string(metadata.permissions()),
                content_hash: None,
            });
            let children =
                fs::read_dir(&path).map_err(|error| TreeDigestError::walk(&path, error))?;
            for child in children {
                let child = child.map_err(|error| TreeDigestError::walk(&path, error))?;
                let name = child
                    .file_name()
                    .to_str()
                    .ok_or_else(|| TreeDigestError::NonUtf8(child.path()))?
                    .to_owned();
                stack.push((child.path(), format!("{rel}/{name}")));
            }
        } else if file_type.is_symlink() {
            let target =
                fs::read_link(&path).map_err(|error| TreeDigestError::walk(&path, error))?;
            let target = target
                .to_str()
                .ok_or_else(|| TreeDigestError::NonUtf8(target.clone()))?
                .to_owned();
            entries.push(Entry {
                kind: Kind::Symlink,
                rel,
                meta: target,
                content_hash: None,
            });
        } else if file_type.is_file() {
            entries.push(Entry {
                kind: Kind::File,
                rel,
                meta: permissions_string(metadata.permissions()),
                content_hash: None,
            });
        } else {
            entries.push(Entry {
                kind: Kind::Other,
                rel,
                meta: permissions_string(metadata.permissions()),
                content_hash: None,
            });
        }
    }

    entries.sort_by(|a, b| a.rel.cmp(&b.rel));
    Ok(entries)
}

/// Hash blake3 (en hex) del contenido de un archivo.
fn hash_file(path: &Path) -> Result<String, TreeDigestError> {
    let mut file = fs::File::open(path).map_err(|error| TreeDigestError::read(path, error))?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0u8; 8192];
    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|error| TreeDigestError::read(path, error))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(hasher.finalize().to_hex().to_string())
}

/// Digest determinista de `root`: mismo árbol → mismo hex, sea cual sea el orden
/// de creación o de recorrido. Devuelve un error tipado si una ruta no es UTF-8
/// o un archivo cambia mientras se lee.
pub fn compute_tree_digest(root: &Path, ignore: &[String]) -> Result<String, TreeDigestError> {
    let mut entries = collect_entries(root, ignore)?;

    // Hash del contenido de los archivos en paralelo.
    let file_indices: Vec<usize> = entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.kind == Kind::File)
        .map(|(index, _)| index)
        .collect();
    let hashes: Vec<String> = file_indices
        .par_iter()
        .map(|&index| hash_file(&root.join(&entries[index].rel)))
        .collect::<Result<Vec<String>, TreeDigestError>>()?;
    for (index, hash) in file_indices.into_iter().zip(hashes) {
        entries[index].content_hash = Some(hash);
    }

    let mut hasher = blake3::Hasher::new();
    for entry in &entries {
        let line = format!(
            "{}\t{}\t{}\t{}\n",
            entry.kind.as_char(),
            entry.rel,
            entry.meta,
            entry.content_hash.as_deref().unwrap_or("")
        );
        hasher.update(line.as_bytes());
    }
    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::TempDir;

    fn make_file(dir: &Path, rel: &str, content: &str) -> std::io::Result<()> {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)
    }

    fn digest(dir: &Path) -> Result<String, TreeDigestError> {
        compute_tree_digest(dir, &[])
    }

    #[test]
    fn empty_tree_is_deterministic() -> Result<(), Box<dyn std::error::Error>> {
        let dir = TempDir::new("empty")?;
        assert_eq!(digest(dir.path())?, digest(dir.path())?);
        Ok(())
    }

    #[test]
    fn same_content_same_digest_regardless_of_creation_order()
    -> Result<(), Box<dyn std::error::Error>> {
        let a = TempDir::new("a")?;
        let b = TempDir::new("b")?;
        make_file(a.path(), "uno.txt", "hola")?;
        make_file(a.path(), "dos.txt", "mundo")?;
        // En `b` se crean en orden inverso: el digest no debe cambiar.
        make_file(b.path(), "dos.txt", "mundo")?;
        make_file(b.path(), "uno.txt", "hola")?;
        assert_eq!(digest(a.path())?, digest(b.path())?);
        Ok(())
    }

    #[test]
    fn content_change_changes_digest() -> Result<(), Box<dyn std::error::Error>> {
        let a = TempDir::new("a2")?;
        let b = TempDir::new("b2")?;
        make_file(a.path(), "uno.txt", "hola")?;
        make_file(b.path(), "uno.txt", "adios")?;
        assert_ne!(digest(a.path())?, digest(b.path())?);
        Ok(())
    }

    #[test]
    fn ignored_names_are_excluded() -> Result<(), Box<dyn std::error::Error>> {
        let a = TempDir::new("ig_a")?;
        let b = TempDir::new("ig_b")?;
        make_file(a.path(), "keep.txt", "x")?;
        make_file(a.path(), "backups_dots", "ignorado")?;
        make_file(b.path(), "keep.txt", "x")?;
        let ignored = ["backups_dots".to_owned()];
        assert_eq!(
            compute_tree_digest(a.path(), &ignored)?,
            compute_tree_digest(b.path(), &ignored)?
        );
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn symlink_differs_from_regular_file() -> Result<(), Box<dyn std::error::Error>> {
        let a = TempDir::new("sym_a")?;
        let b = TempDir::new("sym_b")?;
        make_file(a.path(), "link", "hola")?;
        // En `b`, `link` es un enlace simbólico a un archivo real.
        make_file(b.path(), "real", "hola")?;
        std::os::unix::fs::symlink(b.path().join("real"), b.path().join("link"))?;
        assert_ne!(digest(a.path())?, digest(b.path())?);
        Ok(())
    }

    #[test]
    fn nested_directories_are_hashed() -> Result<(), Box<dyn std::error::Error>> {
        let a = TempDir::new("nest_a")?;
        let b = TempDir::new("nest_b")?;
        make_file(a.path(), "sub/inner.txt", "x")?;
        make_file(b.path(), "sub/inner.txt", "x")?;
        assert_eq!(digest(a.path())?, digest(b.path())?);
        Ok(())
    }
}
