//! Novedades mostradas en la bienvenida (`updates.toml`).

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Update {
    pub titulo: String,
    #[serde(default)]
    pub descripcion: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct UpdatesFile {
    #[serde(default)]
    novedad: Vec<Update>,
}

/// Lee las novedades de `path`. Si el archivo no existe o es inválido, devuelve
/// una lista vacía.
pub fn load_updates(path: &std::path::Path) -> Vec<Update> {
    let Ok(text) = std::fs::read_to_string(path) else {
        return Vec::new();
    };
    toml::from_str::<UpdatesFile>(&text)
        .map(|file| file.novedad)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn parses_updates() {
        let path = std::env::temp_dir().join("md_updates_test.toml");
        let Ok(mut file) = std::fs::File::create(&path) else {
            return;
        };
        let _ = writeln!(file, "[[novedad]]");
        let _ = writeln!(file, "titulo = \"Hola\"");
        let _ = writeln!(file, "descripcion = \"mundo\"");
        let updates = load_updates(&path);
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].titulo, "Hola");
        assert_eq!(updates[0].descripcion, "mundo");
    }

    #[test]
    fn missing_file_returns_empty() {
        let path = std::env::temp_dir().join("md_updates_missing.toml");
        let _ = std::fs::remove_file(&path);
        assert!(load_updates(&path).is_empty());
    }
}
