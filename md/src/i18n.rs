//! Idioma en runtime (ADR-0004): `md` lee `settings.toml` al arrancar y carga el
//! catálogo correspondiente. Sin proceso residente: cada invocación vuelve a leer
//! y el cambio de idioma vale para el siguiente arranque.

use std::path::PathBuf;

use md_core::Translator;

/// Textos traducidos de la invocación actual.
pub struct Texts {
    translator: Translator,
}

impl Texts {
    /// Carga el idioma guardado y su cadena de catálogos. Si falta `settings.toml`
    /// o los catálogos, devuelve un traductor vacío (las claves quedan tal cual,
    /// igual que el comportamiento de `load_chain` sin archivos).
    pub fn load() -> Self {
        let codigo = read_language().unwrap_or_else(|| "es".to_owned());
        let translator = md_core::load_chain(&lang_dir(), &codigo).unwrap_or_default();
        Self { translator }
    }

    /// Texto de `clave` (`seccion.clave`) o la propia clave si no hay traducción.
    pub fn t(&self, key: &str) -> String {
        self.translator.t(key)
    }

    /// Texto de `clave` con sus marcadores `{nombre}` sustituidos por `values`.
    pub fn t_fill(&self, key: &str, values: &[(&str, &str)]) -> String {
        md_core::fill(&self.translator.t(key), values)
    }
}

/// Directorio de configuración: `$XDG_CONFIG_HOME` o `~/.config`.
fn config_home() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("."))
}

/// Raíz de MinimalistDots (`~/.config/minimalistdots`), salvo override por entorno
/// (`MINIMALISTDOTS_CONFIG`, útil en pruebas).
fn minimalistdots_dir() -> PathBuf {
    std::env::var_os("MINIMALISTDOTS_CONFIG")
        .map(PathBuf::from)
        .unwrap_or_else(|| config_home().join("minimalistdots"))
}

/// Directorio de catálogos (`~/.config/minimalistdots/lang`).
fn lang_dir() -> PathBuf {
    minimalistdots_dir().join("lang")
}

/// Archivo de ajustes (`~/.config/minimalistdots/settings.toml`).
fn settings_path() -> PathBuf {
    minimalistdots_dir().join("settings.toml")
}

/// Idioma guardado en `settings.toml`, o `None` si no existe o es inválido.
fn read_language() -> Option<String> {
    let text = std::fs::read_to_string(settings_path()).ok()?;
    md_core::read_settings_language(&text)
}
