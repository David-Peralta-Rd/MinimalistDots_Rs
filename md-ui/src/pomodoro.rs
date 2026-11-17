//! Puerto de alto nivel para el widget `MdPomodoro`: configura la ventana, avanza
//! las fases con un temporizador de 1 s y conecta los botones.
//!
//! La máquina de estados vive en `md_core::Pomodoro`; aquí solo se mantiene en un
//! `RefCell` y se proyecta a las propiedades del widget. Los efectos secundarios
//! (sonido y notificaciones) los dispara el anfitrión `md` vía [`PomodoroEvents`].

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use md_core::{Phase, Pomodoro, TickEvent};
use slint::ComponentHandle;

use crate::generated::MdPomodoro;
use crate::{ShellHost, slint_color};

/// Tamaño de la tarjeta pequeña (concentración o descanso minimizado).
const SMALL_WIDTH: f32 = 190.0;
const SMALL_HEIGHT: f32 = 96.0;
/// Tamaño de la tarjeta grande (descanso).
const BIG_WIDTH: f32 = 500.0;
const BIG_HEIGHT: f32 = 320.0;
/// Intervalo de la cuenta atrás.
const TICK: Duration = Duration::from_secs(1);

/// Colores según la fase (capturados por valor en los callbacks).
#[derive(Clone, Copy)]
struct Accents {
    focus: md_core::Color,
    brk: md_core::Color,
    paused: md_core::Color,
}

/// Configuración de una instancia del temporizador.
pub struct PomodoroConfig {
    /// Identificador de ventana (app-id) para las windowrules de Hyprland.
    pub app_id: String,
    /// Acento en concentración.
    pub accent_focus: md_core::Color,
    /// Acento en descanso.
    pub accent_break: md_core::Color,
    /// Acento en pausa.
    pub accent_paused: md_core::Color,
}

/// Efectos secundarios en cada transición (los aporta `md`).
pub struct PomodoroEvents {
    /// Se llama al arrancar, con la sesión y el total iniciales.
    pub on_started: Box<dyn Fn(u32, u32)>,
    /// Concentración → descanso.
    pub on_break_started: Box<dyn Fn(u32, u32)>,
    /// Descanso → siguiente concentración.
    pub on_focus_started: Box<dyn Fn(u32, u32)>,
    /// Todas las sesiones terminadas.
    pub on_finished: Box<dyn Fn(u32)>,
}

