//! Subcomando `pomodoro`: temporizador de concentración/descanso.
//!
//! Reemplaza a `pomodoro.sh` + `pomodoro.qml` (Quickshell). La máquina de estados
//! vive en `md_core::Pomodoro`, el widget en `md_ui::MdPomodoro` y aquí se
//! orquesta: preguntas, sonido, control por proceso (`toggle`/`stop`) y avisos.
//!
//! Modelo de procesos (igual que el script original):
//! - `md pomodoro toggle`/`start`/`test` preguntan (o no) y lanzan
//!   `md pomodoro serve` **desacoplado** con `setsid`; ese proceso escribe su PID
//!   en `widget.pid` y es el que `toggle`/`stop` matan.
//! - El sonido se reproduce en `md pomodoro sound`, también desacoplado, y
//!   `stop` corta el aviso si sigue sonando.

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use clap::Subcommand;

use md_core::MenuEntry;
use crate::menu::{MenuResult, run_entries};

use crate::i18n::Texts;
use crate::util::notify_send;

// --- Sonido de aviso (se cambia solo aquí, no hay menú) ---------------------
// `SOUND_URL`: audio que se descarga UNA vez a la caché; si no es mp3 se convierte
// con ffmpeg. `SOUND_FILE` (ruta a un audio propio) tiene prioridad. Ambos vacíos
// = sin sonido.
const SOUND_URL: &str = "https://download1475.mediafire.com/cirn7tsn2klgi3OfPehbRNAhnBao4EQj-Re1tLo08QkgHbasLe3EtM0V91BmshvhEBI-rUF0pbHv1m0H8KVC1NZ0XyRuaVPx9MShSEyEazahNgEYrwvALEsnlm7Q0BSQEtrQEXThlpC9QGBSx3b43OZLB34H04sNjwOG9I5yoZJvkYU/6chtw5jvqdqjss6/sound_garage_funky_loop.mp3";
const SOUND_FILE: &str = "";
const SOUND_VOLUME: u32 = 70;
const SOUND_MAX_SECONDS: u32 = 10;

/// Acción del subcomando `pomodoro`.
#[derive(Debug, Clone, Subcommand)]
pub enum PomodoroCommand {
    /// Inicia o detiene el temporizador (o cancela las preguntas abiertas).
    Toggle,
    /// Pregunta los tiempos e inicia.
    Start,
    /// Detiene el temporizador en curso.
    Stop,
    /// Reproduce el sonido de aviso (para probarlo).
    Sound,
    /// Prueba rápida: 6 s de concentración y 20 s de descanso.
    Test,
    /// (interno) ejecuta el widget del temporizador.
    #[command(hide = true)]
    Serve {
        /// Segundos de concentración.
        #[arg(long)]
        focus: u32,
        /// Segundos de descanso.
        #[arg(long)]
        brk: u32,
        /// Número de sesiones.
        #[arg(long)]
        sessions: u32,
    },
}

/// Despacha el subcomando.
pub fn run(command: PomodoroCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        PomodoroCommand::Toggle => toggle(),
        PomodoroCommand::Start => start(),
        PomodoroCommand::Stop => stop(),
        PomodoroCommand::Sound => {
            play_sound();
            Ok(())
        }
        PomodoroCommand::Test => test(),
        PomodoroCommand::Serve {
            focus,
            brk,
            sessions,
        } => serve(focus, brk, sessions),
    }
}

// =============================================================================
// Control (toggle / start / stop / test)
// =============================================================================

/// Mismo atajo: cancela las preguntas abiertas, o apaga el temporizador, o inicia.
fn toggle() -> Result<(), Box<dyn std::error::Error>> {
    if prompt_alive() {
        cancel_prompt();
        return Ok(());
    }
    if widget_running() {
        stop()?;
        let texts = Texts::load();
        notify_send(
            "Pomodoro",
            2500,
            &texts.t("escritorio.pomodoro_detenido"),
            "",
        );
    } else {
        start()?;
    }
    Ok(())
}

/// Pregunta concentración/descanso/sesiones e inicia.
fn start() -> Result<(), Box<dyn std::error::Error>> {
    if widget_running() {
        let texts = Texts::load();
        notify_send(
            "Pomodoro",
            2500,
            &texts.t("escritorio.pomodoro_ya_activo"),
            "",
        );
        return Ok(());
    }

    let texts = Texts::load();
    write_prompt_pid();

    let focus = ask(
        &texts,
        "escritorio.pomodoro_concentracion",
        &[25, 15, 20, 30, 45, 50, 60, 90],
    );
    let Some(focus) = focus else {
        clear_prompt_pid();
        return Ok(());
    };
    let brk = ask(
        &texts,
        "escritorio.pomodoro_descanso",
        &[5, 3, 10, 15, 20, 30],
    );
    let Some(brk) = brk else {
        clear_prompt_pid();
        return Ok(());
    };
    let sessions = ask(
        &texts,
        "escritorio.pomodoro_sesiones",
        &[4, 1, 2, 3, 5, 6, 8],
    );
    clear_prompt_pid();
    let Some(sessions) = sessions else {
        return Ok(());
    };

    launch(focus * 60, brk * 60, sessions)
}

