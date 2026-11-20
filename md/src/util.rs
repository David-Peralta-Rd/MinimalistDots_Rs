//! Utilidades compartidas de los subcomandos de `md`.

use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

/// Carga la paleta instalada (o la del repo en desarrollo), con respaldo embebido.
#[allow(clippy::unwrap_used)]
pub fn load_palette() -> md_core::Palette {
    if let Ok(path) = std::env::var("MD_PALETTE") {
        let path = PathBuf::from(path);
        if let Ok(palette) = md_core::Palette::load(&path) {
            return palette;
        }
    }
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        let candidates = [
            home.join(".config/minimalistdots/palette.toml"),
            home.join(".config/hypr/palette.toml"),
        ];
        for path in candidates {
            if let Ok(palette) = md_core::Palette::load(&path) {
                return palette;
            }
        }
    }
    // Respaldo embebido en el binario (paleta por defecto).
    include_str!("../../theme/palette.toml")
        .parse()
        .unwrap_or_else(|_| {
            // No debería fallar: el archivo se valida en compilación.
            md_core::Palette::load(std::path::Path::new("/dev/null")).unwrap()
        })
}

/// Marca temporal para nombres de archivo (`%y%m%d_%Hh%Mm%Ss`).
pub fn now_stamp() -> String {
    Command::new("date")
        .args(["+%y%m%d_%Hh%Mm%Ss"])
        .output()
        .ok()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .unwrap_or_default()
}

/// Segundos Unix actuales (sin subproceso).
pub fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or_default()
}

/// Desfase local respecto a UTC en segundos (de `date +%z`); `0` si falla.
pub fn utc_offset_seconds() -> i64 {
    let output = match Command::new("date").arg("+%z").output() {
        Ok(output) => output,
        Err(_) => return 0,
    };
    let text = String::from_utf8_lossy(&output.stdout);
    md_core::parse_utc_offset(text.trim()).unwrap_or(0)
}

/// Copia `text` al portapapeles con `wl-copy`.
pub fn copy_to_clipboard(text: &str) -> Result<(), std::io::Error> {
    let mut wl = Command::new("wl-copy").stdin(Stdio::piped()).spawn()?;
    if let Some(mut stdin) = wl.stdin.take() {
        stdin.write_all(text.as_bytes())?;
    }
    let status = wl.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("wl-copy terminó con error"))
    }
}

/// Envía una notificación con `notify-send` (la recibe nuestro propio demonio).
/// Los fallos se ignoran: si el demonio no está corriendo no hay a quién avisar.
/// Devuelve la ruta de una miniatura en caché (128 px) para `path`.
/// Si no es una imagen válida o falla, devuelve la ruta original.
pub fn thumbnail(path: &str) -> String {
    if path.is_empty() {
        return String::new();
    }
    let source = std::path::Path::new(path);
    let cache_dir = std::env::var_os("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".cache")))
        .map(|base| base.join("md").join("thumbs"));
    let Some(cache_dir) = cache_dir else {
        return path.to_owned();
    };
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    path.hash(&mut hasher);
    // Incluye metadatos del archivo para invalidar la miniatura si cambia.
    if let Ok(meta) = std::fs::metadata(source) {
        meta.len().hash(&mut hasher);
        if let Ok(mtime) = meta.modified() {
            mtime.hash(&mut hasher);
        }
    }
    let key = format!("{:x}", hasher.finish());
    let cache_path = cache_dir.join(format!("{key}.png"));
    if cache_path.exists() {
        return cache_path.to_string_lossy().into_owned();
    }
    if let Ok(reader) = image::ImageReader::open(source)
        && let Ok(decoded) = reader.decode()
    {
        let thumb = decoded.thumbnail(128, 128);
        let _ = std::fs::create_dir_all(&cache_dir);
        if thumb.save(&cache_path).is_ok() {
            return cache_path.to_string_lossy().into_owned();
        }
    }
    path.to_owned()
}

pub fn notify_send(app: &str, timeout_ms: u32, summary: &str, body: &str) {
    let timeout = timeout_ms.to_string();
    let mut command = Command::new("notify-send");
    command
        .arg("-a")
        .arg(app)
        .arg("-t")
        .arg(&timeout)
        .arg(summary);
    if !body.is_empty() {
        command.arg(body);
    }
    let _ = command.status();
}
