//! Subcomando `wallpaper`: selector de fondos con `hyprpaper`.
//!
//! Reemplaza a `select_wallpaper.sh`. Las miniaturas se cargan desde la imagen
//! completa (Slint la escala); la generación de miniaturas reales queda para H8.

use std::path::{Path, PathBuf};
use std::process::Command;

use md_core::{MenuEntry, MenuMode, discover_wallpapers};
use md_ui::{MenuConfig, MenuEvent, SlintHost, run_menu};

use crate::i18n::Texts;

use serde_json::json;

/// Ejecuta el selector de fondos de pantalla.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let texts = Texts::load();
    let dir = wallpaper_dir();
    std::fs::create_dir_all(&dir)?;

    let wallpapers = discover_wallpapers(&dir);
    let entries: Vec<MenuEntry> = wallpapers
        .iter()
        .map(|wallpaper| MenuEntry {
            label: wallpaper.name.clone(),
            icon: wallpaper.path.to_string_lossy().into_owned(),
            extra: String::new(),
        })
        .collect();

    let config = MenuConfig {
        app_id: "md.wallpaper".to_owned(),
        title: texts.t("escritorio.fondo_pantalla"),
        prompt: "🖼".to_owned(),
        accent: md_core::Color::new(0x7a, 0xa2, 0xf7), // azul
        row_height: 56.0,                              // filas altas para la miniatura
        width: 560.0,
        height: 520.0,
        can_delete: false,
        can_wipe: false,
        submit_empty: false,
        wipe_label: None,
        confirm_wipe_label: None,
    };

    if let MenuEvent::Selected(index) = run_menu(&SlintHost, &entries, MenuMode::Dmenu, &config)?
        && let Some(wallpaper) = wallpapers.get(index)
    {
        apply(&wallpaper.path)?;
    }
    Ok(())
}

/// Directorio de fondos: `$XDG_PICTURES_DIR/wallpapers` o `~/multimedia/pictures/wallpapers`.
fn wallpaper_dir() -> PathBuf {
    let base = std::env::var_os("XDG_PICTURES_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join("multimedia/pictures"))
        });
    base.map(|dir| dir.join("wallpapers"))
        .unwrap_or_else(|| PathBuf::from("wallpapers"))
}

/// Aplica el fondo con `hyprpaper` y guarda el estado.
fn apply(path: &Path) -> Result<(), std::io::Error> {
    // `,ruta,fill`: monitor vacío = todos los monitores.
    let target = format!(",{},fill", path.display());
    let status = Command::new("hyprctl")
        .args(["hyprpaper", "wallpaper", &target])
        .status()?;
    if !status.success() {
        return Err(std::io::Error::other("hyprpaper no pudo aplicar el fondo"));
    }

    if let Some(state) = state_file() {
        let _ = std::fs::write(state, path.to_string_lossy().as_bytes());
    }
    Ok(())
}

/// Ruta del archivo de estado (`~/.cache/hypr/current_wallpaper`).
fn state_file() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache/hypr/current_wallpaper"))
}

/// Imprime la lista de fondos en JSON (para QuickShell).
pub fn list_json() -> Result<(), Box<dyn std::error::Error>> {
    let texts = crate::i18n::Texts::load();
    let dir = wallpaper_dir();
    std::fs::create_dir_all(&dir)?;
    let wallpapers = discover_wallpapers(&dir);
    let items: Vec<_> = wallpapers
        .iter()
        .map(|wallpaper| {
            json!({
                "name": wallpaper.name,
                "path": wallpaper.path.to_string_lossy(),
            })
        })
        .collect();
    let out = json!({
        "title": texts.t("escritorio.fondo_pantalla"),
        "items": items,
    });
    println!("{}", serde_json::to_string(&out)?);
    Ok(())
}

/// Aplica un fondo a partir de su ruta (para QuickShell).
pub fn apply_path(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    apply(path)?;
    Ok(())
}
