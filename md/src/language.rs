//! Subcomando `language`: elige el idioma de los menús y avisos de `md`.

use std::fs;

use md_core::MenuEntry;

use crate::i18n::{Texts, lang_dir, settings_path};
use crate::menu::{MenuResult, run_entries};

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let texts = Texts::load();
    let dir = lang_dir();
    let languages = md_core::available_languages(&dir).unwrap_or_default();
    if languages.is_empty() {
        println!("No hay catálogos instalados en {}", dir.display());
        return Ok(());
    }

    let labels: Vec<String> = languages
        .iter()
        .map(|(code, name)| format!("{name} ({code})"))
        .collect();
    let entries: Vec<MenuEntry> = labels
        .iter()
        .map(|label| MenuEntry::new(label.clone()))
        .collect();
    let _ = texts;

    let choice = match run_entries(&entries, "🌐", false, false, false, None, None, None, None) {
        Ok(MenuResult::Selected(index)) => languages.get(index).map(|(code, _)| code.clone()),
        _ => None,
    };

    if let Some(code) = choice {
        let path = settings_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&path, format!("idioma = \"{}\"\n", code));
        println!(
            "Idioma cambiado a {}. Vuelve a abrir los menús para aplicarlo.",
            code
        );
    }
    Ok(())
}
