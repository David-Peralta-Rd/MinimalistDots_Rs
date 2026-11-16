//! Subcomando `welcome`: pantalla de bienvenida (idioma, atajos, novedades, GitHub).

use std::fs;
use std::process::Command;

use md_core::{MenuEntry, MenuMode, Update};
use md_ui::{MenuConfig, MenuEvent, SlintHost, run_menu};

use crate::i18n::{Texts, minimalistdots_dir, settings_path};

fn read_bool(key: &str, default: bool) -> bool {
    let text = fs::read_to_string(settings_path()).unwrap_or_default();
    for line in text.lines() {
        let line = line.trim();
        if let Some((k, v)) = line.split_once('=')
            && k.trim() == key
        {
            return v.trim().trim_matches('"') == "true";
        }
    }
    default
}

fn write_bool(key: &str, value: bool) {
    let path = settings_path();
    let mut lines: Vec<String> = fs::read_to_string(&path)
        .unwrap_or_default()
        .lines()
        .map(str::to_owned)
        .collect();
    let mut found = false;
    for line in &mut lines {
        if line.trim_start().starts_with(key) && line.contains('=') {
            *line = format!("{key} = {}", if value { "true" } else { "false" });
            found = true;
        }
    }
    if !found {
        lines.push(format!("{key} = {}", if value { "true" } else { "false" }));
    }
    let _ = fs::create_dir_all(path.parent().unwrap_or_else(|| std::path::Path::new(".")));
    let _ = fs::write(&path, lines.join("\n") + "\n");
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    if !read_bool("bienvenida", true) {
        return Ok(());
    }

    let texts = Texts::load();
    let github_url = "https://github.com/David-Peralta-Rd/MinimalistDots_Rs";

    loop {
        // Novedades leídas dentro del mismo menú.
        let updates_path = minimalistdots_dir().join("updates.toml");
        let updates: Vec<Update> = md_core::load_updates(&updates_path);

        let mut options: Vec<String> = vec![
            texts.t("escritorio.welcome_idioma"),
            texts.t("escritorio.welcome_atajos"),
            texts.t("escritorio.welcome_github"),
            texts.t("escritorio.welcome_desactivar"),
            texts.t("escritorio.welcome_salir"),
        ];
        // Añade las novedades al final, extendiendo su texto en varias líneas
        // para que quepa toda la información; el menú hace scroll con el mouse.
        let update_offset = options.len();
        for update in &updates {
            options.push(format!("📦 {}", update.titulo));
            if !update.descripcion.is_empty() {
                for line in update.descripcion.lines() {
                    options.push(format!("   {}", line));
                }
            }
        }

        let entries: Vec<MenuEntry> = options.iter().map(|o| MenuEntry::new(o.clone())).collect();
        let config = MenuConfig {
            app_id: "md.welcome".to_owned(),
            title: texts.t("escritorio.welcome_titulo"),
            prompt: "✨".to_owned(),
            accent: md_core::Color::new(0x7e, 0xc7, 0xa2),
            row_height: 44.0,
            width: 720.0,
            height: 560.0,
            can_delete: false,
            can_wipe: false,
            submit_empty: false,
            wipe_label: None,
            confirm_wipe_label: None,
        };

        let choice = match run_menu(&SlintHost, &entries, MenuMode::Dmenu, &config) {
            Ok(MenuEvent::Selected(i)) => i,
            _ => break,
        };

        match choice {
            0 => {
                let _ = crate::language::run();
            }
            1 => {
                let _ = crate::binds::run();
            }
            2 => {
                let _ = Command::new("xdg-open").arg(github_url).spawn();
            }
            3 => {
                write_bool("bienvenida", false);
                println!("Bienvenida desactivada. Reactívala editando settings.toml.");
                break;
            }
            4 => break,
            i if i >= update_offset => {
                // Las novedades son informativas; no hace nada al seleccionar.
            }
            _ => break,
        }
    }
    Ok(())
}
