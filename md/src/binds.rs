//! Subcomando `binds`: visor de atajos de teclado agrupados por categoría.
//!
//! Lee `~/.cache/hypr/keybinds.json` (generado por `export_keybinds.lua`) y lo
//! muestra en un menú de solo lectura.

use std::path::PathBuf;

use md_core::{MenuMode, parse_keybinds, to_menu_entries};
use md_ui::{MenuConfig, SlintHost, run_menu};

use crate::i18n::Texts;

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

    let config = MenuConfig {
        app_id: "md.binds".to_owned(),
        title: texts.t("escritorio.atajos"),
        prompt: "⌨️".to_owned(),
        accent: md_core::Color::new(0x7e, 0xc7, 0xa2),
        row_height: 36.0,
        width: 650.0,
        height: 480.0,
        can_delete: false,
        can_wipe: false,
        submit_empty: false,
        wipe_label: None,
        confirm_wipe_label: None,
    };

    // Visor de solo lectura: la selección se ignora (cierra el menú).
    let _ = run_menu(&SlintHost, &entries, MenuMode::Dmenu, &config)?;
    Ok(())
}

/// Ruta del archivo de atajos que genera `export_keybinds.lua`.
fn keybinds_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache/hypr/keybinds.json"))
}
