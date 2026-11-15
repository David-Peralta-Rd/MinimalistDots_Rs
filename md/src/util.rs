//! Utilidades compartidas de los subcomandos de `md`.

use std::process::Command;

/// Marca temporal para nombres de archivo (`%y%m%d_%Hh%Mm%Ss`).
pub fn now_stamp() -> String {
    Command::new("date")
        .args(["+%y%m%d_%Hh%Mm%Ss"])
        .output()
        .ok()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .unwrap_or_default()
}
