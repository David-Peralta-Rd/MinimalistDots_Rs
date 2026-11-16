//! Demonio de notificaciones (`md notify serve`): sirve la interfaz
//! `org.freedesktop.Notifications` (y una interfaz de control propia) y muestra
//! las tarjetas con el widget `MdNotifications`.
//!
//! Reemplaza a `notifd.qml` (Quickshell). La v1 no emite las señales
//! `NotificationClosed`/`ActionInvoked` ni invoca acciones (quedan para después);
//! sí soporta urgencia, progreso, no-molestar e historial.

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use md_core::Urgency;
use md_ui::{NotificationData, Notifications};
use zbus::blocking::connection::Builder;
use zbus::interface;
use zbus::zvariant::OwnedValue;

/// Tiempo visible por defecto (ms) para notificaciones no críticas sin `timeout`.
const DEFAULT_TIMEOUT: Duration = Duration::from_millis(10000);
const CRITICAL_TIMEOUT: Duration = Duration::from_millis(30000);
/// Número máximo de tarjetas visibles a la vez.
const MAX_POPUPS: usize = 5;
/// Líneas de historial que se conservan al arrancar.
const HISTORY_KEEP: usize = 300;
/// Aplicación cuyos avisos no se guardan en el historial (el propio menú).
const IGNORED_APP: &str = "Historial";

/// Arranca el demonio: construye la ventana, sirve DBus y entra en el bucle.
pub fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let (notifications, component) = Notifications::create(MAX_POPUPS)?;

    let dnd = Arc::new(AtomicBool::new(false));
    let _connection = build_connection(&notifications, dnd.clone())?;

    truncate_history(HISTORY_KEEP);

    // Bloquea hasta que se cierra la ventana (el demonio es residente). El
    // `_connection` sigue vivo durante el bucle, sirviendo DBus en su hilo.
    Notifications::run(component)?;
    Ok(())
}

/// Interfaz `org.freedesktop.Notifications`.
struct NotificationsIface {
    notifications: Notifications,
    dnd: Arc<AtomicBool>,
    default_timeout: Duration,
}

#[interface(name = "org.freedesktop.Notifications")]
impl NotificationsIface {
    fn get_server_information(&self) -> (String, String, String, String) {
        (
            "MinimalistDots".to_owned(),
            "MinimalistDots".to_owned(),
            env!("CARGO_PKG_VERSION").to_owned(),
            "1.2".to_owned(),
        )
    }

    fn get_capabilities(&self) -> Vec<String> {
        vec![
            "body".to_owned(),
            "persistence".to_owned(),
            "actions".to_owned(),
        ]
    }

    #[allow(clippy::too_many_arguments)]
    fn notify(
        &self,
        app_name: String,
        replaces_id: u32,
        _app_icon: String,
        summary: String,
        body: String,
        _actions: Vec<String>,
        hints: HashMap<String, OwnedValue>,
        expire_timeout: i32,
    ) -> u32 {
        let urgency = urgency_from_hints(&hints);

        // Historial (los avisos del propio menú no ensucian el historial).
        if app_name != IGNORED_APP {
            let notification = md_core::Notification {
                time: crate::util::now_unix(),
                app: app_name.clone(),
                summary: summary.clone(),
                body: body.clone(),
                urgency,
            };
            let _ = append_history(&notification);
        }

        // Reemplazo: cierra la notificación previa (si existía).
        if replaces_id > 0 {
            self.notifications.close(u64::from(replaces_id));
        }

        let critical = urgency.is_critical();
        let dnd = self.dnd.load(Ordering::SeqCst);

        // En "no molestar" solo se muestran las críticas; el resto se registra
        // sin mostrarse (pero devolvemos un id válido).
        if dnd && !critical {
            return self.notifications.reserve_id() as u32;
        }

        let data = NotificationData {
            app: app_name,
            summary,
            body,
            urgency,
            progress: progress_from_hints(&hints),
        };
        let timeout = timeout_for(urgency, expire_timeout, self.default_timeout);
        self.notifications.push(data, timeout) as u32
    }

    fn close_notification(&self, id: u32) {
        self.notifications.close(u64::from(id));
    }
}

/// Interfaz de control propia (toggle/status/limpiar popups).
struct ControlIface {
    notifications: Notifications,
    dnd: Arc<AtomicBool>,
}

#[interface(name = "com.github.MinimalistDots.Notifd")]
impl ControlIface {
    fn toggle_dnd(&self) -> String {
        let now_on = !self.dnd.fetch_xor(true, Ordering::SeqCst);
        if now_on { "on" } else { "off" }.to_owned()
    }

    fn status(&self) -> String {
        if self.dnd.load(Ordering::SeqCst) {
            "dnd"
        } else {
            "normal"
        }
        .to_owned()
    }

    fn clear_popups(&self) {
        self.notifications.dismiss_all();
    }
}

/// Construye la conexión de sesión y sirve ambas interfaces bajo el nombre
/// `org.freedesktop.Notifications` (la interfaz de control vive en otra ruta).
fn build_connection(
    notifications: &Notifications,
    dnd: Arc<AtomicBool>,
) -> Result<zbus::blocking::Connection, zbus::Error> {
    Builder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at(
            "/org/freedesktop/Notifications",
            NotificationsIface {
                notifications: notifications.clone(),
                dnd: dnd.clone(),
                default_timeout: DEFAULT_TIMEOUT,
            },
        )?
        .serve_at(
            "/com/github/MinimalistDots/Notifd",
            ControlIface {
                notifications: notifications.clone(),
                dnd,
            },
        )?
        .build()
}

