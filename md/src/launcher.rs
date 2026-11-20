//! Subcomando `launcher`: lista aplicaciones `.desktop` y lanza la elegida.
//!
//! Reemplaza a `wofi --show drun` (referencia: `launcher.qml`).

use md_core::{DesktopEntry, IconIndex, MenuEntry};
use std::fs;

use crate::i18n::Texts;
use crate::quickshell;

/// Ejecuta el lanzador de aplicaciones.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let texts = Texts::load();
    let apps = md_core::discover_desktop_entries();
    let icons = IconIndex::discover();

    // Las entradas del menú y `apps` comparten el mismo orden.
    let entries: Vec<MenuEntry> = apps
        .iter()
        .map(|app| MenuEntry {
            label: app.name.clone(),
            icon: icons
                .resolve(&app.icon)
                .map(|path| path.to_string_lossy().into_owned())
                .unwrap_or_default(),
            extra: app.generic_name.clone(),
        })
        .collect();

    let _ = texts;

    // Serializa las entradas para QuickShell.
    let json_entries: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| {
            serde_json::json!({
                "label": e.label,
                "icon": e.icon,
                "extra": e.extra,
            })
        })
        .collect();
    let entries_path = std::env::temp_dir().join("md-launcher-entries.json");
    let result_path = std::env::temp_dir().join("md-launcher-result.json");
    let _ = fs::remove_file(&result_path);
    fs::write(&entries_path, serde_json::to_string(&json_entries).unwrap_or_default())?;

    let _ = quickshell::spawn_wait("Launcher.qml", &[
        ("MD_LAUNCHER_ENTRIES", entries_path.to_str().unwrap_or("")),
        ("MD_LAUNCHER_RESULT", result_path.to_str().unwrap_or("")),
        ("MD_LAUNCHER_PROMPT", "🚀"),
        ("QML_XHR_ALLOW_FILE_READ", "1"),
    ]);

    // Lee la selección.
    if let Ok(raw) = fs::read_to_string(&result_path) {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(index) = value.get("selected").and_then(|v| v.as_i64()) {
                if index >= 0 {
                    let index = index as usize;
                    if let Some(app) = apps.get(index) {
                        launch(app)?;
                    }
                }
            }
        }
    }

    let _ = fs::remove_file(&entries_path);
    let _ = fs::remove_file(&result_path);
    Ok(())
}

/// Lanza la aplicación (con `footclient` si `Terminal=true`).
fn launch(app: &DesktopEntry) -> Result<(), std::io::Error> {
    let args = md_core::exec_args(&app.exec);
    if args.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "línea Exec vacía",
        ));
    }
    let mut command = if app.terminal {
        let mut command = std::process::Command::new("footclient");
        command.args(&args);
        command
    } else {
        let mut command = std::process::Command::new(&args[0]);
        command.args(&args[1..]);
        command
    };
    // `spawn` deja el proceso en segundo plano; al soltar el `Child` sigue vivo.
    command.spawn()?;
    Ok(())
}
