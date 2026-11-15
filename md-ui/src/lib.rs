//! Interfaz Slint de MinimalistDots: los widgets y el puerto de ventanas.
//!
//! Los `.slint` se compilan en tiempo de compilación (`build.rs`) y sus
//! componentes quedan disponibles vía `slint::include_modules!()`.

// El código generado por Slint usa `unwrap`/`panic` internamente y viene
// protegido para sus propios lints; lo aislamos aquí para que los `deny` del
// workspace sigan aplicando a nuestro código.
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod generated {
    slint::include_modules!();
}

mod menu;
mod notifications;
mod osd;
mod pomodoro;

pub use generated::*;
pub use menu::{MenuConfig, MenuEvent, run as run_menu};
pub use notifications::{NotificationData, Notifications};
pub use osd::{OsdConfig, run as run_osd};
pub use pomodoro::{PomodoroConfig, PomodoroEvents, run as run_pomodoro};

/// Puerto por el que los widgets muestran sus ventanas.
///
/// La v1 usa ventanas Slint normales (xdg-toplevel, ADR-0003). Cambiar el
/// anfitrión (por ejemplo, a layer-shell) no debería tocar los archivos `.slint`.
pub trait ShellHost {
    /// Ejecuta el bucle de eventos del componente hasta que su ventana se cierra.
    fn run<C: slint::ComponentHandle>(&self, component: C) -> Result<(), slint::PlatformError>;
}

/// Anfitrión por defecto: ventanas Slint (xdg-toplevel).
#[derive(Debug, Default, Clone, Copy)]
pub struct SlintHost;

impl ShellHost for SlintHost {
    fn run<C: slint::ComponentHandle>(&self, component: C) -> Result<(), slint::PlatformError> {
        component.run()
    }
}

/// Convierte un color de `md-core` al tipo `slint::Color`.
pub fn slint_color(color: md_core::Color) -> slint::Color {
    slint::Color::from_rgb_u8(color.r(), color.g(), color.b())
}

/// Muestra el diálogo "Acerca de" (requisito de la licencia Royalty-free de Slint).
pub fn about(host: &impl ShellHost) -> Result<(), slint::PlatformError> {
    let window = AboutWindow::new()?;
    host.run(window)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_core_color_to_slint() {
        let color = slint_color(md_core::Color::new(0x1e, 0x22, 0x2a));
        assert_eq!(color.red(), 0x1e);
        assert_eq!(color.green(), 0x22);
        assert_eq!(color.blue(), 0x2a);
        assert_eq!(color.alpha(), 0xff);
    }
}