/// Detiene el temporizador y corta el sonido.
fn stop() -> Result<(), Box<dyn std::error::Error>> {
    stop_sound();
    if let Some(pid) = read_pid("widget.pid") {
        let _ = Command::new("kill").arg(pid.to_string()).status();
    }
    let _ = std::fs::remove_file(state_dir().join("widget.pid"));
    Ok(())
}

/// Prueba rápida para ver la tarjeta de descanso sin esperar 25 minutos.
fn test() -> Result<(), Box<dyn std::error::Error>> {
    stop()?;
    let texts = Texts::load();
    notify_send(
        "Pomodoro",
        2500,
        &texts.t("escritorio.pomodoro_prueba"),
        &texts.t("escritorio.pomodoro_prueba_cuerpo"),
    );
    launch(6, 20, 2)
}

/// Ejecuta el widget en primer plano (proceso desacoplado por `launch`).
fn serve(focus: u32, brk: u32, sessions: u32) -> Result<(), Box<dyn std::error::Error>> {
    std::fs::create_dir_all(state_dir())?;
    std::fs::write(
        state_dir().join("widget.pid"),
        std::process::id().to_string(),
    )?;

    let _child = std::process::Command::new("qs")
        .arg("-p")
        .arg(format!("{}/.local/bin/MinimalistDots_Rs/quickshell/Pomodoro.qml", std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_owned())))
        .env("MD_POMODORO_FOCUS", (focus * 60).to_string())
        .env("MD_POMODORO_BREAK", (brk * 60).to_string())
        .env("MD_POMODORO_SESSIONS", sessions.to_string())
        .spawn();

    // Mantiene el proceso vivo mientras corre el widget.
    loop {
        std::thread::sleep(std::time::Duration::from_secs(3600));
    }
}

/// Texto "Sesión X de Y" con los marcadores ya sustituidos.
fn session_of(texts: &Texts, session: u32, total: u32) -> String {
    let session = session.to_string();
    let total = total.to_string();
    texts.t_fill(
        "escritorio.pomodoro_sesion_de",
        &[("session", &session), ("total", &total)],
    )
}

// =============================================================================
// Preguntas (menú dmenu reutilizado)
// =============================================================================

/// Pregunta un número eligiendo entre `options` (o tecleándolo). `None` al cancelar.
fn ask(texts: &Texts, key: &str, options: &[u32]) -> Option<u32> {
    let entries: Vec<MenuEntry> = options
        .iter()
        .map(|n| MenuEntry::new(n.to_string()))
        .collect();
    let _ = texts;

    match run_entries(&entries, "🍅", false, false, true, None, None).ok()? {
        MenuResult::Selected(index) => options.get(index).copied(),
        MenuResult::Typed(query) => query.trim().parse().ok().filter(|n| (1..=999).contains(n)),
        _ => None,
    }
}

// =============================================================================
// Procesos y estado
// =============================================================================

/// Directorio de estado (`$XDG_RUNTIME_DIR` o `/tmp`, con sufijo de usuario).
fn state_dir() -> PathBuf {
    let user = std::env::var("USER").unwrap_or_default();
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join(format!("minimalistdots-pomodoro-{user}"))
}

/// Directorio de caché del sonido descargado.
fn sound_cache_dir() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .unwrap_or_else(|| PathBuf::from("/tmp"))
        .join("minimalistdots/pomodoro")
}

/// Ruta del binario actual (`~/.local/bin/md` instalado, o `target/…/md` en dev).
fn current_exe() -> Result<PathBuf, std::io::Error> {
    std::env::current_exe()
}

