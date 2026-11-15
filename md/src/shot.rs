//! Subcomando `shot`: capturas de pantalla con `grimblast`.
//!
//! Reemplaza a `screenshot.sh`. Las notificaciones de resultado llegarán con
//! `md notify` (H6); por ahora el resultado es silencioso.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use clap::ValueEnum;

/// Modo de captura.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum Mode {
    /// Captura todas las pantallas.
    #[value(alias = "p")]
    Printscreen,
    /// Selecciona un área o ventana.
    #[value(alias = "s")]
    Snip,
    /// Selecciona un área con la pantalla congelada.
    #[value(alias = "sf")]
    Snapfreeze,
    /// Captura el monitor enfocado.
    #[value(alias = "m")]
    Monitor,
    /// Extrae texto (OCR) y lo copia al portapapeles.
    #[value(alias = "sc")]
    Scan,
    /// Escanea un código QR y copia el resultado.
    #[value(alias = "sq")]
    Qr,
}

/// Ejecuta la captura.
pub fn run(mode: Mode) -> Result<(), Box<dyn std::error::Error>> {
    let save_dir = screenshots_dir();
    std::fs::create_dir_all(&save_dir)?;
    let save_file = save_dir.join(format!("{}_screenshot.png", crate::util::now_stamp()));
    let temp = temp_file();

    match mode {
        Mode::Scan => ocr(&temp)?,
        Mode::Qr => qr(&temp)?,
        _ => capture(mode, &temp, &save_file)?,
    }
    Ok(())
}

/// Captura (y anota si hay herramienta) guardando en `save_file`.
fn capture(mode: Mode, temp: &Path, save_file: &Path) -> Result<(), std::io::Error> {
    let (grimblast_mode, freeze) = match mode {
        Mode::Printscreen => ("screen", false),
        Mode::Snip => ("area", false),
        Mode::Snapfreeze => ("area", true),
        Mode::Monitor => ("output", false),
        Mode::Scan | Mode::Qr => unreachable!(),
    };

    if let Some(tool) = annotation_tool() {
        grimblast_copysave(grimblast_mode, freeze, temp)?;
        run_annotation(tool, temp, save_file)?;
    } else {
        grimblast_copysave(grimblast_mode, freeze, save_file)?;
    }
    Ok(())
}

/// Captura un área (congelada) y copia el texto extraído con OCR.
fn ocr(temp: &Path) -> Result<(), std::io::Error> {
    grimblast_copysave("area", true, temp)?;
    let lang = std::env::var("SCREENSHOT_OCR_LANG").unwrap_or_else(|_| "eng".to_owned());
    let output = Command::new("tesseract")
        .arg(temp)
        .arg("stdout")
        .arg("-l")
        .arg(&lang)
        .output()?;
    let text = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if text.is_empty() {
        return Ok(());
    }
    copy_to_clipboard(&text)
}

/// Captura un área (congelada) y copia el contenido del código QR.
fn qr(temp: &Path) -> Result<(), std::io::Error> {
    grimblast_copysave("area", true, temp)?;
    let output = Command::new("zbarimg")
        .args(["--raw", "-q"])
        .arg(temp)
        .output()?;
    let text = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if text.is_empty() {
        return Ok(());
    }
    copy_to_clipboard(&text)
}

/// `grimblast [--freeze] copysave <modo> <destino>`.
fn grimblast_copysave(mode: &str, freeze: bool, target: &Path) -> Result<(), std::io::Error> {
    let mut command = Command::new("grimblast");
    if freeze {
        command.arg("--freeze");
    }
    let status = command.args(["copysave", mode]).arg(target).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("grimblast terminó con error"))
    }
}

/// Herramienta de anotación disponible (`swappy` o `satty`).
fn annotation_tool() -> Option<&'static str> {
    if command_exists("swappy") {
        Some("swappy")
    } else if command_exists("satty") {
        Some("satty")
    } else {
        None
    }
}

/// Abre la herramienta de anotación sobre `temp` y guarda en `save_file`.
fn run_annotation(tool: &str, temp: &Path, save_file: &Path) -> Result<(), std::io::Error> {
    let status = match tool {
        "swappy" => Command::new("swappy")
            .arg("-o")
            .arg(save_file)
            .arg("-f")
            .arg(temp)
            .status()?,
        "satty" => Command::new("satty")
            .arg("-o")
            .arg(save_file)
            .arg("-f")
            .arg(temp)
            .arg("--copy-command")
            .arg("wl-copy")
            .status()?,
        _ => {
            return Err(std::io::Error::other(
                "herramienta de anotación desconocida",
            ));
        }
    };
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("la edición no terminó correctamente"))
    }
}

/// Copia `text` al portapapeles con `wl-copy`.
fn copy_to_clipboard(text: &str) -> Result<(), std::io::Error> {
    let mut wl = Command::new("wl-copy").stdin(Stdio::piped()).spawn()?;
    if let Some(mut stdin) = wl.stdin.take() {
        stdin.write_all(text.as_bytes())?;
    }
    let status = wl.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("wl-copy terminó con error"))
    }
}

/// Comprueba si un ejecutable está en el `PATH`.
fn command_exists(name: &str) -> bool {
    Command::new("which")
        .arg(name)
        .output()
        .is_ok_and(|out| out.status.success())
}

/// Directorio de capturas (`$XDG_PICTURES_DIR/screenshots`).
fn screenshots_dir() -> PathBuf {
    let base = std::env::var_os("XDG_PICTURES_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join("multimedia/pictures"))
        });
    base.map(|dir| dir.join("screenshots"))
        .unwrap_or_else(|| PathBuf::from("screenshots"))
}

/// Archivo temporal para capturas (lo usa `grimblast` antes de anotar).
fn temp_file() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("hypr_screenshot.png")
}
