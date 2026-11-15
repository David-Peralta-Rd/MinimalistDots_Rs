//! Color RGB de la paleta, con conversiones a los formatos que consumen
//! Hyprland (`rgba(RRGGBBaa)`) y Slint (`#AARRGGBB`).

use serde::Deserialize;

/// Un color RGB (sin alfa), representado por sus tres canales.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    /// Construye un color a partir de sus canales.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Interpreta `#RRGGBB` (o `RRGGBB`) como color.
    pub fn from_hex(input: &str) -> Result<Self, ColorError> {
        let hex = input.strip_prefix('#').unwrap_or(input);
        if hex.len() != 6 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ColorError::invalid(input));
        }
        // Tras validar que son 6 dígitos hexadecimales ASCII, la conversión no falla.
        let value = u32::from_str_radix(hex, 16).map_err(|_| ColorError::invalid(input))?;
        Ok(Self {
            r: (value >> 16) as u8,
            g: (value >> 8) as u8,
            b: value as u8,
        })
    }

    pub const fn r(self) -> u8 {
        self.r
    }

    pub const fn g(self) -> u8 {
        self.g
    }

    pub const fn b(self) -> u8 {
        self.b
    }

    /// Devuelve `#RRGGBB`.
    pub fn hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Devuelve `rgba(RRGGBBaa)` para Hyprland (el alfa va al final, en hex).
    pub fn to_hypr(self, alpha: u8) -> String {
        format!(
            "rgba({:02x}{:02x}{:02x}{:02x})",
            self.r, self.g, self.b, alpha
        )
    }

    /// Devuelve `#AARRGGBB` para Slint/QML (el alfa va primero, en hex).
    pub fn to_slint(self, alpha: u8) -> String {
        format!("#{:02x}{:02x}{:02x}{:02x}", alpha, self.r, self.g, self.b)
    }
}

/// Error al interpretar un color hexadecimal.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("color inválido {0:?}: se esperaba el formato #RRGGBB")]
pub struct ColorError(String);

impl ColorError {
    fn invalid(input: &str) -> Self {
        Self(input.to_owned())
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        Color::from_hex(&text).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_with_and_without_hash() -> Result<(), ColorError> {
        assert_eq!(Color::from_hex("#1e222a")?, Color::new(0x1e, 0x22, 0x2a));
        assert_eq!(Color::from_hex("1e222a")?, Color::new(0x1e, 0x22, 0x2a));
        Ok(())
    }

    #[test]
    fn rejects_invalid_input() {
        assert!(Color::from_hex("#12345").is_err());
        assert!(Color::from_hex("#12345g").is_err());
        assert!(Color::from_hex("").is_err());
    }

    #[test]
    fn formats_for_each_backend() -> Result<(), ColorError> {
        let color = Color::from_hex("#1e222a")?;
        assert_eq!(color.hex(), "#1e222a");
        assert_eq!(color.to_hypr(0xee), "rgba(1e222aee)");
        assert_eq!(color.to_slint(0xff), "#ff1e222a");
        Ok(())
    }
}