/// Extrae la urgencia del hint estándar `urgency` (byte, o `i32` por tolerancia).
fn urgency_from_hints(hints: &HashMap<String, OwnedValue>) -> Urgency {
    let byte = hints.get("urgency").and_then(|value| {
        value
            .downcast_ref::<u8>()
            .ok()
            .or_else(|| value.downcast_ref::<i32>().ok().map(|n| n as u8))
    });
    byte.map(Urgency::from_dbus).unwrap_or(Urgency::Normal)
}

/// Extrae el hint `value` (0–100) para la barra de progreso; `-1` si no existe.
fn progress_from_hints(hints: &HashMap<String, OwnedValue>) -> i32 {
    let level = hints.get("value").and_then(|value| {
        value
            .downcast_ref::<i32>()
            .ok()
            .or_else(|| value.downcast_ref::<u8>().ok().map(i32::from))
    });
    level.map(|n| n.clamp(0, 100)).unwrap_or(-1)
}

/// Convierte el `expire_timeout` de DBus a una duración (`None` = no expira).
fn timeout_for(urgency: Urgency, expire_timeout: i32, default: Duration) -> Option<Duration> {
    if expire_timeout > 0 {
        Some(Duration::from_millis(expire_timeout as u64))
    } else if urgency.is_critical() {
        Some(CRITICAL_TIMEOUT)
    } else {
        Some(default)
    }
}

/// Ruta del historial (`$XDG_STATE_HOME|~/.local/state` + `/MinimalistDots/notifications.jsonl`).
pub fn history_path() -> Option<std::path::PathBuf> {
    let base = std::env::var_os("XDG_STATE_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".local/state"))
        })?;
    Some(base.join("MinimalistDots/notifications.jsonl"))
}

/// Añade una notificación al historial (una línea JSON por entrada).
fn append_history(notification: &md_core::Notification) -> Result<(), std::io::Error> {
    let Some(path) = history_path() else {
        return Ok(());
    };
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)?;
    writeln!(file, "{}", notification.to_json_line())
}

/// Conserva solo las últimas `keep` líneas del historial.
pub fn truncate_history(keep: usize) {
    let Some(path) = history_path() else {
        return;
    };
    let Ok(content) = std::fs::read_to_string(&path) else {
        return;
    };
    let lines: Vec<&str> = content.lines().collect();
    if lines.len() <= keep {
        return;
    }
    let kept = lines[lines.len() - keep..].join("\n") + "\n";
    let _ = std::fs::write(&path, kept);
}

/// Borra el historial.
pub fn clear_history() -> Result<(), std::io::Error> {
    let Some(path) = history_path() else {
        return Ok(());
    };
    std::fs::write(path, b"")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hint_u8(value: u8) -> OwnedValue {
        OwnedValue::from(value)
    }

    fn hint_i32(value: i32) -> OwnedValue {
        OwnedValue::from(value)
    }

    #[test]
    fn maps_expire_timeout() {
        // > 0: se respeta (ms).
        assert_eq!(
            timeout_for(Urgency::Normal, 3000, DEFAULT_TIMEOUT),
            Some(Duration::from_millis(3000))
        );
        // críticas sin timeout explícito: no expiran solas.
        assert_eq!(
            timeout_for(Urgency::Critical, -1, DEFAULT_TIMEOUT),
            Some(CRITICAL_TIMEOUT)
        );
        assert_eq!(
            timeout_for(Urgency::Critical, 0, DEFAULT_TIMEOUT),
            Some(CRITICAL_TIMEOUT)
        );
        // no críticas sin timeout: usan el por defecto.
        assert_eq!(
            timeout_for(Urgency::Normal, -1, DEFAULT_TIMEOUT),
            Some(DEFAULT_TIMEOUT)
        );
    }

    #[test]
    fn reads_urgency_from_hints() {
        let mut hints = HashMap::new();
        hints.insert("urgency".to_owned(), hint_u8(2));
        assert_eq!(urgency_from_hints(&hints), Urgency::Critical);

        // tolera `i32` en vez de byte.
        let mut hints = HashMap::new();
        hints.insert("urgency".to_owned(), hint_i32(1));
        assert_eq!(urgency_from_hints(&hints), Urgency::Normal);

        // sin hint: normal.
        assert_eq!(urgency_from_hints(&HashMap::new()), Urgency::Normal);
    }

    #[test]
    fn reads_progress_from_hints() {
        let mut hints = HashMap::new();
        hints.insert("value".to_owned(), hint_i32(42));
        assert_eq!(progress_from_hints(&hints), 42);

        // se recorta a 0..100.
        let mut hints = HashMap::new();
        hints.insert("value".to_owned(), hint_i32(150));
        assert_eq!(progress_from_hints(&hints), 100);

        // sin hint: oculto.
        assert_eq!(progress_from_hints(&HashMap::new()), -1);
    }
}
