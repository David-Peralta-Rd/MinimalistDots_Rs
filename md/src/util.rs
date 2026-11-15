//! Utilidades compartidas de los subcomandos de `md`.

use std::io::Write;
use std::process::{Command, Stdio};

/// Marca temporal para nombres de archivo (`%y%m%d_%Hh%Mm%Ss`).
pub fn now_stamp() -> String {
    Command::new("date")
        .args(["+%y%m%d_%Hh%Mm%Ss"])
        .output()
        .ok()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .unwrap_or_default()
}

/// Segundos Unix actuales (sin subproceso).
pub fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}

/// Desfase local respecto a UTC en segundos (de `date +%z`); `0` si falla.
pub fn utc_offset_seconds() -> i64 {
    let output = match Command::new("date").arg("+%z").output() {
        Ok(output) => output,
        Err(_) => return 0,
    };
    let text = String::from_utf8_lossy(&output.stdout);
    md_core::parse_utc_offset(text.trim()).unwrap_or(0)
}

/// Copia `text` al portapapeles con `wl-copy`.
pub fn copy_to_clipboard(text: &str) -> Result<(), std::io::Error> {
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

/// Envía una notificación con `notify-send` (la recibe nuestro propio demonio).
/// Los fallos se ignoran: si el demonio no está corriendo no hay a quién avisar.
pub fn notify_send(app: &str, timeout_ms: u32, summary: &str, body: &str) {
    let timeout = timeout_ms.to_string();
    let mut command = Command::new("notify-send");
    command
        .arg("-a")
        .arg(app)
        .arg("-t")
        .arg(&timeout)
        .arg(summary);
    if !body.is_empty() {
        command.arg(body);
    }
    let _ = command.status();
}
