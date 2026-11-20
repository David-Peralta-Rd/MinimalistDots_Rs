//! Subcomando `record`: grabación de pantalla con `gpu-screen-recorder`.
//!
//! Reemplaza a `screenrecord.sh`. Los avisos de progreso/resultado se envían con
//! `notify-send` (los recibe el demonio `md notify serve`).

use std::path::{Path, PathBuf};
use std::process::Command;

use clap::ValueEnum;

use crate::i18n::Texts;

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
    let texts = Texts::load();
    if mode == Mode::Toggle {
        if stop_active_recording()? {
            notify(
                &texts.t("escritorio.grabacion_detenida"),
                &texts.t("escritorio.grabacion_procesando"),
            );
        } else {
            notify("Screen Recorder", &texts.t("escritorio.no_grabacion"));
        }
        return Ok(());
    }

    // Si ya hay una grabación en curso, se detiene y se sale (igual que el
    // script original: pulsa de nuevo para empezar otra).
    if stop_active_recording()? {
        notify(
            &texts.t("escritorio.grabacion_detenida"),
            &texts.t("escritorio.grabacion_procesando"),
        );
        return Ok(());
    }

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
                notify(
                    &texts.t("escritorio.grabacion_cancelada"),
                    &texts.t("escritorio.no_area"),
                );
                return Ok(());
            };
            // En esta versión de gpu-screen-recorder `-region` está obsoleto;
            // la geometría se pasa directamente a `-w`.
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
        args.push("default_output".into());
    }

    notify(
        &texts.t("escritorio.grabacion_iniciada"),
        &texts.t("escritorio.grabacion_iniciada_cuerpo"),
    );

    // En primer plano: al recibir SIGINT (desde `md record toggle`) la
    // herramienta guarda el vídeo y termina.
    let status = Command::new("gpu-screen-recorder")
        .args(&args)
        .arg("-o")
        .arg(&save_path)
        .status()?;

    if status.success() {
        if save_path.exists() {
            let ruta = save_path.display().to_string();
            let body = texts.t_fill("escritorio.grabacion_guardada_cuerpo", &[("ruta", &ruta)]);
            notify(&texts.t("escritorio.grabacion_guardada"), &body);
        }
        Ok(())
    } else {
        Err(Box::new(std::io::Error::other(
            "gpu-screen-recorder terminó con error",
        )))
    }
}

/// Aviso de grabación (`notify-send` con la app "Screen Recorder").
fn notify(summary: &str, body: &str) {
    crate::util::notify_send("Screen Recorder", 3000, summary, body);
}

/// Detiene cualquier grabación en curso (SIGINT para guardar bien el vídeo).
/// Devuelve `true` si había algún proceso que detener.
fn stop_active_recording() -> Result<bool, std::io::Error> {
    // `pgrep` por nombre falla con nombres >15 chars (límite del kernel), así que
    // recorremos /proc y filtramos por el ejecutable real (argv[0]).
    let mut stopped = false;
    for entry in std::fs::read_dir("/proc")? {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        let name = entry.file_name();
        let name = match name.to_str() {
            Some(n) => n,
            None => continue,
        };
        if !name.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        let cmdline = match std::fs::read(format!("/proc/{name}/cmdline")) {
            Ok(b) => b,
            Err(_) => continue,
        };
        if cmdline.is_empty() {
            continue;
        }
        let argv0 = cmdline.split(|&b| b == 0).next().unwrap_or(&[]);
        let argv0 = String::from_utf8_lossy(argv0);
        let base = argv0.rsplit('/').next().unwrap_or(&argv0);
        if base == "gpu-screen-recorder" {
            let _ = Command::new("kill").args(["-INT", name]).status();
            stopped = true;
        }
    }
    Ok(stopped)
}

/// Devuelve la geometría del área elegida con `slurp` (`None` si se cancela).
fn select_area() -> Result<Option<String>, std::io::Error> {
    let output = Command::new("slurp").args(["-f", "%wx%h+%x+%y"]).output()?;
    let geometry = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    Ok((!geometry.is_empty()).then_some(geometry))
}

/// Directorio de grabaciones (`$XDG_VIDEOS_DIR/recordings`).
fn recordings_dir() -> PathBuf {
    // Respeta XDG_VIDEOS_DIR si apunta a un directorio de videos real; si no
    // (vacío o HOME), usa la ruta histórica del dotfiles: ~/multimedia/videos.
    let base = std::env::var_os("XDG_VIDEOS_DIR")
        .map(PathBuf::from)
        .filter(|dir| dir.as_path() != Path::new("") && dir.as_path() != dirs_home().as_path())
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join("multimedia/videos"))
        });
    base.map(|dir| dir.join("recordings"))
        .unwrap_or_else(|| PathBuf::from("recordings"))
}

fn dirs_home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
}
