//! Paleta de colores de MinimalistDots, cargada desde `theme/palette.toml`.
//!
//! Las claves del TOML están en español (datos); los campos de Rust, en inglés
//! (identificadores), tal y como documenta el ADR-0004.

use std::fs;
use std::path::Path;

use serde::Deserialize;

use crate::color::Color;

/// La única paleta del proyecto. Se carga desde `theme/palette.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Palette {
    #[serde(rename = "fondo")]
    pub background: Color,
    #[serde(rename = "superficie")]
    pub surface: Color,
    #[serde(rename = "seleccion")]
    pub selection: Color,
    #[serde(rename = "borde_inactivo")]
    pub border_inactive: Color,
    #[serde(rename = "borde_activo")]
    pub border_active: Color,
    #[serde(rename = "texto")]
    pub text: Color,
    #[serde(rename = "texto_suave")]
    pub subtext: Color,
    #[serde(rename = "acento_azul")]
    pub accent_blue: Color,
    #[serde(rename = "acento_verde")]
    pub accent_green: Color,
    #[serde(rename = "acento_rojo")]
    pub accent_red: Color,
    #[serde(rename = "acento_amarillo")]
    pub accent_yellow: Color,
    #[serde(rename = "acento_magenta")]
    pub accent_magenta: Color,
    #[serde(rename = "acento_cian")]
    pub accent_cyan: Color,
}

impl std::str::FromStr for Palette {
    type Err = PaletteError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        toml::from_str(text).map_err(PaletteError::from)
    }
}

impl Palette {
    /// Carga la paleta desde un archivo.
    pub fn load(path: &Path) -> Result<Self, PaletteError> {
        let text = fs::read_to_string(path)?;
        text.parse()
    }
}

/// Error al cargar o interpretar la paleta.
#[derive(Debug, thiserror::Error)]
pub enum PaletteError {
    #[error("no se pudo leer la paleta: {0}")]
    Io(#[from] std::io::Error),
    #[error("paleta inválida: {0}")]
    Toml(#[from] toml::de::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    const MINIMAL: &str = "\
fondo = \"#1e222a\"
superficie = \"#282c34\"
seleccion = \"#3e4451\"
borde_inactivo = \"#4b5263\"
borde_activo = \"#7ec7a2\"
texto = \"#abb2bf\"
texto_suave = \"#5c6370\"
acento_azul = \"#7aa2f7\"
acento_verde = \"#7ec7a2\"
acento_rojo = \"#e06c75\"
acento_amarillo = \"#d19a66\"
acento_magenta = \"#c678dd\"
acento_cian = \"#56b6c2\"
";

    #[test]
    fn parses_every_key() -> Result<(), PaletteError> {
        let palette: Palette = MINIMAL.parse()?;
        assert_eq!(palette.background.hex(), "#1e222a");
        assert_eq!(palette.surface.hex(), "#282c34");
        assert_eq!(palette.selection.hex(), "#3e4451");
        assert_eq!(palette.border_inactive.hex(), "#4b5263");
        assert_eq!(palette.border_active.hex(), "#7ec7a2");
        assert_eq!(palette.text.hex(), "#abb2bf");
        assert_eq!(palette.subtext.hex(), "#5c6370");
        assert_eq!(palette.accent_blue.hex(), "#7aa2f7");
        assert_eq!(palette.accent_green.hex(), "#7ec7a2");
        assert_eq!(palette.accent_red.hex(), "#e06c75");
        assert_eq!(palette.accent_yellow.hex(), "#d19a66");
        assert_eq!(palette.accent_magenta.hex(), "#c678dd");
        assert_eq!(palette.accent_cyan.hex(), "#56b6c2");
        Ok(())
    }

    #[test]
    fn rejects_unknown_keys() {
        let text = format!("{MINIMAL}extra = \"#000000\"\n");
        assert!(text.parse::<Palette>().is_err());
    }

    #[test]
    fn loads_from_file() -> Result<(), PaletteError> {
        let dir = crate::test_util::TempDir::new("palette")?;
        dir.write("palette.toml", MINIMAL)?;
        let palette = Palette::load(&dir.path().join("palette.toml"))?;
        assert_eq!(palette.text.hex(), "#abb2bf");
        Ok(())
    }
}
