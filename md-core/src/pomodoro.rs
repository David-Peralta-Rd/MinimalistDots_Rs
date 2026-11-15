//! Máquina de estados del temporizador Pomodoro (lógica pura, sin UI ni tiempo
//! real: el avance de segundos lo decide quien llama a [`Pomodoro::tick`]).
//!
//! Reemplaza la lógica del antiguo `pomodoro.qml`/`pomodoro.sh`. El temporizador
//! vive en la vista (Slint + `slint::Timer` en `md`); aquí solo están las fases,
//! la cuenta atrás y las transiciones, para poder probarlas sin ventana.

/// Fase actual del temporizador.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// Concentración.
    Focus,
    /// Descanso.
    Break,
}

/// Qué pasó al avanzar un segundo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TickEvent {
    /// Sigue contando; no hay transición.
    Ticking,
    /// Terminó la concentración y empezó el descanso.
    BreakStarted { session: u32, total: u32 },
    /// Terminó el descanso y empieza la siguiente concentración.
    FocusStarted { session: u32, total: u32 },
    /// Se completaron todas las sesiones.
    Finished { total: u32 },
}

/// Error de construcción: los tiempos y sesiones deben ser positivos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("los tiempos y sesiones deben ser mayores que cero")]
pub struct PomodoroError;

/// Temporizador Pomodoro: alterna concentración y descanso durante `total`
/// sesiones. Empieza en concentración, sesión 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pomodoro {
    focus_secs: u32,
    break_secs: u32,
    total: u32,
    session: u32,
    phase: Phase,
    remaining: u32,
    paused: bool,
    collapsed: bool,
    finished: bool,
}

impl Pomodoro {
    /// Crea un temporizador nuevo. `focus_secs`, `break_secs` y `total` deben
    /// ser mayores que cero.
    pub fn new(focus_secs: u32, break_secs: u32, total: u32) -> Result<Self, PomodoroError> {
        if focus_secs == 0 || break_secs == 0 || total == 0 {
            return Err(PomodoroError);
        }
        Ok(Self {
            focus_secs,
            break_secs,
            total,
            session: 1,
            phase: Phase::Focus,
            remaining: focus_secs,
            paused: false,
            collapsed: false,
            finished: false,
        })
    }

    pub const fn phase(&self) -> Phase {
        self.phase
    }

    pub const fn session(&self) -> u32 {
        self.session
    }

    pub const fn total(&self) -> u32 {
        self.total
    }

    pub const fn remaining(&self) -> u32 {
        self.remaining
    }

    pub const fn paused(&self) -> bool {
        self.paused
    }

    pub const fn collapsed(&self) -> bool {
        self.collapsed
    }

    pub const fn is_finished(&self) -> bool {
        self.finished
    }

    /// El descanso se muestra como tarjeta grande salvo que se minimice.
    pub const fn is_big_break(&self) -> bool {
        !self.finished && !self.collapsed && matches!(self.phase, Phase::Break)
    }

    /// Segundos que dura la fase actual.
    const fn phase_len(&self) -> u32 {
        match self.phase {
            Phase::Focus => self.focus_secs,
            Phase::Break => self.break_secs,
        }
    }

    /// Cuenta atrás en formato `MM:SS`.
    pub fn clock_text(&self) -> String {
        format!("{:02}:{:02}", self.remaining / 60, self.remaining % 60)
    }

    /// Progreso de la fase actual en porcentaje entero (0–100).
    pub fn pct(&self) -> u32 {
        let len = self.phase_len();
        // `len` es siempre > 0 (validado en `new`); se calcula en `u64` para no
        // desbordar `remaining * 100` con valores extremos.
        ((u64::from(self.remaining) * 100) / u64::from(len)) as u32
    }

    /// Avanza un segundo. Devuelve lo que ocurrió (transición o nada).
    pub fn tick(&mut self) -> TickEvent {
        if self.paused || self.finished || self.remaining == 0 {
            return TickEvent::Ticking;
        }
        self.remaining -= 1;
        if self.remaining == 0 {
            self.advance()
        } else {
            TickEvent::Ticking
        }
    }

    /// Transición al terminar una fase (concentración → descanso → siguiente
    /// concentración → fin), como en el `advance()` del QML original.
    fn advance(&mut self) -> TickEvent {
        match self.phase {
            Phase::Focus => {
                self.phase = Phase::Break;
                self.collapsed = false;
                self.remaining = self.break_secs;
                TickEvent::BreakStarted {
                    session: self.session,
                    total: self.total,
                }
            }
            Phase::Break if self.session >= self.total => {
                self.finished = true;
                self.remaining = 0;
                TickEvent::Finished { total: self.total }
            }
            Phase::Break => {
                self.session += 1;
                self.phase = Phase::Focus;
                self.remaining = self.focus_secs;
                TickEvent::FocusStarted {
                    session: self.session,
                    total: self.total,
                }
            }
        }
    }