/// Lanza `md pomodoro serve` desacoplado (nueva sesión con `setsid`).
fn launch(focus: u32, brk: u32, sessions: u32) -> Result<(), Box<dyn std::error::Error>> {
    Command::new("setsid")
        .arg(current_exe()?)
        .args([
            "pomodoro",
            "serve",
            "--focus",
            &focus.to_string(),
            "--brk",
            &brk.to_string(),
            "--sessions",
            &sessions.to_string(),
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    Ok(())
}

/// Lee un PID guardado en `state_dir()/name`.
fn read_pid(name: &str) -> Option<u32> {
    let text = std::fs::read_to_string(state_dir().join(name)).ok()?;
    text.trim().parse().ok()
}

/// ¿El proceso `pid` sigue vivo y es un `md pomodoro`? (evita PID reciclados).
fn pid_alive(pid: u32) -> bool {
    std::fs::read(format!("/proc/{pid}/cmdline"))
        .map(|bytes| bytes.windows(8).any(|w| w == b"pomodoro".as_slice()))
        .unwrap_or(false)
}

/// ¿Hay un temporizador (widget) en curso?
fn widget_running() -> bool {
    read_pid("widget.pid").is_some_and(pid_alive)
}

/// ¿Hay preguntas abiertas (proceso `start` esperando respuesta)?
fn prompt_alive() -> bool {
    read_pid("prompt.pid").is_some_and(pid_alive)
}

/// Cancela las preguntas abiertas matando al proceso que las muestra.
fn cancel_prompt() {
    if let Some(pid) = read_pid("prompt.pid") {
        let _ = Command::new("kill").arg(pid.to_string()).status();
    }
    let _ = std::fs::remove_file(state_dir().join("prompt.pid"));
}

/// Marca este proceso como el que muestra las preguntas.
fn write_prompt_pid() {
    std::fs::create_dir_all(state_dir()).ok();
    let _ = std::fs::write(
        state_dir().join("prompt.pid"),
        std::process::id().to_string(),
    );
}

/// Quita la marca de preguntas abiertas.
fn clear_prompt_pid() {
    let _ = std::fs::remove_file(state_dir().join("prompt.pid"));
}

// =============================================================================
// Sonido
// =============================================================================

/// Reproduce el aviso (desacoplado) desde el widget al cambiar de fase.
fn play_sound_detached() {
    let Ok(exe) = current_exe() else {
        return;
    };
    let _ = Command::new("setsid")
        .arg(&exe)
        .args(["pomodoro", "sound"])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
}

/// Reproduce el aviso en primer plano (subcomando `sound`). Escribe su PID en
/// `sound.pid` para que `stop` pueda cortarlo.
fn play_sound() {
    let Some(file) = ensure_sound() else {
        return;
    };
    let Some(player) = build_player(&file) else {
        return;
    };

    stop_sound();
    std::fs::create_dir_all(state_dir()).ok();
    let _ = std::fs::write(
        state_dir().join("sound.pid"),
        std::process::id().to_string(),
    );

    let max = SOUND_MAX_SECONDS.to_string();
    let _ = Command::new("timeout").arg(&max).args(&player).status();

    if read_pid("sound.pid") == Some(std::process::id()) {
        let _ = std::fs::remove_file(state_dir().join("sound.pid"));
    }
}

/// Corta el sonido en curso (si lo hay).
fn stop_sound() {
    if let Some(pid) = read_pid("sound.pid") {
        let _ = Command::new("kill").arg(pid.to_string()).status();
    }
    let _ = std::fs::remove_file(state_dir().join("sound.pid"));
}

/// Devuelve la ruta del audio listo para reproducir (descargándolo/convirtiéndolo
/// si hace falta), o `None` si no hay sonido configurado o falla la descarga.
fn ensure_sound() -> Option<PathBuf> {
    if !SOUND_FILE.is_empty() && Path::new(SOUND_FILE).is_file() {
        return Some(PathBuf::from(SOUND_FILE));
    }
    if SOUND_URL.is_empty() {
        return None;
    }

    let cache = sound_cache_dir();
    std::fs::create_dir_all(&cache).ok()?;
    let key = url_key(SOUND_URL);
    let out = cache.join(format!("alarm-{key}.mp3"));
    if out.is_file() {
        return Some(out);
    }
    if let Some(raw) = existing_raw(&cache, &key) {
        return Some(raw);
    }

    let tmp = cache.join(format!("alarm-{key}.download"));
    let downloaded = Command::new("curl")
        .args([
            "-fL",
            "--retry",
            "3",
            "--connect-timeout",
            "15",
            "--max-time",
            "90",
            "-sS",
            "-A",
            "MinimalistDots-pomodoro/1.0",
            "-o",
        ])
        .arg(&tmp)
        .arg(SOUND_URL)
        .status()
        .is_ok_and(|status| status.success());
    if !downloaded {
        let _ = std::fs::remove_file(&tmp);
        return None;
    }

    if is_mp3(&tmp) {
        std::fs::rename(&tmp, &out).ok()?;
        return Some(out);
    }

    if command_exists("ffmpeg") {
        let max = (SOUND_MAX_SECONDS + 2).to_string();
        let ok = Command::new("ffmpeg")
            .args(["-nostdin", "-loglevel", "error", "-y", "-i"])
            .arg(&tmp)
            .args(["-vn", "-t", &max, "-codec:a", "libmp3lame", "-q:a", "4"])
            .arg(&out)
            .status()
            .is_ok_and(|status| status.success());
        if ok {
            let _ = std::fs::remove_file(&tmp);
            return Some(out);
        }
        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(&out);
        return None;
    }

    // Sin ffmpeg: se conserva tal cual (lo reproducen mpv/ffplay/cvlc).
    let raw = cache.join(format!("alarm-{key}.raw.{}", extension_of(SOUND_URL)));
    std::fs::rename(&tmp, &raw).ok()?;
    Some(raw)
}

/// Ruta de un `alarm-<key>.raw.*` ya descargado (sin ffmpeg).
fn existing_raw(cache: &Path, key: &str) -> Option<PathBuf> {
    let prefix = format!("alarm-{key}.raw.");
    for entry in std::fs::read_dir(cache).ok()?.flatten() {
        if entry.file_name().to_string_lossy().starts_with(&prefix) {
            return Some(entry.path());
        }
    }
    None
}

/// ¿Es un mp3? Por cabecera: `ID3` o sincronía MPEG (`FF Ex`/`FF Fx`).
fn is_mp3(path: &Path) -> bool {
    use std::io::Read;

    let Ok(mut file) = std::fs::File::open(path) else {
        return false;
    };
    let mut head = [0u8; 3];
    let n = file.read(&mut head).ok().unwrap_or(0);
    if n == 3 && &head == b"ID3" {
        return true;
    }
    n >= 2 && head[0] == 0xFF && (head[1] & 0xE0) == 0xE0
}

/// Extensión del archivo de la URL (acotada a 2–4 letras/dígitos).
fn extension_of(url: &str) -> String {
    let path = url.split('?').next().unwrap_or(url);
    let ext = Path::new(path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("audio");
    if (2..=4).contains(&ext.len()) && ext.bytes().all(|b| b.is_ascii_alphanumeric()) {
        ext.to_owned()
    } else {
        "audio".to_owned()
    }
}

/// Comando del reproductor disponible, según el formato y lo instalado.
fn build_player(file: &Path) -> Option<Vec<String>> {
    let path = file.display().to_string();
    let vol = SOUND_VOLUME.to_string();

    let is_mp3 = file.extension().and_then(|ext| ext.to_str()) == Some("mp3");
    if is_mp3 && command_exists("mpg123") {
        // mpg123: 32768 = 100 %.
        let scale = (32768 * SOUND_VOLUME / 100).to_string();
        return Some(vec!["mpg123".into(), "-q".into(), "-f".into(), scale, path]);
    }
    if command_exists("mpv") {
        return Some(vec![
            "mpv".into(),
            "--no-video".into(),
            "--really-quiet".into(),
            "--no-terminal".into(),
            format!("--volume={vol}"),
            path,
        ]);
    }
    if command_exists("ffplay") {
        return Some(vec![
            "ffplay".into(),
            "-nodisp".into(),
            "-autoexit".into(),
            "-loglevel".into(),
            "quiet".into(),
            "-volume".into(),
            vol,
            path,
        ]);
    }
    if command_exists("cvlc") {
        return Some(vec![
            "cvlc".into(),
            "--play-and-exit".into(),
            "--quiet".into(),
            path,
        ]);
    }
    None
}

/// ¿Está `name` en el `PATH`?
fn command_exists(name: &str) -> bool {
    Command::new("sh")
        .args(["-c", &format!("command -v {name} >/dev/null 2>&1")])
        .status()
        .is_ok_and(|status| status.success())
}

/// Clave corta (8 hex) derivada de la URL para nombrar la caché.
fn url_key(url: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in url.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")[..8].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_key_is_stable_and_short() {
        let key = url_key("https://example.com/a.mp3");
        assert_eq!(key.len(), 8);
        assert_eq!(key, url_key("https://example.com/a.mp3"));
        assert_ne!(url_key("a"), url_key("b"));
    }

    #[test]
    fn extension_of_parses_url() {
        assert_eq!(extension_of("https://x/y/a.mp3?token=1"), "mp3");
        assert_eq!(extension_of("https://x/y/b.ogg"), "ogg");
        // Sin extensión o extensión inválida → "audio".
        assert_eq!(extension_of("https://x/y/noext"), "audio");
    }
}
