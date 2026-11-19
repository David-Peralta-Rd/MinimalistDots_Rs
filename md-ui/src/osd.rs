//! Puerto de alto nivel para el widget `MdOsd`: una tarjeta transitoria que se
//! muestra unos instantes y se cierra sola (volumen, brillo…).

use std::time::Duration;

use slint::ComponentHandle;

use crate::generated::MdOsd;
use crate::{ShellHost, slint_color};

/// Configuración de una tarjeta OSD.
#[derive(Debug, Clone)]
pub struct OsdConfig {
    /// Identificador de ventana (app-id) para las windowrules de Hyprland.
    pub app_id: String,
    /// Glifo opcional (nerd-font); cadena vacía = sin icono.
    pub glyph: String,
    /// Texto principal ("70%", "Mute"…).
    pub label: String,
    /// Nivel 0–100 para la barra; `-1` la oculta.
    pub level: i32,
    /// Marca el borde y el texto en rojo (mute).
    pub muted: bool,
    pub accent: md_core::Color,
    pub width: f32,
    pub height: f32,
    /// Tiempo visible antes de cerrarse sola.
    pub lifetime: Duration,
}

impl Default for OsdConfig {
    fn default() -> Self {
        Self {
            app_id: "md.osd".to_owned(),
            glyph: String::new(),
            label: String::new(),
            level: -1,
            muted: false,
            accent: md_core::Color::new(0x7e, 0xc7, 0xa2),
            width: 80.0,
            height: 40.0,
            lifetime: Duration::from_millis(800),
        }
    }
}

/// Muestra la tarjeta OSD y la cierra sola al cumplirse `lifetime`.
pub fn run(host: &impl ShellHost, config: &OsdConfig) -> Result<(), slint::PlatformError> {
    let component = MdOsd::new()?;
    // `MdOsd::new()` inicializa el backend: el app-id ya se puede fijar antes
    // de mostrar la ventana (lo necesitan las windowrules de Hyprland).
    slint::set_xdg_app_id(config.app_id.clone())?;
    component.set_glyph(config.glyph.clone().into());
    component.set_label(config.label.clone().into());
    component.set_level(config.level);
    component.set_muted(config.muted);
    component.set_accent(slint_color(config.accent));
    component.set_md_width(config.width);
    component.set_md_height(config.height);

    // Cierre automático: un temporizador oculta la ventana al cumplirse el
    // tiempo. El `Timer` debe vivir mientras corre el bucle de eventos, por
    // eso se declara aquí y no se suelta hasta que `run` retorna.
    let weak = component.as_weak();
    let timer = slint::Timer::default();
    timer.start(slint::TimerMode::SingleShot, config.lifetime, move || {
        if let Some(component) = weak.upgrade() {
            let _ = component.hide();
        }
        // Cierra el bucle de eventos para que el proceso termine a tiempo
        // (evita ventanas residuales al pulsar volumen rápidamente).
        let _ = slint::quit_event_loop();
    });

    host.run(component)
}
