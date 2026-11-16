//! Puerto de alto nivel para `MdNotifications`: la pila de notificaciones del
//! demonio.
//!
//! El demonio (en `md`) sirve la interfaz DBus en un hilo y llama a
//! [`Notifications::push`]/[`Notifications::close`]/[`Notifications::dismiss_all`]
//! desde ahí. Este módulo traslada esas llamadas al hilo del bucle de Slint con
//! `invoke_from_event_loop` y reconstruye el modelo de la ventana.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use md_core::Urgency;
use slint::ComponentHandle;

use crate::generated::{MdNotifications, NotificationItem};

/// Alto fijo de cada tarjeta y separación entre ellas. El alto de la ventana se
/// calcula a partir de estos valores para ajustarse exactamente al contenido.
const CARD_HEIGHT: f32 = 76.0;
const SPACING: f32 = 8.0;

/// Intervalo del temporizador que expira las notificaciones vencidas.
const EXPIRY_TICK: Duration = Duration::from_millis(200);

/// Datos de una notificación a mostrar (proyección del `Notification` de md-core).
#[derive(Debug, Clone)]
pub struct NotificationData {
    pub app: String,
    pub summary: String,
    pub body: String,
    pub urgency: Urgency,
    /// Nivel 0–100 para la barra; `-1` la oculta.
    pub progress: i32,
}

/// Una notificación activa (con su fecha de expiración).
struct Active {
    id: u64,
    data: NotificationData,
    expires_at: Option<Instant>,
}

/// Manejador de la pila de notificaciones. Es `Send + Sync`: se puede clonar y
/// entregar al hilo DBus para empujar/cerrar notificaciones desde fuera del
/// bucle de eventos.
#[derive(Clone)]
pub struct Notifications {
    weak: slint::Weak<MdNotifications>,
    active: Arc<Mutex<Vec<Active>>>,
    next_id: Arc<AtomicU64>,
    max_popups: usize,
}

impl Notifications {
    /// Crea la ventana, conecta la expiración y devuelve el manejador junto con
    /// el componente (que luego se pasa a [`Self::run`]). Debe llamarse en el
    /// hilo principal, antes de arrancar el bucle de eventos.
    ///
    /// `max_popups` limita cuántas tarjetas se muestran a la vez (la más antigua
    /// se descarta al superarlo).
    pub fn create(max_popups: usize) -> Result<(Self, MdNotifications), slint::PlatformError> {
        let component = MdNotifications::new()?;
        // `MdNotifications::new()` inicializa el backend: el app-id ya se puede
        // fijar antes de mostrar la ventana (lo necesitan las windowrules).
        slint::set_xdg_app_id("md.notifications".to_owned())?;
        component.set_md_width(360.0);
        component.set_md_height(CARD_HEIGHT);

        let weak = component.as_weak();
        let active: Arc<Mutex<Vec<Active>>> = Arc::new(Mutex::new(Vec::new()));
        let next_id = Arc::new(AtomicU64::new(1));

        // Un único temporizador (en el hilo del bucle) expira las notificaciones
        // vencidas. Se mantiene vivo con `mem::forget`: vive lo que dure el
        // proceso (el demonio es residente).
        let timer = slint::Timer::default();
        let timer_weak = weak.clone();
        let timer_active = active.clone();
        timer.start(slint::TimerMode::Repeated, EXPIRY_TICK, move || {
            let changed = {
                let Ok(mut guard) = timer_active.lock() else {
                    return;
                };
                let now = Instant::now();
                let before = guard.len();
                guard.retain(|item| item.expires_at.is_none_or(|t| t > now));
                guard.len() != before
            };
            if changed && let Some(component) = timer_weak.upgrade() {
                refresh(&component, &timer_active);
            }
        });
        std::mem::forget(timer);

        component.on_close_requested({
            let weak = weak.clone();
            let active = active.clone();
            move |index: i32| {
                if let Ok(mut guard) = active.lock()
                    && let Ok(index) = usize::try_from(index)
                    && index < guard.len()
                {
                    guard.remove(index);
                }
                if let Some(component) = weak.upgrade() {
                    refresh(&component, &active);
                }
            }
        });

        let handle = Self {
            weak,
            active,
            next_id,
            max_popups,
        };
        Ok((handle, component))
    }

    /// Reserva un id sin mostrar ninguna notificación (para el modo no-molestar,
    /// donde el demonio devuelve un id pero no muestra la tarjeta).
    pub fn reserve_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Añade una notificación y la muestra. Devuelve el id asignado.
    ///
    /// `timeout = None` significa "no expira sola" (críticas o `expire_timeout = 0`).
    pub fn push(&self, data: NotificationData, timeout: Option<Duration>) -> u64 {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let expires_at = timeout.map(|duration| Instant::now() + duration);
        if let Ok(mut guard) = self.active.lock() {
            guard.insert(
                0,
                Active {
                    id,
                    data,
                    expires_at,
                },
            );
            if guard.len() > self.max_popups {
                guard.truncate(self.max_popups);
            }
        }
        self.request_refresh();
        id
    }

    /// Cierra una notificación por id (si sigue visible).
    pub fn close(&self, id: u64) {
        if let Ok(mut guard) = self.active.lock() {
            guard.retain(|item| item.id != id);
        }
        self.request_refresh();
    }

    /// Cierra todas las notificaciones visibles.
    pub fn dismiss_all(&self) {
        if let Ok(mut guard) = self.active.lock() {
            guard.clear();
        }
        self.request_refresh();
    }

    /// Ejecuta el bucle de eventos del componente (bloquea hasta que se cierra).
    pub fn run(component: MdNotifications) -> Result<(), slint::PlatformError> {
        component.run()
    }

    /// Pide al bucle de eventos que reconstruya el modelo (se puede llamar desde
    /// cualquier hilo).
    fn request_refresh(&self) {
        let weak = self.weak.clone();
        let active = self.active.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(component) = weak.upgrade() {
                refresh(&component, &active);
            }
        });
    }
}

/// Reconstruye el modelo y ajusta la ventana (en el hilo del bucle).
fn refresh(component: &MdNotifications, active: &Arc<Mutex<Vec<Active>>>) {
    let items: Vec<NotificationItem> = {
        let Ok(guard) = active.lock() else {
            return;
        };
        guard.iter().map(to_item).collect()
    };

    let count = items.len();
    component.set_notifications(slint::VecModel::from_slice(&items));
    if count == 0 {
        // Mantener la ventana mapeada (evita que `show()` no la recupere en
        // Wayland tras un `hide()`); altura 0 la hace invisible.
        component.set_md_height(0.0);
        let _ = component.show();
    } else {
        let height = count as f32 * CARD_HEIGHT + (count as f32 - 1.0) * SPACING;
        component.set_md_height(height);
        let _ = component.show();
    }
}

/// Convierte una notificación activa a la fila que pinta Slint.
fn to_item(active: &Active) -> NotificationItem {
    NotificationItem {
        app: active.data.app.clone().into(),
        summary: active.data.summary.clone().into(),
        body: active.data.body.clone().into(),
        urgency: active.data.urgency.as_str().into(),
        progress: active.data.progress,
    }
}
