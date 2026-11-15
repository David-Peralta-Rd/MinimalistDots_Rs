//! Subcomando `launcher`: lista aplicaciones `.desktop` y lanza la elegida.
//!
//! Reemplaza a `wofi --show drun` (referencia: `launcher.qml`).

use md_core::{DesktopEntry, IconIndex, MenuEntry, MenuMode};
use md_ui::{MenuConfig, MenuEvent, SlintHost, run_menu};

/// Ejecuta el lanzador de aplicaciones.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
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

    let config = MenuConfig {
        app_id: "md.launcher".to_owned(),
        title: "Lanzador".to_owned(),
        prompt: "🚀".to_owned(),
        accent: md_core::Color::new(0x7e, 0xc7, 0xa2), // acento verde
        row_height: 36.0,
        width: 500.0,
        height: 460.0,
        can_delete: false,
        can_wipe: false,
        submit_empty: false,
    };

    if let MenuEvent::Selected(index) = run_menu(&SlintHost, &entries, MenuMode::Launcher, &config)?
        && let Some(app) = apps.get(index)
    {
        launch(app)?;
    }
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
