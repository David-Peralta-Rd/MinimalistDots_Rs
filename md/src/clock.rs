//! Subcomando `clock`: reloj de escritorio persistente.
//!
//! Reemplaza a `clock.qml` (Quickshell) y al servicio `desktop_clock.lua`, que
//! ya arranca `md clock` al iniciar Hyprland. El widget vive en `md_ui::MdClock`;
//! aquí solo se aporta la hora/fecha actual con `date`, que respeta la
//! localización del sistema (`LC_TIME`/`LANG`) para los nombres de día y mes.

use std::process::Command;

use md_ui::{ClockConfig, ClockText, SlintHost, run_clock};

/// Ejecuta el reloj (bloquea hasta que se cierra la ventana).
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let config = ClockConfig {
        app_id: "md.clock".to_owned(),
    };
    run_clock(&SlintHost, &config, current_clock_text)?;
    Ok(())
}

/// Hora y fecha actuales. `date` devuelve `HH:MM` y `%a, %d %b` (p. ej.
/// `"dom, 15 nov"`); la fecha se pasa a minúsculas como hacía el QML original.
fn current_clock_text() -> ClockText {
    let output = Command::new("date")
        .args(["+%H:%M%n%a, %d %b"])
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .unwrap_or_default();

    let mut lines = output.splitn(2, '\n');
    let time = lines.next().unwrap_or("").trim().to_owned();
    let date = lines.next().unwrap_or("").trim().to_lowercase();
    ClockText { time, date }
}
