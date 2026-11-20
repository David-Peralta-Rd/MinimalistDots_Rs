//! Puente mínimo con QuickShell: lanza archivos QML del directorio instalado.
//!
//! Centraliza la ruta de instalación y el lanzamiento para evitar duplicar
//! lógica en cada subcomando migrado desde Slint.

use std::process::Command;

/// Lanza un archivo QML instalado (p. ej. "Osd.qml") con variables de entorno.
pub fn spawn(qml_file: &str, envs: &[(&str, &str)]) -> std::io::Result<()> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_owned());
    let qml_path = format!("{home}/.local/bin/MinimalistDots_Rs/quickshell/{qml_file}");

    let mut cmd = Command::new("qs");
    cmd.arg("-p").arg(&qml_path);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    cmd.spawn()?;
    Ok(())
}

/// Mata instancias previas de un archivo QML para evitar solapamiento.
pub fn kill(qml_file: &str) {
    let pattern = format!("quickshell.*{}", qml_file.replace('.', "\\."));
    let _ = Command::new("pkill").args(["-f", &pattern]).status();
}

/// Lanza un archivo QML y espera a que termine. Útil para menús modales.
pub fn spawn_wait(qml_file: &str, envs: &[(&str, &str)]) -> std::io::Result<std::process::ExitStatus> {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_owned());
    let qml_path = format!("{home}/.local/bin/MinimalistDots_Rs/quickshell/{qml_file}");

    let mut cmd = Command::new("qs");
    cmd.arg("-p").arg(&qml_path);
    for (key, value) in envs {
        cmd.env(key, value);
    }
    let mut child = cmd.spawn()?;
    child.wait()
}