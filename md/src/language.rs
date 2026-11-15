//! Subcomando `language`: elige el idioma de los menús y avisos de `md`.

use std::fs;

use md_core::{MenuEntry, MenuMode};
use md_ui::{MenuConfig, MenuEvent, SlintHost, run_menu};

use crate::i18n::{Texts, lang_dir, settings_path};

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
    let config = MenuConfig {
        app_id: "md.language".to_owned(),
        title: texts.t("escritorio.idioma_titulo"),
        prompt: "🌐".to_owned(),
        accent: md_core::Color::new(0x56, 0xb6, 0xc2),
        row_height: 40.0,
        width: 480.0,
        height: 360.0,
        can_delete: false,
        can_wipe: false,
        submit_empty: false,
        wipe_label: None,
        confirm_wipe_label: None,
    };

    let choice = match run_menu(&SlintHost, &entries, MenuMode::Dmenu, &config) {
        Ok(MenuEvent::Selected(index)) => languages.get(index).map(|(code, _)| code.clone()),
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
