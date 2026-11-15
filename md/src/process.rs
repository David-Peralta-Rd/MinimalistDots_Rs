//! Subcomando `process`: gestor de procesos (listar y matar).
//!
//! Reemplaza a `process_manager.sh`. Versión de lista única por `ps`; la
//! división GUI/sistema del script original se puede añadir después.

use md_core::{MenuEntry, MenuMode, parse_ps};
use md_ui::{MenuConfig, MenuEvent, SlintHost, run_menu};

/// Ejecuta el gestor de procesos.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let processes = list_processes()?;
    let entries: Vec<MenuEntry> = processes
        .iter()
        .map(|process| MenuEntry::new(process.label.clone()))
        .collect();

    let config = MenuConfig {
        app_id: "md.process".to_owned(),
        title: "Gestor de procesos".to_owned(),
        prompt: "⚙️".to_owned(),
        accent: md_core::Color::new(0xe0, 0x6c, 0x75), // rojo
        row_height: 36.0,
        width: 640.0,
        height: 480.0,
        can_delete: false,
        can_wipe: false,
        submit_empty: false,
    };

    if let MenuEvent::Selected(index) = run_menu(&SlintHost, &entries, MenuMode::Dmenu, &config)?
        && let Some(process) = processes.get(index)
    {
        kill(process.pid)?;
        crate::util::notify_send(
            "Process Manager",
            2500,
            "Proceso terminado",
            &format!("Proceso cerrado con PID: {}", process.pid),
        );
    }
    Ok(())
}

/// Lista los procesos del usuario vía `ps`, ordenados por memoria.
fn list_processes() -> Result<Vec<md_core::ProcessEntry>, std::io::Error> {
    let mut command = std::process::Command::new("ps");
    command.args(["-o", "pid=,comm=,%mem=,%cpu=", "--sort=-%mem"]);
    let user = std::env::var("USER").unwrap_or_default();
    if !user.is_empty() {
        command.args(["-u", &user]);
    }
    let output = command.output()?;
    if !output.status.success() {
        return Err(std::io::Error::other("ps terminó con error"));
    }
    Ok(parse_ps(&String::from_utf8_lossy(&output.stdout)))
}

/// Mata el proceso (SIGKILL, igual que el script original).
fn kill(pid: u32) -> Result<(), std::io::Error> {
    let pid = pid.to_string();
    let status = std::process::Command::new("kill")
        .args(["-9", pid.as_str()])
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "no se pudo matar el PID {pid}"
        )))
    }
}
