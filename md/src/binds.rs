//! Subcomando `binds`: visor de atajos de teclado agrupados por categoría.
//!
//! Lee `~/.cache/hypr/keybinds.json` (generado por `export_keybinds.lua`) y lo
//! muestra en un menú de solo lectura.

use std::path::PathBuf;

use md_core::{parse_keybinds, to_menu_entries};

use crate::i18n::Texts;
use crate::menu::run_entries;

/// Ejecuta el visor de atajos.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let texts = Texts::load();
    let path = keybinds_path().ok_or_else(|| {
        std::io::Error::other("no se pudo determinar $HOME para localizar keybinds.json")
    })?;

    let json = std::fs::read_to_string(&path).map_err(|_| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("no se encontró el archivo de atajos ({})", path.display()),
        )
    })?;

    let keybinds = parse_keybinds(&json)?;
    let entries = to_menu_entries(&keybinds);

    let _ = run_entries(&entries, "⌨️", false, false, false, None, None, None, None)?;
    let _ = texts;
    Ok(())
}

/// Ruta del archivo de atajos que genera `export_keybinds.lua`.
fn keybinds_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache/hypr/keybinds.json"))
}
