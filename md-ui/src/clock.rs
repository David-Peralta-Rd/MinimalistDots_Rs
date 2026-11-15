//! Puerto de alto nivel para el widget `MdClock`: reloj persistente.
//!
//! No hay máquina de estados ni lógica: un temporizador de 1 s detecta el cambio
//! de minuto (con `SystemTime`, sin subproceso) y, al ocurrir, pide el texto a
//! `now` (que aporta el anfitrión `md` y respeta la localización del sistema).

use std::time::Duration;

use slint::ComponentHandle;

use crate::ShellHost;
use crate::generated::MdClock;

/// Texto que muestra el reloj en un instante dado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClockText {
    /// Hora (`HH:MM`).
    pub time: String,
    /// Fecha (`"dom, 15 nov"`).
    pub date: String,
}

/// Configuración del reloj.
pub struct ClockConfig {
    /// Identificador de ventana (app-id) para las windowrules de Hyprland.
    pub app_id: String,
}

/// Intervalo del temporizador que detecta el cambio de minuto.
const TICK: Duration = Duration::from_secs(1);

/// Ejecuta el reloj hasta que se cierra la ventana. `now` devuelve el texto del
/// instante actual; se llama una vez al arrancar y luego solo al cambiar el minuto.
pub fn run(
    host: &impl ShellHost,
    config: &ClockConfig,
    mut now: impl FnMut() -> ClockText + 'static,
) -> Result<(), slint::PlatformError> {
    let component = MdClock::new()?;
    // `MdClock::new()` inicializa el backend: el app-id ya se puede fijar antes
    // de mostrar la ventana (lo necesitan las windowrules de Hyprland).
    slint::set_xdg_app_id(config.app_id.clone())?;

    let text = now();
    component.set_clock_text(text.time.into());
    component.set_date_text(text.date.into());

    let weak = component.as_weak();
    let mut last_minute = current_minute();
    let timer = slint::Timer::default();
    timer.start(slint::TimerMode::Repeated, TICK, move || {
        let minute = current_minute();
        if minute == last_minute {
            return;
        }
        last_minute = minute;
        let text = now();
        if let Some(component) = weak.upgrade() {
            component.set_clock_text(text.time.into());
            component.set_date_text(text.date.into());
        }
    });

    host.run(component)
}

/// Minuto actual desde la época (solo para detectar el cambio de minuto).
fn current_minute() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() / 60)
        .unwrap_or(0)
}
