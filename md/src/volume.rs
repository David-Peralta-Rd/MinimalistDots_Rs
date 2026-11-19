//! Subcomando `volume`: cambia el volumen con `wpctl` y muestra un OSD.
//!
//! Reemplaza a `volume.sh` + `osd-volume.qml` (el antiguo `md_volume_osd`).

use std::process::Command;

use clap::ValueEnum;

use md_core::VolumeState;
use md_ui::{OsdConfig, SlintHost, run_osd};

/// Acción de volumen pedida por el usuario.
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum Action {
    /// Sube el volumen un 5%.
    Up,
    /// Baja el volumen un 5%.
    Down,
    /// Alterna el silencio.
    Mute,
}

/// Ejecuta la acción y muestra el OSD con el estado resultante.
pub fn run(action: Action) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        Action::Up => wpctl(&["set-volume", "-l", "1.0", "@DEFAULT_AUDIO_SINK@", "5%+"])?,
        Action::Down => wpctl(&["set-volume", "@DEFAULT_AUDIO_SINK@", "5%-"])?,
        Action::Mute => wpctl(&["set-mute", "@DEFAULT_AUDIO_SINK@", "toggle"])?,
    }

    // Sin `wpctl` no podemos leer el estado: el cambio ya se hizo, se omite el OSD.
    let Some(state) = current_state() else {
        return Ok(());
    };

    // OSD minimalista: solo el porcentaje (o "Mute"), sin icono ni barra.
    // Se cierran ventanas OSD previas para evitar solapamiento al pulsar rápido.
    let _ = Command::new("hyprctl")
        .args(["dispatch", "closewindow", "class:md.osd"])
        .status();

    let config = OsdConfig {
        glyph: String::new(),
        label: state.label(),
        level: -1,
        muted: state.muted,
        accent: md_core::Color::new(0x7e, 0xc7, 0xa2),
        width: 80.0,
        height: 40.0,
        ..OsdConfig::default()
    };
    run_osd(&SlintHost, &config)?;
    Ok(())
}

/// Ejecuta `wpctl` con los argumentos dados.
fn wpctl(args: &[&str]) -> Result<(), std::io::Error> {
    let status = Command::new("wpctl").args(args).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "wpctl terminó con código {status}"
        )))
    }
}

/// Lee el estado actual desde `wpctl get-volume`.
fn current_state() -> Option<VolumeState> {
    let output = Command::new("wpctl")
        .args(["get-volume", "@DEFAULT_AUDIO_SINK@"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8(output.stdout).ok()?;
    VolumeState::parse_wpctl(&text)
}
