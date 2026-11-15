//! Subcomando `notify`: servidor de notificaciones, historial y control.
//!
//! Reemplaza a `notifications_menu.sh` (historial/dnd/dismiss/clear) y a
//! `notifd.qml` (el servidor residente, en `notifd.rs`).

use clap::Subcommand;

use md_core::{MenuEntry, MenuMode, Notification};
use md_ui::{MenuConfig, MenuEvent, SlintHost, run_menu};

use crate::i18n::Texts;

/// Acciones del subcomando `notify`.
#[derive(Debug, Clone, Subcommand)]
pub enum NotifyCommand {
    /// Servidor residente de notificaciones (lo arranca Hyprland al iniciar).
    Serve,

    /// Historial de notificaciones (elegir copia, no-molestar, borrar).
    History,

    /// Activa o desactiva el modo "no molestar".
    Dnd,

    /// Cierra las notificaciones visibles.
    Dismiss,

    /// Borra el historial.
    Clear,

    /// Imprime el estado del modo "no molestar" (`dnd` o `normal`).
    Status,
}

/// Despacha el subcomando.
pub fn run(command: NotifyCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        NotifyCommand::Serve => crate::notifd::serve(),
        NotifyCommand::History => history(),
        NotifyCommand::Dnd => dnd(),
        NotifyCommand::Dismiss => dismiss(),
        NotifyCommand::Clear => clear(),
        NotifyCommand::Status => status(),
    }
}

/// Llama a un método de la interfaz de control del demonio (sin argumentos).
fn control_call<R>(method: &str) -> Result<R, Box<dyn std::error::Error>>
where
    R: for<'d> zbus::zvariant::DynamicDeserialize<'d>,
{
    let connection = zbus::blocking::Connection::session()?;
    let proxy = zbus::blocking::Proxy::new(
        &connection,
        "org.freedesktop.Notifications",
        "/com/github/MinimalistDots/Notifd",
        "com.github.MinimalistDots.Notifd",
    )?;
    Ok(proxy.call(method, &())?)
}

/// ¿Está activo el modo "no molestar"? (`false` si el demonio no responde).
fn is_dnd() -> bool {
    matches!(control_call::<String>("Status"), Ok(state) if state == "dnd")
}

/// Alterna el modo "no molestar".
fn dnd() -> Result<(), Box<dyn std::error::Error>> {
    let texts = Texts::load();
    let result: String = control_call("ToggleDnd")?;
    if result == "on" {
        // Crítica: se muestra aunque el modo no-molestar quede activo.
        let summary = texts.t("escritorio.no_molestar_activado");
        let body = texts.t("escritorio.no_molestar_activado_cuerpo");
        let _ = std::process::Command::new("notify-send")
            .args([
                "-a",
                "Historial",
                "-u",
                "critical",
                "-t",
                "2500",
                &summary,
                &body,
            ])
            .status();
    } else {
        crate::util::notify_send(
            "Historial",
            2500,
            &texts.t("escritorio.no_molestar_desactivado"),
            "",
        );
    }
    Ok(())
}

/// Cierra las notificaciones visibles.
fn dismiss() -> Result<(), Box<dyn std::error::Error>> {
    control_call::<()>("ClearPopups")?;
    Ok(())
}

/// Borra el historial y avisa.
fn clear() -> Result<(), Box<dyn std::error::Error>> {
    let texts = Texts::load();
    crate::notifd::clear_history()?;
    crate::util::notify_send(
        "Historial",
        2500,
        &texts.t("escritorio.historial_borrado"),
        "",
    );
    Ok(())
}

/// Imprime el estado del modo "no molestar".
fn status() -> Result<(), Box<dyn std::error::Error>> {
    let state: String = control_call("Status")?;
    println!("{state}");
    Ok(())
}

/// Lee el historial (orden del archivo: más antigua primero), ignorando líneas
/// dañadas.
fn read_history() -> Vec<Notification> {
    let Some(path) = crate::notifd::history_path() else {
        return Vec::new();
    };
    let Ok(content) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    content
        .lines()
        .filter_map(Notification::from_json_line)
        .collect()
}

/// Historial interactivo: elegir una entrada la copia; las dos primeras filas
/// alternan "no molestar" y borran el historial.
fn history() -> Result<(), Box<dyn std::error::Error>> {
    let texts = Texts::load();
    let notifications = read_history();
    let offset = crate::util::utc_offset_seconds();

    let dnd_label = if is_dnd() {
        texts.t("escritorio.desactivar_no_molestar")
    } else {
        texts.t("escritorio.activar_no_molestar")
    };

    let mut entries = vec![
        MenuEntry::new(dnd_label),
        MenuEntry::new(texts.t("escritorio.borrar_historial")),
    ];
    // Orden de visualización: más reciente primero. `order` mapea la posición del
    // menú a su índice en el archivo (para copiar la entrada correcta).
    let order: Vec<usize> = (0..notifications.len()).rev().collect();
    for &file_index in &order {
        let notification = &notifications[file_index];
        let time_label = md_core::format_timestamp(notification.time, offset);
        let label = md_core::format_menu_line(notification, file_index, &time_label, 220);
        entries.push(MenuEntry::new(label));
    }

    let config = MenuConfig {
        app_id: "md.notify".to_owned(),
        title: texts.t("escritorio.notificaciones"),
        prompt: texts.t("escritorio.notificaciones_prompt"),
        accent: md_core::Color::new(0x7a, 0xa2, 0xf7),
        row_height: 36.0,
        width: 720.0,
        height: 440.0,
        can_delete: false,
        can_wipe: false,
        submit_empty: false,
        wipe_label: None,
        confirm_wipe_label: None,
    };

    if let MenuEvent::Selected(index) = run_menu(&SlintHost, &entries, MenuMode::Dmenu, &config)? {
        if index == 0 {
            return dnd();
        }
        if index == 1 {
            return clear();
        }
        if let Some(&file_index) = order.get(index - 2) {
            let notification = &notifications[file_index];
            let text = if notification.body.is_empty() {
                notification.summary.clone()
            } else {
                format!("{}\n{}", notification.summary, notification.body)
            };
            crate::util::copy_to_clipboard(&text)?;
            let preview: String = text.replace('\n', " ").chars().take(80).collect();
            crate::util::notify_send(
                "Historial",
                2500,
                &texts.t("escritorio.copiado_portapapeles"),
                &preview,
            );
        }
    }
    Ok(())
}