    /// Pausa o reanuda.
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Reinicia la fase actual desde cero (y des-minimiza el descanso).
    pub fn restart_phase(&mut self) {
        self.paused = false;
        self.collapsed = false;
        self.remaining = self.phase_len();
    }

    /// Minimiza la tarjeta de descanso al widget pequeño.
    pub fn collapse(&mut self) {
        self.collapsed = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_values() {
        assert_eq!(Pomodoro::new(0, 5, 4), Err(PomodoroError));
        assert_eq!(Pomodoro::new(25, 0, 4), Err(PomodoroError));
        assert_eq!(Pomodoro::new(25, 5, 0), Err(PomodoroError));
    }

    #[test]
    fn starts_in_focus_session_one() -> Result<(), PomodoroError> {
        let pomodoro = Pomodoro::new(25, 5, 4)?;
        assert_eq!(pomodoro.phase(), Phase::Focus);
        assert_eq!(pomodoro.session(), 1);
        assert_eq!(pomodoro.remaining(), 25);
        assert!(!pomodoro.paused());
        assert!(!pomodoro.is_big_break());
        Ok(())
    }

    #[test]
    fn counts_down() -> Result<(), PomodoroError> {
        let mut pomodoro = Pomodoro::new(25, 5, 4)?;
        assert_eq!(pomodoro.tick(), TickEvent::Ticking);
        assert_eq!(pomodoro.remaining(), 24);
        assert_eq!(pomodoro.clock_text(), "00:24");
        Ok(())
    }

    #[test]
    fn focus_end_starts_break() -> Result<(), PomodoroError> {
        let mut pomodoro = Pomodoro::new(2, 3, 4)?;
        assert_eq!(pomodoro.tick(), TickEvent::Ticking); // 1
        assert_eq!(
            pomodoro.tick(),
            TickEvent::BreakStarted {
                session: 1,
                total: 4
            }
        );
        assert_eq!(pomodoro.phase(), Phase::Break);
        assert_eq!(pomodoro.remaining(), 3);
        assert!(pomodoro.is_big_break());
        Ok(())
    }

    #[test]
    fn break_end_starts_next_focus() -> Result<(), PomodoroError> {
        let mut pomodoro = Pomodoro::new(2, 1, 4)?;
        pomodoro.tick(); // 1
        pomodoro.tick(); // → break (remaining 1)
        assert_eq!(
            pomodoro.tick(),
            TickEvent::FocusStarted {
                session: 2,
                total: 4
            }
        );
        assert_eq!(pomodoro.phase(), Phase::Focus);
        assert_eq!(pomodoro.session(), 2);
        assert_eq!(pomodoro.remaining(), 2);
        Ok(())
    }

    #[test]
    fn last_break_finishes() -> Result<(), PomodoroError> {
        // 1 sesión: concentración de 1 s y descanso de 1 s.
        let mut pomodoro = Pomodoro::new(1, 1, 1)?;
        assert_eq!(
            pomodoro.tick(),
            TickEvent::BreakStarted {
                session: 1,
                total: 1
            }
        );
        assert_eq!(pomodoro.tick(), TickEvent::Finished { total: 1 });
        assert!(pomodoro.is_finished());
        // Tras terminar, `tick` no avanza más.
        assert_eq!(pomodoro.tick(), TickEvent::Ticking);
        Ok(())
    }

    #[test]
    fn pause_blocks_tick() -> Result<(), PomodoroError> {
        let mut pomodoro = Pomodoro::new(25, 5, 4)?;
        pomodoro.toggle_pause();
        assert!(pomodoro.paused());
        assert_eq!(pomodoro.tick(), TickEvent::Ticking);
        assert_eq!(pomodoro.remaining(), 25);
        pomodoro.toggle_pause();
        assert!(!pomodoro.paused());
        assert_eq!(pomodoro.tick(), TickEvent::Ticking);
        assert_eq!(pomodoro.remaining(), 24);
        Ok(())
    }

    #[test]
    fn restart_resets_phase() -> Result<(), PomodoroError> {
        let mut pomodoro = Pomodoro::new(25, 5, 4)?;
        pomodoro.tick(); // 24
        pomodoro.toggle_pause();
        pomodoro.restart_phase();
        assert_eq!(pomodoro.remaining(), 25);
        assert!(!pomodoro.paused());
        assert!(!pomodoro.collapsed());
        Ok(())
    }

    #[test]
    fn collapse_minimizes_break() -> Result<(), PomodoroError> {
        let mut pomodoro = Pomodoro::new(1, 5, 1)?;
        pomodoro.tick(); // → break
        assert!(pomodoro.is_big_break());
        pomodoro.collapse();
        assert!(!pomodoro.is_big_break());
        assert!(pomodoro.collapsed());
        Ok(())
    }

    #[test]
    fn percentage_is_full_at_start() -> Result<(), PomodoroError> {
        let pomodoro = Pomodoro::new(25, 5, 4)?;
        assert_eq!(pomodoro.pct(), 100);
        Ok(())
    }
}
