//! Estado del volumen y utilidades puras (sin ejecutar `wpctl`).

/// Nivel de volumen (0–100) y si está silenciado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VolumeState {
    pub muted: bool,
    pub level: u8,
}

impl VolumeState {
    /// Parsea la salida de `wpctl get-volume @DEFAULT_AUDIO_SINK@`.
    ///
    /// Formato esperado: `Volume: 0.70` o `Volume: 0.70 [MUTED]`. El nivel se
    /// redondea a 0–100; `None` si no se reconoce la salida.
    pub fn parse_wpctl(output: &str) -> Option<Self> {
        let muted = output.contains("[MUTED]");
        let level = output
            .split_whitespace()
            .find_map(|token| token.parse::<f32>().ok())
            .map(|value| ((value * 100.0).round() as u8).min(100))?;
        Some(Self { muted, level })
    }

    /// Icono (nerd-font) que representa el estado.
    pub fn icon(&self) -> &'static str {
        if self.muted {
            "󰝟"
        } else if self.level >= 70 {
            "󰕾"
        } else if self.level >= 30 {
            "󰖀"
        } else {
            "󰕿"
        }
    }

    /// Etiqueta legible ("Mute" o "70%"). La clave de idioma se cablea en H7.
    pub fn label(&self) -> String {
        if self.muted {
            "Mute".to_owned()
        } else {
            format!("{}%", self.level)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(output: &str) -> Result<VolumeState, &'static str> {
        VolumeState::parse_wpctl(output).ok_or("no se pudo parsear")
    }

    #[test]
    fn parses_plain_volume() -> Result<(), &'static str> {
        let state = parse("Volume: 0.70")?;
        assert!(!state.muted);
        assert_eq!(state.level, 70);
        Ok(())
    }

    #[test]
    fn parses_muted_volume() -> Result<(), &'static str> {
        let state = parse("Volume: 0.35 [MUTED]")?;
        assert!(state.muted);
        assert_eq!(state.level, 35);
        Ok(())
    }

    #[test]
    fn clamps_to_100() -> Result<(), &'static str> {
        let state = parse("Volume: 1.20")?;
        assert_eq!(state.level, 100);
        Ok(())
    }

    #[test]
    fn rejects_unrecognized_output() {
        assert_eq!(VolumeState::parse_wpctl("sin volumen"), None);
    }

    #[test]
    fn chooses_icon_by_level() {
        assert_eq!(
            VolumeState {
                muted: true,
                level: 0
            }
            .icon(),
            "󰝟"
        );
        assert_eq!(
            VolumeState {
                muted: false,
                level: 70
            }
            .icon(),
            "󰕾"
        );
        assert_eq!(
            VolumeState {
                muted: false,
                level: 30
            }
            .icon(),
            "󰖀"
        );
        assert_eq!(
            VolumeState {
                muted: false,
                level: 29
            }
            .icon(),
            "󰕿"
        );
    }

    #[test]
    fn builds_label() {
        assert_eq!(
            VolumeState {
                muted: true,
                level: 42
            }
            .label(),
            "Mute"
        );
        assert_eq!(
            VolumeState {
                muted: false,
                level: 42
            }
            .label(),
            "42%"
        );
    }
}
