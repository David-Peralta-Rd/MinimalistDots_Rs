//! Subcomando `clock`: reloj de escritorio persistente.
//!
//! Reemplaza a `clock.qml` (Quickshell) y al servicio `desktop_clock.lua`, que
//! ya arranca `md clock` al iniciar Hyprland. El widget vive en `md_ui::MdClock`;
//! aquí solo se aporta la hora/fecha actual con `date`, que respeta la
//! localización del sistema (`LC_TIME`/`LANG`) para los nombres de día y mes.

use std::process::Command;
use std::thread;
use std::time::Duration;

use md_ui::{ClockConfig, ClockText, SlintHost, run_clock};


/// Ejecuta el reloj (bloquea hasta que se cierra la ventana).
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Baja el reloj por debajo de las demás ventanas (best-effort): Hyprland no
    // permite layer-shell desde Slint, así que se hace por hyprctl tras crear.
    thread::spawn(|| {
        thread::sleep(Duration::from_secs(2));
        let _ = Command::new("sh")
            .arg("-c")
            .arg(r#"addr=$(hyprctl clients -j 2>/dev/null | (command -v jq >/dev/null && jq -r '.[] | select(.class=="md.clock") | .address' | head -n1 || true)); [ -n "$addr" ] && hyprctl eval "hl.dsp.window.alter_zorder({ window = '$addr', mode = 'bottom' })" >/dev/null 2>&1"#)
            .status();
    });

    let config = ClockConfig {
        app_id: "md.clock".to_owned(),
    };
    run_clock(&SlintHost, &config, current_clock_text)?;
    Ok(())
}

/// Hora y fecha actuales. `date` devuelve `HH:MM` y `%a, %d %b` (p. ej.
/// `"dom, 15 nov"`); la fecha se pasa a minúsculas como hacía el QML original.
fn current_clock_text() -> ClockText {
    let output = Command::new("date")
        .args(["+%H:%M%n%a, %d %b"])
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .unwrap_or_default();

    let mut lines = output.splitn(2, '\n');
    let time = lines.next().unwrap_or("").trim().to_owned();
    let date = lines.next().unwrap_or("").trim().to_lowercase();
    ClockText { time, date }
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
                    let l = (0.299 * px[0] as f64
                        + 0.587 * px[1] as f64
                        + 0.114 * px[2] as f64) as u64;
                    sum += l;
                    count += 1;
                }
                let avg = if count > 0 { sum / count } else { 128 };
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