/// Ejecuta el temporizador y devuelve cuando la ventana se cierra.
pub fn run(
    host: &impl ShellHost,
    config: &PomodoroConfig,
    state: Pomodoro,
    events: PomodoroEvents,
) -> Result<(), slint::PlatformError> {
    let accents = Accents {
        focus: config.accent_focus,
        brk: config.accent_break,
        paused: config.accent_paused,
    };

    let small = MdPomodoro::new()?;
    slint::set_xdg_app_id(config.app_id.clone())?;
    small.set_md_title("md.pomodoro".into());
    let state = Rc::new(RefCell::new(state));
    let weak_small = small.as_weak();
    let big_cell: Rc<RefCell<Option<MdPomodoro>>> = Rc::new(RefCell::new(None));

    apply(&small, &state.borrow(), accents);

    // Notificación inicial ("Pomodoro iniciado / Sesión 1 de N").
    let (session, total) = {
        let state = state.borrow();
        (state.session(), state.total())
    };
    (events.on_started)(session, total);

    // ----- botones (tarjeta de trabajo) -----
    small.on_pause_requested({
        let state = state.clone();
        let weak_small = weak_small.clone();
        let big_cell = big_cell.clone();
        move || {
            state.borrow_mut().toggle_pause();
            let big = big_cell.borrow();
            let weak = if state.borrow().is_big_break() {
                big.as_ref().map(|c| c.as_weak())
            } else {
                Some(weak_small.clone())
            };
            if let Some(weak) = weak
                && let Some(component) = weak.upgrade()
            {
                apply(&component, &state.borrow(), accents);
            }
        }
    });
    small.on_restart_requested({
        let state = state.clone();
        let weak_small = weak_small.clone();
        let big_cell = big_cell.clone();
        move || {
            state.borrow_mut().restart_phase();
            let big = big_cell.borrow();
            let weak = if state.borrow().is_big_break() {
                big.as_ref().map(|c| c.as_weak())
            } else {
                Some(weak_small.clone())
            };
            if let Some(weak) = weak
                && let Some(component) = weak.upgrade()
            {
                apply(&component, &state.borrow(), accents);
            }
        }
    });
    small.on_collapse_requested({
        let state = state.clone();
        let weak_small = weak_small.clone();
        move || {
            state.borrow_mut().collapse();
            if let Some(component) = weak_small.upgrade() {
                apply(&component, &state.borrow(), accents);
            }
        }
    });
    small.on_stop_requested({
        let weak_small = weak_small.clone();
        let big_cell = big_cell.clone();
        move || {
            if let Some(component) = weak_small.upgrade() {
                let _ = component.hide();
            }
            if let Some(big) = big_cell.borrow().as_ref() {
                let _ = big.hide();
            }
        }
    });

    // ----- cuenta atrás -----
    let timer = slint::Timer::default();
    timer.start(slint::TimerMode::Repeated, TICK, {
        let state = state.clone();
        let weak_small = weak_small.clone();
        let big_cell = big_cell.clone();
        move || {
            {
                let state = state.borrow();
                if state.paused() || state.is_finished() {
                    return;
                }
            }
            let event = state.borrow_mut().tick();
            let now_big = state.borrow().is_big_break();
            let weak = if now_big {
                // Crea la tarjeta grande la primera vez que se necesita.
                let mut big_opt = big_cell.borrow_mut();
                if big_opt.is_none() {
                    if let Ok(big) = MdPomodoro::new() {
                        big.set_md_title("md.pomodoro.break".into());
                        // Conecta botones mínimos en la tarjeta grande.
                        big.on_pause_requested({
                            let state = state.clone();
                            let big_cell = big_cell.clone();
                            move || {
                                state.borrow_mut().toggle_pause();
                                if let Some(big) = big_cell.borrow().as_ref()
                                    && let Some(component) = big.as_weak().upgrade()
                                {
                                    apply(&component, &state.borrow(), accents);
                                }
                            }
                        });
                        big.on_restart_requested({
                            let state = state.clone();
                            let big_cell = big_cell.clone();
                            move || {
                                state.borrow_mut().restart_phase();
                                if let Some(big) = big_cell.borrow().as_ref()
                                    && let Some(component) = big.as_weak().upgrade()
                                {
                                    apply(&component, &state.borrow(), accents);
                                }
                            }
                        });
                        big.on_stop_requested({
                            let weak_small = weak_small.clone();
                            let big_cell = big_cell.clone();
                            move || {
                                if let Some(s) = weak_small.upgrade() {
                                    let _ = s.hide();
                                }
                                if let Some(big) = big_cell.borrow().as_ref() {
                                    let _ = big.hide();
                                }
                            }
                        });
                        let _ = big.show();
                        *big_opt = Some(big);
                    }
                } else if let Some(big) = big_opt.as_ref() {
                    let _ = big.show();
                }
                big_opt.as_ref().map(|b| b.as_weak())
            } else {
                if let Some(big) = big_cell.borrow().as_ref() {
                    let _ = big.hide();
                }
                Some(weak_small.clone())
            };
            if let Some(weak) = weak
                && let Some(component) = weak.upgrade()
            {
                apply(&component, &state.borrow(), accents);
            }
            match event {
                TickEvent::Ticking => {}
                TickEvent::BreakStarted { session, total } => {
                    (events.on_break_started)(session, total);
                }
                TickEvent::FocusStarted { session, total } => {
                    (events.on_focus_started)(session, total);
                }
                TickEvent::Finished { total } => {
                    (events.on_finished)(total);
                    if let Some(component) = weak_small.upgrade() {
                        let _ = component.hide();
                    }
                    if let Some(big) = big_cell.borrow().as_ref() {
                        let _ = big.hide();
                    }
                }
            }
        }
    });

    host.run(small)
}

/// Proyecta el estado de la máquina a las propiedades del widget.
fn apply(component: &MdPomodoro, state: &Pomodoro, accents: Accents) {
    component.set_clock_text(state.clock_text().into());
    component.set_session_text(format!("{}/{}", state.session(), state.total()).into());
    component.set_subtitle(subtitle(state).into());
    component.set_paused(state.paused());
    component.set_big_break(state.is_big_break());
    component.set_pct(state.pct() as i32);
    component.set_accent(slint_color(accent_for(state, accents)));

    let big = state.is_big_break();
    component.set_md_width(if big { BIG_WIDTH } else { SMALL_WIDTH });
    component.set_md_height(if big { BIG_HEIGHT } else { SMALL_HEIGHT });
}

/// Acento según el estado (gris en pausa; rojo en concentración, verde en descanso).
fn accent_for(state: &Pomodoro, accents: Accents) -> md_core::Color {
    if state.paused() {
        accents.paused
    } else {
        match state.phase() {
            Phase::Focus => accents.focus,
            Phase::Break => accents.brk,
        }
    }
}

/// Texto de la tarjeta de descanso ("Sesión X de Y completada · último descanso").
/// Es texto interno del widget (no localizado, igual que las etiquetas del .slint).
fn subtitle(state: &Pomodoro) -> String {
    let mut text = format!("Sesión {} de {} completada", state.session(), state.total());
    if state.session() >= state.total() {
        text.push_str(" · último descanso");
    }
    text
}
