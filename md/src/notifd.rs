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
use std::sync::Mutex;
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

/// Estado de una notificación activa.
#[derive(Clone, serde::Serialize)]
struct ActiveNotification {
    id: u64,
    app: String,
    summary: String,
    body: String,
    urgency: String,
    progress: i32,
}

/// Estado compartido del demonio.
#[derive(Clone)]
struct DaemonState {
    active: Arc<Mutex<Vec<ActiveNotification>>>,
    expires: Arc<Mutex<Vec<(u64, std::time::Instant, Option<Duration>)>>>,
    next_id: Arc<std::sync::atomic::AtomicU64>,
    dnd: Arc<AtomicBool>,
}

const STATE_PATH: &str = "/tmp/md-notifications.json";

fn write_state(active: &Arc<Mutex<Vec<ActiveNotification>>>) {
    let Ok(guard) = active.lock() else { return };
    let _ = std::fs::write(STATE_PATH, serde_json::to_string(&*guard).unwrap_or_else(|_| "[]".to_owned()));
}

/// Arranca el demonio: sirve DBus y lanza QuickShell.
pub fn serve() -> Result<(), Box<dyn std::error::Error>> {
    let active: Arc<Mutex<Vec<ActiveNotification>>> = Arc::new(Mutex::new(Vec::new()));
    let expires: Arc<Mutex<Vec<(u64, std::time::Instant, Option<Duration>)>>> = Arc::new(Mutex::new(Vec::new()));
    let next_id = Arc::new(std::sync::atomic::AtomicU64::new(1));
    let dnd = Arc::new(AtomicBool::new(false));

    let state = DaemonState {
        active: active.clone(),
        expires: expires.clone(),
        next_id: next_id.clone(),
        dnd: dnd.clone(),
    };

    let _connection = build_connection(&state)?;

    truncate_history(HISTORY_KEEP);

    // Lanza QuickShell residente.
    let _ = std::process::Command::new("qs")
        .arg("-p")
        .arg(format!("{}/.local/bin/MinimalistDots_Rs/quickshell/Notifications.qml", std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_owned())))
        .env("MD_NOTIFICATIONS_STATE", STATE_PATH)
        .env("QML_XHR_ALLOW_FILE_READ", "1")
        .spawn();

    write_state(&active);

    // Hilo de expiración.
    let active_exp = active.clone();
    let expires_exp = expires.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_millis(200));
            let now = std::time::Instant::now();
            let mut changed = false;
            if let Ok(mut guard) = expires_exp.lock() {
                let expired: Vec<u64> = guard
                    .iter()
                    .filter(|(_, t, timeout)| timeout.is_some_and(|d| *t + d < now))
                    .map(|(id, _, _)| *id)
                    .collect();
                if !expired.is_empty() {
                    changed = true;
                    guard.retain(|(id, _, _)| !expired.contains(id));
                    if let Ok(mut a) = active_exp.lock() {
                        a.retain(|n| !expired.contains(&n.id));
                    }
                }
            }
            if changed {
                write_state(&active_exp);
            }
        }
    });

    // Mantiene el proceso vivo.
    loop {
        std::thread::sleep(Duration::from_secs(3600));
    }
}

/// Interfaz `org.freedesktop.Notifications`.
struct NotificationsIface {
    state: DaemonState,
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

        let id = self.state.next_id.fetch_add(1, Ordering::Relaxed);

        // Reemplazo: cierra la notificación previa (si existía).
        if replaces_id > 0 {
            self.state.active.lock().unwrap().retain(|n| n.id != replaces_id as u64);
            self.state.expires.lock().unwrap().retain(|(nid, _, _)| *nid != replaces_id as u64);
        }

        let critical = urgency.is_critical();
        let dnd = self.state.dnd.load(Ordering::SeqCst);

        // En "no molestar" solo se muestran las críticas; el resto se registra
        // sin mostrarse (pero devolvemos un id válido).
        if dnd && !critical {
            return id as u32;
        }

        let progress = progress_from_hints(&hints);
        let timeout = timeout_for(urgency, expire_timeout, self.default_timeout);

        let item = ActiveNotification {
            id,
            app: app_name,
            summary,
            body,
            urgency: urgency.as_str().to_owned(),
            progress,
        };
        if let Ok(mut guard) = self.state.active.lock() {
            guard.insert(0, item);
            if guard.len() > MAX_POPUPS {
                guard.truncate(MAX_POPUPS);
            }
        }
        if let Ok(mut guard) = self.state.expires.lock() {
            guard.push((id, std::time::Instant::now(), timeout));
        }
        write_state(&self.state.active);
        id as u32
    }

    fn close_notification(&self, id: u32) {
        if let Ok(mut guard) = self.state.active.lock() {
            guard.retain(|n| n.id != id as u64);
        }
        if let Ok(mut guard) = self.state.expires.lock() {
            guard.retain(|(nid, _, _)| *nid != id as u64);
        }
        write_state(&self.state.active);
    }
}

/// Interfaz de control propia (toggle/status/limpiar popups).
struct ControlIface {
    state: DaemonState,
}

#[interface(name = "com.github.MinimalistDots.Notifd")]
impl ControlIface {
    fn toggle_dnd(&self) -> String {
        let now_on = !self.state.dnd.fetch_xor(true, Ordering::SeqCst);
        if now_on { "on" } else { "off" }.to_owned()
    }

    fn status(&self) -> String {
        if self.state.dnd.load(Ordering::SeqCst) {
            "dnd"
        } else {
            "normal"
        }
        .to_owned()
    }

    fn clear_popups(&self) {
        if let Ok(mut guard) = self.state.active.lock() {
            guard.clear();
        }
        if let Ok(mut guard) = self.state.expires.lock() {
            guard.clear();
        }
        write_state(&self.state.active);
    }
}

/// Construye la conexión de sesión y sirve ambas interfaces bajo el nombre
/// `org.freedesktop.Notifications` (la interfaz de control vive en otra ruta).
fn build_connection(
    state: &DaemonState,
) -> Result<zbus::blocking::Connection, zbus::Error> {
    Builder::session()?
        .name("org.freedesktop.Notifications")?
        .serve_at(
            "/org/freedesktop/Notifications",
            NotificationsIface {
                state: state.clone(),
                default_timeout: DEFAULT_TIMEOUT,
            },
        )?
        .serve_at(
            "/com/github/MinimalistDots/Notifd",
            ControlIface {
                state: state.clone(),
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
