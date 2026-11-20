//! Subcomando `clock`: reloj de escritorio persistente.
//!
//! Reemplaza a `clock.qml` (Quickshell) y al servicio `desktop_clock.lua`, que
//! ya arranca `md clock` al iniciar Hyprland. El widget vive en `md_ui::MdClock`;
//! aquí solo se aporta la hora/fecha actual con `date`, que respeta la
//! localización del sistema (`LC_TIME`/`LANG`) para los nombres de día y mes.

use crate::quickshell;

/// Ejecuta el reloj (QuickShell, residente).
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    quickshell::spawn("clock.qml", &[])?;
    Ok(())
}

/// Imprime un color de texto con contraste para el fondo actual (JSON).
pub fn print_color() -> Result<(), Box<dyn std::error::Error>> {
    let palette = crate::util::load_palette();
    let mut text = palette.text.hex();
    if let Some(home) = std::env::var_os("HOME") {
        let state = std::path::PathBuf::from(home).join(".cache/hypr/current_wallpaper");
        if let Ok(raw) = std::fs::read_to_string(&state) {
            let path = std::path::PathBuf::from(raw.trim());
            if let Ok(img) = image::open(&path) {
                let small = img.resize_exact(32, 32, image::imageops::FilterType::Triangle);
                let rgb = small.to_rgb8();
                let mut sum: u64 = 0;
                let mut count: u64 = 0;
                for px in rgb.pixels() {
                    let l =
                        (0.299 * px[0] as f64 + 0.587 * px[1] as f64 + 0.114 * px[2] as f64) as u64;
                    sum += l;
                    count += 1;
                }
                let avg = sum.checked_div(count).unwrap_or(128);
                text = if avg < 128 {
                    palette.contrast_light.hex()
                } else {
                    palette.contrast_dark.hex()
                };
            }
        }
    }
    println!("{}", serde_json::json!({ "text": text }));
    Ok(())
}
