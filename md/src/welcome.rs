//! Subcomando `welcome`: pantalla de bienvenida (idioma, atajos, novedades, GitHub).

use std::fs;
use std::process::Command;

use md_core::Update;


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
        let updates_path = minimalistdots_dir().join("updates.toml");
        let updates: Vec<Update> = md_core::load_updates(&updates_path);
        let mut changelog: Vec<String> = Vec::new();
        for update in &updates {
            changelog.push(format!("📦 {}", update.titulo));
            if !update.descripcion.is_empty() {
                for line in update.descripcion.lines() {
                    changelog.push(format!("   {}", line));
                }
            }
        }
        if changelog.is_empty() {
            changelog.push("Sin novedades registradas.".to_owned());
        }

        let bienvenida_on = read_bool("bienvenida", true);
        let config = md_ui::WelcomeConfig {
            title: texts.t("escritorio.welcome_titulo"),
            changelog,
            bienvenida_on,
        };

        let result = md_ui::run_welcome(
            &md_ui::SlintHost,
            config,
            move || {
                let _ = crate::language::run();
            },
            move || {
                let _ = crate::binds::run();
            },
            move || {
                let _ = Command::new("xdg-open").arg(github_url).spawn();
            },
            move || {
                let new_val = !read_bool("bienvenida", true);
                write_bool("bienvenida", new_val);
            },
        );
        if result.is_err() {
            break;
        }
        // The window's exit-app callback closes it; loop ends.
        break;
    }
    Ok(())
}
