//! Entradas `.desktop` de freedesktop: parseo, descubrimiento e iconos.
//!
//! Solo lógica pura y lectura de archivos; el lanzamiento real de procesos lo
//! hace el binario `md`.

use std::collections::{BTreeSet, HashMap};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Una aplicación de escritorio lista para listar y lanzar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopEntry {
    pub name: String,
    pub icon: String,
    pub generic_name: String,
    pub exec: String,
    pub terminal: bool,
}

impl DesktopEntry {
    /// Interpreta el contenido de un archivo `.desktop`.
    ///
    /// Devuelve `None` cuando no es una aplicación lanzable (no es de tipo
    /// `Application`, está oculta con `NoDisplay`, o le faltan `Name`/`Exec`).
    pub fn parse(text: &str) -> Option<Self> {
        let mut section = String::new();
        let mut entry_type = String::new();
        let mut name = String::new();
        let mut generic_name = String::new();
        let mut icon = String::new();
        let mut exec = String::new();
        let mut terminal = false;
        let mut no_display = false;

        for raw_line in text.lines() {
            let line = raw_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(rest) = line.strip_prefix('[') {
                if let Some(end) = rest.find(']') {
                    section = rest[..end].trim().to_owned();
                }
                continue;
            }
            if section != "Desktop Entry" {
                continue;
            }
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };
            let value = unescape_desktop(value.trim());
            match key.trim() {
                "Type" => entry_type = value,
                "Name" => name = value,
                "GenericName" => generic_name = value,
                "Icon" => icon = value,
                "Exec" => exec = value,
                "Terminal" => terminal = value == "true",
                "NoDisplay" => no_display = value == "true",
                _ => {}
            }
        }

        let is_application = entry_type.is_empty() || entry_type == "Application";
        if !is_application || no_display || name.is_empty() || exec.is_empty() {
            return None;
        }
        Some(Self {
            name,
            icon,
            generic_name,
            exec,
            terminal,
        })
    }
}

/// Deshace los escapes del formato `.desktop` (`\s`, `\n`, `\t`, `\r`, `\\`).
fn unescape_desktop(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('s') => out.push(' '),
            Some('n') => out.push('\n'),
            Some('t') => out.push('\t'),
            Some('r') => out.push('\r'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// Separa una línea `Exec` en `(programa, argumentos...)`, eliminando los
/// códigos de campo (`%f`, `%U`, ...) y respetando las comillas dobles.
pub fn exec_args(exec: &str) -> Vec<String> {
    let mut args = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = exec.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '%' {
            match chars.peek().copied() {
                Some('%') => {
                    current.push('%');
                    chars.next();
                }
                Some(next) if "fFuUdDnNickvm".contains(next) => {
                    // Código de campo: en un lanzador sin archivos se elimina.
                    chars.next();
                }
                _ => current.push('%'),
            }
            continue;
        }
        if c == '"' {
            in_quotes = !in_quotes;
            continue;
        }
        if c.is_whitespace() && !in_quotes {
            if !current.is_empty() {
                args.push(std::mem::take(&mut current));
            }
            continue;
        }
        current.push(c);
    }
    if !current.is_empty() {
        args.push(current);
    }
    args
}

/// Directorios de datos de XDG en orden de prioridad (primero el del usuario).
pub fn data_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let home = env::var_os("HOME").map(PathBuf::from);

    let data_home = env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| home.as_ref().map(|home| home.join(".local/share")));
    if let Some(dir) = data_home {
        dirs.push(dir);
    }

    let data_dirs =
        env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".to_owned());
    for dir in data_dirs.split(':').filter(|part| !part.is_empty()) {
        dirs.push(PathBuf::from(dir));
    }

    dirs
}

/// Descubre las aplicaciones `.desktop` instaladas (deduplicadas por nombre de
/// archivo, ganando el directorio de mayor prioridad).
pub fn discover_desktop_entries() -> Vec<DesktopEntry> {
    let mut seen = BTreeSet::new();
    let mut entries = Vec::new();

    for data_dir in data_dirs() {
        let apps_dir = data_dir.join("applications");
        let Ok(read_dir) = fs::read_dir(&apps_dir) else {
            continue;
        };
        let mut paths: Vec<PathBuf> = read_dir
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("desktop"))
            .collect();
        paths.sort();

        for path in paths {
            let file_id = path
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            if file_id.is_empty() || !seen.insert(file_id) {
                continue;
            }
            if let Ok(text) = fs::read_to_string(&path)
                && let Some(entry) = DesktopEntry::parse(&text)
            {
                entries.push(entry);
            }
        }
    }

    entries
}

/// Índice de iconos del sistema: mapea nombres de icono a rutas de archivo.
///
/// Se construye recorriendo los directorios de iconos una sola vez, para que
/// resolver el icono de cada aplicación sea `O(1)`.
#[derive(Debug, Default)]
pub struct IconIndex {
    by_name: HashMap<String, PathBuf>,
}

impl IconIndex {
    /// Construye el índice desde las raíces indicadas (testeable).
    pub fn build(roots: &[PathBuf]) -> Self {
        let mut by_name = HashMap::new();
        for root in roots {
            walk_icons(root, &mut by_name);
        }
        Self { by_name }
    }

    /// Índice con las raíces por defecto (XDG + `/usr/share/pixmaps`).
    pub fn discover() -> Self {
        let mut roots = Vec::new();
        for data_dir in data_dirs() {
            roots.push(data_dir.join("icons"));
        }
        if let Some(home) = env::var_os("HOME").map(PathBuf::from) {
            roots.push(home.join(".icons"));
        }
        roots.push(PathBuf::from("/usr/local/share/pixmaps"));
        roots.push(PathBuf::from("/usr/share/pixmaps"));
        Self::build(&roots)
    }

