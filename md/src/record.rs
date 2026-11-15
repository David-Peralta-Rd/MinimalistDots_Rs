//! Subcomando `record`: grabación de pantalla con `gpu-screen-recorder`.
//!
//! Reemplaza a `screenrecord.sh`. Las notificaciones de progreso/resultado
//! llegarán con `md notify` (H6); por ahora el resultado es silencioso.

use std::path::PathBuf;
use std::process::Command;

use clap::ValueEnum;

/// Modo de grabación.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Mode {
    /// Graba un área o ventana (selección con `slurp`).
    Snip,
    /// Graba el monitor completo.
    Monitor,
    /// Detiene cualquier grabación en curso.
    Toggle,
}

/// Ejecuta la grabación o la detiene.
pub fn run(mode: Mode, audio: bool) -> Result<(), Box<dyn std::error::Error>> {
    if mode == Mode::Toggle {
        stop_active_recording()?;
        return Ok(());
    }

    // Si ya hay una grabación en curso, se detiene antes de empezar otra.
    stop_active_recording()?;

    let save_dir = recordings_dir();
    std::fs::create_dir_all(&save_dir)?;
    let save_path = save_dir.join(format!("{}_recording.mp4", crate::util::now_stamp()));

    let mut args: Vec<String> = vec![
        "-c".into(),
        "mp4".into(),
        "-q".into(),
        "high".into(),
        "-k".into(),
        "hevc".into(),
        "-f".into(),
        "60".into(),
    ];

    match mode {
        Mode::Snip => {
            let Some(geometry) = select_area()? else {
                return Ok(()); // selección cancelada
            };
            args.push("-w".into());
            args.push(geometry);
        }
        Mode::Monitor => {
            args.push("-w".into());
            args.push("screen".into());
        }
        Mode::Toggle => unreachable!(),
    }

    if audio {
        args.push("-a".into());
        args.push("default".into());
    }

    let status = Command::new("gpu-screen-recorder")
        .args(&args)
        .arg("-o")
        .arg(&save_path)
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(Box::new(std::io::Error::other(
            "gpu-screen-recorder terminó con error",
        )))
    }
}

/// Detiene cualquier grabación en curso (SIGINT para guardar bien el vídeo).
fn stop_active_recording() -> Result<(), std::io::Error> {
    let output = Command::new("pgrep")
        .args(["-f", "gpu-screen-recorder"])
        .output()?;
    let pids = String::from_utf8_lossy(&output.stdout);
    for pid in pids.split_whitespace() {
        let _ = Command::new("kill").args(["-INT", pid]).status();
    }
    Ok(())
}

/// Devuelve la geometría del área elegida con `slurp` (`None` si se cancela).
fn select_area() -> Result<Option<String>, std::io::Error> {
    let output = Command::new("slurp").args(["-f", "%wx%h+%x+%y"]).output()?;
    let geometry = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    Ok((!geometry.is_empty()).then_some(geometry))
}

/// Directorio de grabaciones (`$XDG_VIDEOS_DIR/recordings`).
fn recordings_dir() -> PathBuf {
    let base = std::env::var_os("XDG_VIDEOS_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join("multimedia/videos"))
        });
    base.map(|dir| dir.join("recordings"))
        .unwrap_or_else(|| PathBuf::from("recordings"))
}
