//! Fondos de pantalla: descubrimiento de imágenes en un directorio.

use std::path::{Path, PathBuf};

/// Una imagen de fondo disponible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wallpaper {
    pub name: String,
    pub path: PathBuf,
}

/// Extensiones de imagen reconocidas (sin el punto).
const EXTENSIONS: &[&str] = &["jpg", "jpeg", "png"];

/// Descubre imágenes en `dir` (sin recursión), ordenadas por nombre.
///
/// Un directorio inexistente o ilegible devuelve una lista vacía.
pub fn discover_wallpapers(dir: &Path) -> Vec<Wallpaper> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };

    let mut wallpapers: Vec<Wallpaper> = entries
        .filter_map(|entry| entry.ok())
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_file() {
                return None;
            }
            let extension = path.extension()?.to_str()?.to_lowercase();
            if !EXTENSIONS.contains(&extension.as_str()) {
                return None;
            }
            let name = path.file_name()?.to_str()?.to_owned();
            Some(Wallpaper { name, path })
        })
        .collect();

    wallpapers.sort_by_key(|wallpaper| wallpaper.name.to_lowercase());
    wallpapers
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::TempDir;

    #[test]
    fn discovers_images_sorted_and_skips_others() -> Result<(), Box<dyn std::error::Error>> {
        let dir = TempDir::new("wallpapers")?;
        let root = dir.path();
        std::fs::write(root.join("b.png"), b"png")?;
        std::fs::write(root.join("a.JPG"), b"jpg")?;
        std::fs::write(root.join("c.txt"), b"txt")?;
        std::fs::write(root.join("d.jpeg"), b"jpeg")?;
        std::fs::create_dir_all(root.join("sub"))?;
        std::fs::write(root.join("sub/e.png"), b"png")?;

        let wallpapers = discover_wallpapers(root);
        let names: Vec<&str> = wallpapers.iter().map(|w| w.name.as_str()).collect();
        // Solo imágenes del nivel superior, ordenadas sin distinguir mayúsculas.
        assert_eq!(names, vec!["a.JPG", "b.png", "d.jpeg"]);
        assert_eq!(wallpapers[0].path, root.join("a.JPG"));
        Ok(())
    }

    #[test]
    fn empty_for_missing_dir() {
        assert!(discover_wallpapers(Path::new("/no/existe")).is_empty());
    }
}