    /// Resuelve un icono: una ruta absoluta se usa tal cual (si existe); un
    /// nombre se busca en el índice.
    pub fn resolve(&self, icon: &str) -> Option<PathBuf> {
        if icon.is_empty() {
            return None;
        }
        if icon.starts_with('/') {
            let path = PathBuf::from(icon);
            return path.is_file().then_some(path);
        }
        self.by_name.get(icon).cloned()
    }
}

/// Preferencia de un icono: `(rango de formato, área en píxeles)`.
/// Menor rango es mejor (`svg` < `png` < `xpm`); a igual formato, mayor área.
fn icon_preference(path: &Path) -> (u8, u64) {
    let format = match path.extension().and_then(|ext| ext.to_str()) {
        Some("svg") => 0,
        Some("png") => 1,
        Some("xpm") => 2,
        _ => 3,
    };
    (format, size_area(path))
}

/// ¿`new` es mejor icono que `old`?
fn is_better(new: &Path, old: &Path) -> bool {
    let (new_format, new_area) = icon_preference(new);
    let (old_format, old_area) = icon_preference(old);
    new_format < old_format || (new_format == old_format && new_area > old_area)
}

/// Mayor área `WxH` encontrada en los componentes de la ruta (0 si no hay).
fn size_area(path: &Path) -> u64 {
    let mut best = 0;
    for component in path.components() {
        let text = component.as_os_str().to_string_lossy();
        if let Some((width, height)) = text.split_once('x')
            && let (Ok(width), Ok(height)) = (width.parse::<u64>(), height.parse::<u64>())
        {
            best = best.max(width.saturating_mul(height));
        }
    }
    best
}

/// Recorre `root` (recursivamente) indexando `png`/`svg`/`xpm` por su nombre.
fn walk_icons(root: &Path, by_name: &mut HashMap<String, PathBuf>) {
    let Ok(read_dir) = fs::read_dir(root) else {
        return;
    };
    let mut entries: Vec<_> = read_dir.flatten().collect();
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();
        if file_type.is_dir() {
            walk_icons(&path, by_name);
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
            continue;
        };
        if !matches!(extension, "png" | "svg" | "xpm") {
            continue;
        }
        let Some(name) = path.file_stem().and_then(|stem| stem.to_str()) else {
            continue;
        };
        let better = match by_name.get(name) {
            None => true,
            Some(existing) => is_better(&path, existing),
        };
        if better {
            by_name.insert(name.to_owned(), path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::TempDir;

    const APP: &str = "\
[Desktop Entry]
Type=Application
Name=Firefox
GenericName=Navegador web
Icon=firefox
Exec=firefox %u
Terminal=false
";

    #[test]
    fn parses_a_basic_entry() -> Result<(), &'static str> {
        let entry = DesktopEntry::parse(APP).ok_or("debe parsear")?;
        assert_eq!(entry.name, "Firefox");
        assert_eq!(entry.generic_name, "Navegador web");
        assert_eq!(entry.icon, "firefox");
        assert_eq!(entry.exec, "firefox %u");
        assert!(!entry.terminal);
        Ok(())
    }

    #[test]
    fn skips_non_applications_and_hidden() {
        let link = "[Desktop Entry]\nType=Link\nName=X\nURL=http://x\n";
        assert_eq!(DesktopEntry::parse(link), None);

        let hidden = "[Desktop Entry]\nType=Application\nName=X\nExec=x\nNoDisplay=true\n";
        assert_eq!(DesktopEntry::parse(hidden), None);

        let missing_exec = "[Desktop Entry]\nType=Application\nName=X\n";
        assert_eq!(DesktopEntry::parse(missing_exec), None);
    }

    #[test]
    fn unescapes_values() -> Result<(), &'static str> {
        let text = "[Desktop Entry]\nType=Application\nName=Mi\\sApp\nExec=mi\\sapp\\n--flag\n";
        let entry = DesktopEntry::parse(text).ok_or("debe parsear")?;
        assert_eq!(entry.name, "Mi App");
        assert_eq!(entry.exec, "mi app\n--flag");
        Ok(())
    }

    #[test]
    fn splits_exec_removing_field_codes_and_quotes() {
        assert_eq!(exec_args("firefox %u"), vec!["firefox"]);
        assert_eq!(exec_args("code --new-window"), vec!["code", "--new-window"]);
        assert_eq!(
            exec_args("sh -c \"echo hola mundo\""),
            vec!["sh", "-c", "echo hola mundo"]
        );
        assert_eq!(exec_args("100%%"), vec!["100%"]);
    }

    #[test]
    fn builds_icon_index_and_resolves_names() -> Result<(), Box<dyn std::error::Error>> {
        let dir = TempDir::new("icons")?;
        let png_dir = dir.path().join("hicolor/48x48/apps");
        let svg_dir = dir.path().join("hicolor/scalable/apps");
        fs::create_dir_all(&png_dir)?;
        fs::create_dir_all(&svg_dir)?;
        fs::write(png_dir.join("firefox.png"), b"png")?;
        fs::write(svg_dir.join("firefox.svg"), b"svg")?;

        let index = IconIndex::build(&[dir.path().to_path_buf()]);
        let resolved = index.resolve("firefox").ok_or("debe resolver")?;
        // `svg` (escalable) se prefiere sobre `png` (48x48).
        assert_eq!(resolved.extension().and_then(|e| e.to_str()), Some("svg"));
        assert_eq!(index.resolve("inexistente"), None);
        assert_eq!(index.resolve(""), None);
        Ok(())
    }
}
