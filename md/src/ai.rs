//! Subcomando `ai`: IA local — menú de modelos y servidores `llama-server`.
//!
//! Reemplaza a `llm_menu.sh` + el menú Quickshell. La lógica pura (catálogo de
//! modelos, nombres amigables, `models.env`) vive en `md_core::ai`; aquí se
//! orquesta: menús (`md_ui`), descargas (curl/aria2), procesos (`llama-server`
//! desacoplado con `setsid`), RAM y diagnóstico.
//!
//! Modelo de procesos (igual que el script original):
//! - `md ai toggle`/`start` muestran menús y lanzan `llama-server` desacoplado;
//!   cada servidor escribe su PID en `big.pid`/`small.pid` y `stop` los mata.
//! - Mientras se descarga, el PID de `curl`/`aria2c` queda en `download.pid`;
//!   pulsar el atajo otra vez lo mata y cancela la descarga.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

use clap::{Subcommand, ValueEnum};

use md_core::{MenuEntry, MenuMode};
use md_ui::{MenuConfig, MenuEvent, SlintHost, run_menu};

use crate::i18n::Texts;
use crate::util::notify_send;

// --- Constantes --------------------------------------------------------------
const RAM_MARGIN_GIB: u64 = 3;
const READY_TIMEOUT_SECS: u64 = 600;
const DL_CONNECTIONS: &str = "16";
const APP_NAME: &str = "LLM local";

/// Acción del subcomando `ai`.
#[derive(Debug, Clone, Subcommand)]
pub enum AiCommand {
    /// Mismo atajo: apaga los servidores, cancela la descarga o abre el menú.
    Toggle,
    /// Abre el menú e inicia los modelos elegidos.
    Start,
    /// Apaga los servidores (`chat`, `fim` o `all`).
    Stop {
        #[arg(value_enum, default_value = "all")]
        target: StopTarget,
    },
    /// Muestra qué está corriendo.
    Status,
    /// Diagnóstico: llama-server, red, Hugging Face, disco y RAM.
    Doctor,
    /// Arranque sin menús con los modelos de `models.env`.
    Up {
        #[arg(value_enum, default_value = "chat")]
        target: UpTarget,
        /// Modelo `.gguf` concreto para el chat (anula `CHAT_MODEL`).
        model: Option<String>,
    },
    /// Lista los modelos ya descargados (para `md ai dev`).
    #[command(name = "list-local")]
    ListLocal,
    /// (Re)genera el proveedor local en opencode.json (sin modelo por defecto).
    Config,
    /// Mide tokens/s reales con llama-bench.
    Bench {
        #[arg(value_enum, default_value = "chat")]
        target: UpTarget,
    },
    /// Proyecto nuevo desde la plantilla (git + GitHub opcional).
    New {
        name: String,
        #[arg(long)]
        public: bool,
        #[arg(long)]
        local: bool,
        #[arg(short = 'd', long)]
        desc: Option<String>,
    },
    /// Pregunta qué modelo usar y abre OpenCode.
    Code {
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        args: Vec<String>,
    },
}

/// Qué servidor parar con `stop`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum StopTarget {
    Chat,
    Fim,
    All,
}

/// Qué servidores arrancar con `up`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum UpTarget {
    Chat,
    Fim,
    All,
}

/// Despacha el subcomando.
pub fn run(command: AiCommand) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        AiCommand::Toggle => toggle(),
        AiCommand::Start => start(),
        AiCommand::Stop { target } => {
            stop(target);
            if target == StopTarget::All {
                let texts = Texts::load();
                notify(&texts.t("escritorio.ai_apagado"), "");
            }
            Ok(())
        }
        AiCommand::Status => status(),
        AiCommand::Doctor => {
            if !doctor()? {
                std::process::exit(1);
            }
            Ok(())
        }
        AiCommand::Up { target, model } => {
            if !up(target, model.as_deref())? {
                std::process::exit(1);
            }
            Ok(())
        }
        AiCommand::ListLocal => list_local(),
        AiCommand::Config => config(),
        AiCommand::Bench { target } => bench(target),
        AiCommand::New {
            name,
            public,
            local,
            desc,
        } => new_project(&name, public, local, desc.as_deref()),
        AiCommand::Code { args } => code(&args),
    }
}

// =============================================================================
// Rutas y entorno
// =============================================================================

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

fn runtime_dir() -> PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

fn cache_dir() -> PathBuf {
    std::env::var_os("XDG_CACHE_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

fn config_dir() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from("/tmp"))
}

fn state_dir() -> PathBuf {
    let user = std::env::var("USER").unwrap_or_default();
    runtime_dir().join(format!("llm-servers-{user}"))
}

fn log_dir() -> PathBuf {
    cache_dir().join("llm-servers")
}

fn conf_dir() -> PathBuf {
    config_dir().join("llm-toggle")
}

fn download_dir() -> PathBuf {
    home_dir().join("models")
}

fn model_dirs() -> Vec<PathBuf> {
    let home = home_dir();
    vec![
        home.join("models"),
        home.join(".cache/llama.cpp"),
        home.join(".cache/huggingface/hub"),
    ]
}

fn big_pid_file() -> PathBuf {
    state_dir().join("big.pid")
}

fn small_pid_file() -> PathBuf {
    state_dir().join("small.pid")
}

fn prompt_pid_file() -> PathBuf {
    state_dir().join("prompt.pid")
}

fn download_pid_file() -> PathBuf {
    state_dir().join("download.pid")
}

fn last_file() -> PathBuf {
    conf_dir().join("last")
}

fn big_log() -> PathBuf {
    log_dir().join("big.log")
}

fn small_log() -> PathBuf {
    log_dir().join("small.log")
}

fn download_log() -> PathBuf {
    log_dir().join("download.log")
}

fn menu_log() -> PathBuf {
    log_dir().join("menu.log")
}

fn models_env_path() -> PathBuf {
    std::env::var_os("AI_ENV")
        .map(PathBuf::from)
        .unwrap_or_else(|| config_dir().join("ai/models.env"))
}

fn hf_base() -> String {
    std::env::var("HF_BASE").unwrap_or_else(|_| "https://huggingface.co".to_owned())
}

/// Carga `models.env` (si existe) con los valores por defecto de `llm_menu.sh`.
fn load_models_env() -> md_core::ModelsEnv {
    let get = |name: &str| std::env::var(name).ok();
    match std::fs::read_to_string(models_env_path()) {
        Ok(text) => md_core::parse_models_env(&text, &get),
        Err(_) => md_core::ModelsEnv::default(),
    }
}

// =============================================================================
// Notificaciones y menús
// =============================================================================

fn notify(summary: &str, body: &str) {
    notify_send(APP_NAME, 6000, summary, body);
}

/// Muestra un menú de una columna y devuelve la etiqueta elegida (`None` si se
/// cancela con Esc).
fn choose(title: &str, labels: &[String]) -> Option<String> {
    let entries: Vec<MenuEntry> = labels
        .iter()
        .map(|label| MenuEntry::new(label.clone()))
        .collect();
    let config = MenuConfig {
        app_id: "md.ai.ask".to_owned(),
        title: title.to_owned(),
        prompt: "🤖".to_owned(),
        accent: md_core::Color::new(0x56, 0xb6, 0xc2),
        row_height: 40.0,
        width: 680.0,
        height: 440.0,
        can_delete: false,
        can_wipe: false,
        submit_empty: false,
        wipe_label: None,
        confirm_wipe_label: None,
    };
    match run_menu(&SlintHost, &entries, MenuMode::Dmenu, &config).ok()? {
        MenuEvent::Selected(index) => labels.get(index).cloned(),
        _ => None,
    }
}

// =============================================================================
// Procesos (servidores y descarga)
// =============================================================================

fn pid_exists(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

fn read_pid_file(path: &Path) -> Option<u32> {
    std::fs::read_to_string(path).ok()?.trim().parse().ok()
}

fn is_running(pid_file: &Path) -> bool {
    read_pid_file(pid_file).is_some_and(pid_exists)
}

fn command_exists(name: &str) -> bool {
    Command::new("sh")
        .args(["-c", &format!("command -v {name} >/dev/null 2>&1")])
        .status()
        .is_ok_and(|status| status.success())
}

/// Detiene el servidor cuyo PID está en `pid_file` (SIGTERM, luego SIGKILL).
fn stop_server(pid_file: &Path) {
    let Some(pid) = read_pid_file(pid_file) else {
        let _ = std::fs::remove_file(pid_file);
        return;
    };
    if pid_exists(pid) {
        let _ = Command::new("kill").arg(pid.to_string()).status();
        for _ in 0..40 {
            if !pid_exists(pid) {
                break;
            }
            sleep(Duration::from_millis(250));
        }
        if pid_exists(pid) {
            let _ = Command::new("kill").arg("-9").arg(pid.to_string()).status();
        }
    }
    let _ = std::fs::remove_file(pid_file);
}

/// Redirige stdout y stderr de `cmd` a `log` (en modo anexo).
fn append_log(cmd: &mut Command, log: &Path) {
    if let Ok(file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log)
        && let Ok(stderr) = file.try_clone()
    {
        cmd.stdout(Stdio::from(file)).stderr(Stdio::from(stderr));
    }
}

/// ¿Esta versión de `llama-server` conoce `--device`?
fn supports_device_flag() -> bool {
    Command::new("llama-server")
        .arg("--help")
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).contains("--device"))
        .unwrap_or(false)
}

/// Parámetros de un servidor `llama-server` que arranca [`start_server`].
struct ServerSpec<'a> {
    pid_file: &'a Path,
    model: &'a str,
    port: u16,
    ctx: u32,
    threads: u32,
    log: &'a Path,
    ngl: &'a str,
    jinja: bool,
}

/// Arranca `llama-server` desacoplado (el PID se escribe desde el propio
/// proceso con `setsid` + `sh -c`, igual que el script original).
fn start_server(spec: &ServerSpec) -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = vec![
        "-m".to_owned(),
        spec.model.to_owned(),
        "--alias".to_owned(),
        md_core::friendly_name(spec.model),
        "--host".to_owned(),
        "127.0.0.1".to_owned(),
        "--port".to_owned(),
        spec.port.to_string(),
        "-c".to_owned(),
        spec.ctx.to_string(),
        "-t".to_owned(),
        spec.threads.to_string(),
    ];
    if spec.ngl == "0" {
        args.push("-ngl".to_owned());
        args.push("0".to_owned());
        if supports_device_flag() {
            args.push("--device".to_owned());
            args.push("none".to_owned());
        }
    } else if !spec.ngl.is_empty() {
        args.push("-ngl".to_owned());
        args.push(spec.ngl.to_owned());
    }
    if spec.jinja {
        args.push("--jinja".to_owned());
    }

    let _ = std::fs::remove_file(spec.pid_file);
    let _ = std::fs::create_dir_all(spec.log.parent().unwrap_or_else(|| Path::new("/tmp")));

    let mut cmd = Command::new("setsid");
    cmd.arg("sh")
        .arg("-c")
        .arg("echo $$ > \"$1\"; shift; exec \"$@\"")
        .arg("_")
        .arg(spec.pid_file)
        .arg("llama-server")
        .args(&args)
        .stdin(Stdio::null());
    append_log(&mut cmd, spec.log);

    let status = cmd.spawn()?.wait()?;
    if !status.success() {
        return Err(std::io::Error::other("setsid no pudo arrancar llama-server").into());
    }

    // Esperar a que el PID quede escrito (evita falsos "apagado a mano").
    for _ in 0..60 {
        if read_pid_file(spec.pid_file).is_some() {
            break;
        }
        sleep(Duration::from_millis(50));
    }
    Ok(())
}

/// Estado de la espera a que el servidor responda en `port`.
enum WaitOutcome {
    Ready,
    Timeout,
    Stopped,
    Died,
}

fn wait_ready(port: u16, pid_file: &Path) -> WaitOutcome {
    let mut elapsed = 0u64;
    while elapsed < READY_TIMEOUT_SECS {
        if !pid_file.exists() {
            return WaitOutcome::Stopped;
        }
        match read_pid_file(pid_file) {
            Some(pid) if pid_exists(pid) => {}
            _ => return WaitOutcome::Died,
        }
        if Command::new("curl")
            .args(["-sf", "--max-time", "2"])
            .arg(format!("http://127.0.0.1:{port}/health"))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .is_ok_and(|status| status.success())
        {
            return WaitOutcome::Ready;
        }
        sleep(Duration::from_secs(2));
        elapsed += 2;
    }
    WaitOutcome::Timeout
}

fn tail_text(log: &Path) -> String {
    let Ok(text) = std::fs::read_to_string(log) else {
        return "sin detalle".to_owned();
    };
    let lines: Vec<&str> = text.lines().collect();
    let tail: Vec<&str> = lines.iter().rev().take(2).rev().copied().collect();
    if tail.is_empty() {
        "sin detalle".to_owned()
    } else {
        tail.join(" ")
    }
}

// =============================================================================
// Memoria y disco
// =============================================================================

/// Valor de una clave de `/proc/meminfo` (en bytes).
fn meminfo_kib(key: &str) -> u64 {
    let Ok(text) = std::fs::read_to_string("/proc/meminfo") else {
        return 0;
    };
    for line in text.lines() {
        if let Some((name, value)) = line.split_once(':')
            && name == key
        {
            let kib = value
                .split_whitespace()
                .next()
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(0);
            return kib * 1024;
        }
    }
    0
}

fn mem_available() -> u64 {
    meminfo_kib("MemAvailable")
}

fn mem_total() -> u64 {
    meminfo_kib("MemTotal")
}

fn file_size(path: &Path) -> u64 {
    std::fs::metadata(path).map(|meta| meta.len()).unwrap_or(0)
}

/// Espacio realmente ocupado en disco (con `du`, para archivos dispersos de aria2).
fn disk_bytes(path: &Path) -> u64 {
    if !path.exists() {
        return 0;
    }
    Command::new("du")
        .args(["-B1"])
        .arg(path)
        .output()
        .ok()
        .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
        .and_then(|text| text.split_whitespace().next().and_then(|v| v.parse().ok()))
        .unwrap_or(0)
}

fn free_space(path: &Path) -> Option<u64> {
    let out = Command::new("df")
        .args(["--output=avail", "-B1"])
        .arg(path)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .last()?
        .trim()
        .parse()
        .ok()
}

// =============================================================================
// Descarga de modelos (curl/aria2, reanudable)
// =============================================================================

fn remote_size(url: &str) -> u64 {
    let out = Command::new("curl")
        .args(["-sIL", "--max-time", "30"])
        .arg(url)
        .output()
        .ok();
    let Some(out) = out else { return 0 };
    let mut result = 0u64;
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        let lower = line.trim().to_ascii_lowercase();
        if let Some(value) = lower.strip_prefix("content-length:")
            && let Ok(size) = value.trim().parse::<u64>()
        {
            result = size;
        }
    }
    result
}

/// Ejecuta la descarga y devuelve el código de salida (`-1` = cancelada con
/// señal). Escribe el PID de `curl`/`aria2c` en `download.pid` para poder
/// cancelarla con el atajo.
fn run_download(url: &str, part: &Path, tool: &str, file: &str) -> i32 {
    let mut cmd = if tool == "aria2c" {
        let mut cmd = Command::new("aria2c");
        cmd.args([
            "-c",
            "-x",
            DL_CONNECTIONS,
            "-s",
            DL_CONNECTIONS,
            "-k",
            "8M",
            "--file-allocation=none",
            "--disk-cache=64M",
            "--max-tries=5",
            "--retry-wait=3",
            "--connect-timeout=20",
            "--auto-file-renaming=false",
            "--allow-overwrite=true",
            "--summary-interval=0",
            "--console-log-level=warn",
            "--download-result=hide",
            "-d",
        ])
        .arg(download_dir())
        .arg("-o")
        .arg(format!("{file}.part"))
        .arg(url);
        cmd
    } else {
        let mut cmd = Command::new("curl");
        cmd.args([
            "-L",
            "--fail",
            "--retry",
            "5",
            "--retry-delay",
            "3",
            "--retry-all-errors",
            "--connect-timeout",
            "20",
            "-C",
            "-",
            "-sS",
            "-o",
        ])
        .arg(part)
        .arg(url);
        cmd
    };
    append_log(&mut cmd, &download_log());
    cmd.stdin(Stdio::null());

    let Ok(mut child) = cmd.spawn() else {
        return 1;
    };
    let _ = std::fs::create_dir_all(state_dir());
    let _ = std::fs::write(download_pid_file(), child.id().to_string());

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let _ = std::fs::remove_file(download_pid_file());
                if status.success() {
                    return 0;
                }
                if status.code().is_none() {
                    return -1; // matada con señal (cancelada con el atajo)
                }
                return status.code().unwrap_or(1);
            }
            Ok(None) => sleep(Duration::from_secs(3)),
            Err(_) => {
                let _ = std::fs::remove_file(download_pid_file());
                return 1;
            }
        }
    }
}

/// Descarga `reference` (`dl:repo/archivo.gguf`) a `~/models`. Devuelve si quedó
/// completo. Notifica el resultado (salvo al cancelar, que ya avisa `toggle`).
fn download_model(texts: &Texts, reference: &str) -> bool {
    let repo = md_core::dl_repo(reference);
    let file = md_core::dl_file(reference);
    let url = format!("{}/{}/resolve/main/{}", hf_base(), repo, file);
    let dest = download_dir().join(&file);
    let part = download_dir().join(format!("{file}.part"));

    if std::fs::create_dir_all(download_dir()).is_err() {
        let dir = download_dir().display().to_string();
        notify(
            &texts.t_fill("escritorio.ai_sin_crear_dir", &[("dir", &dir)]),
            &texts.t("escritorio.ai_sin_crear_dir_cuerpo"),
        );
        return false;
    }

    let mut total = remote_size(&url);
    if total < 1048576 {
        total = md_core::preset_bytes(reference);
    }
    let have = disk_bytes(&part);
    let need = total.saturating_sub(have);

    if let Some(free) = free_space(&download_dir())
        && need > 0
        && free < need
    {
        let dir = download_dir().display().to_string();
        notify(
            &texts.t_fill("escritorio.ai_sin_espacio", &[("archivo", &file)]),
            &texts.t_fill(
                "escritorio.ai_sin_espacio_cuerpo",
                &[
                    ("necesito", &md_core::gib(need)),
                    ("libre", &md_core::gib(free)),
                    ("dir", &dir),
                ],
            ),
        );
        return false;
    }

    let tool = if command_exists("aria2c") {
        "aria2c"
    } else {
        "curl"
    };
    let conexiones = if tool == "aria2c" {
        format!("{DL_CONNECTIONS} conexiones")
    } else {
        "1 conexión; instala aria2 para ir más rápido".to_owned()
    };
    notify(
        &texts.t_fill(
            "escritorio.ai_descargando",
            &[("modelo", &md_core::friendly_name(&file))],
        ),
        &format!(
            "{} GB desde Hugging Face ({}). Pulsa el atajo otra vez para cancelar.",
            md_core::gib(total),
            conexiones
        ),
    );

    let part_aria2 = PathBuf::from(format!("{}.aria2", part.display()));
    let rc = if total > 0 && have >= total && !part_aria2.exists() {
        0
    } else {
        run_download(&url, &part, tool, &file)
    };

    match rc {
        -1 => return false, // cancelada: ya avisó `toggle`
        0 => {}
        code => {
            notify(
                &texts.t_fill("escritorio.ai_descarga_fallo", &[("archivo", &file)]),
                &format!(
                    "{tool} terminó con código {code}. Revisa tu conexión. (log: {})",
                    download_log().display()
                ),
            );
            return false;
        }
    }

    let cur = std::fs::metadata(&part).map(|meta| meta.len()).unwrap_or(0);
    if total >= 1048576 && cur < total * 99 / 100 {
        notify(
            &texts.t_fill("escritorio.ai_descarga_incompleta", &[("archivo", &file)]),
            &texts.t_fill(
                "escritorio.ai_descarga_incompleta_cuerpo",
                &[
                    ("actual", &md_core::gib(cur)),
                    ("total", &md_core::gib(total)),
                ],
            ),
        );
        return false;
    }
    if std::fs::rename(&part, &dest).is_err() {
        return false;
    }
    notify(
        &texts.t("escritorio.ai_descargado"),
        &md_core::friendly_name(&file),
    );
    true
}

/// Deja la ruta local del `.gguf` (descargándolo si hace falta) o `None`.
fn resolve_ref(texts: &Texts, reference: &str) -> Option<String> {
    if reference.strip_prefix("dl:").is_some() {
        let path = download_dir().join(md_core::dl_file(reference));
        if path.is_file() {
            return Some(path.display().to_string());
        }
        if download_model(texts, reference) {
            return Some(path.display().to_string());
        }
        return None;
    }
    Some(reference.to_owned())
}

// =============================================================================
// Descubrir modelos
// =============================================================================

/// Una entrada del menú de modelos (descargado o descargable).
struct ModelEntry {
    label: String,
    reference: String,
    category: String,
    local: bool,
}

fn find_gguf_files(dir: &Path, depth: u32, out: &mut Vec<PathBuf>) {
    if depth > 6 {
        return;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        // `file_type` no sigue enlaces: evita ciclos en el caché de Hugging Face.
        let is_dir = entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false);
        if is_dir {
            find_gguf_files(&path, depth + 1, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("gguf") {
            files.push(path);
        }
    }
    files.sort();
    out.extend(files);
}

/// ¿Es un trozo posterior de un modelo partido en varios archivos?
fn is_non_first_split(base: &str) -> bool {
    let Some(stem) = base.strip_suffix(".gguf") else {
        return false;
    };
    let mut parts = stem.rsplit('-');
    let total = parts.next().unwrap_or("");
    let sep = parts.next().unwrap_or("");
    let index = parts.next().unwrap_or("");
    sep == "of"
        && total.len() == 5
        && total.bytes().all(|b| b.is_ascii_digit())
        && index.len() == 5
        && index.bytes().all(|b| b.is_ascii_digit())
        && index != "00001"
}

fn preset_category<'a>(presets: &'a [md_core::Preset], filename: &str) -> Option<&'a str> {
    presets
        .iter()
        .find(|preset| preset.file == filename)
        .map(|preset| preset.category.as_str())
}

/// Escanea los directorios de modelos y el catálogo, y devuelve las entradas
/// del menú (primero las descargadas, luego las descargables).
fn scan_models() -> Vec<ModelEntry> {
    let presets = md_core::presets();
    let mut entries: Vec<ModelEntry> = Vec::new();
    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut have_file: HashSet<String> = HashSet::new();
    let total = mem_total();

    for dir in model_dirs() {
        if !dir.is_dir() {
            continue;
        }
        let mut files = Vec::new();
        find_gguf_files(&dir, 0, &mut files);
        for file in files {
            let base = file
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            if base.to_lowercase().contains("mmproj") || is_non_first_split(&base) {
                continue;
            }
            let real = std::fs::canonicalize(&file).unwrap_or_else(|_| file.clone());
            if !seen.insert(real) {
                continue;
            }
            have_file.insert(base.clone());
            let bytes = file_size(&file);
            let category = preset_category(&presets, &base)
                .unwrap_or(md_core::CAT_LOCAL)
                .to_owned();
            let label = format!(
                "✔ {}  [{} GB]{}{}",
                md_core::friendly_name(&file.display().to_string()),
                md_core::gib(bytes),
                md_core::agent_tag(&base),
                md_core::ram_warn(bytes, total),
            );
            entries.push(ModelEntry {
                label,
                reference: file.display().to_string(),
                category,
                local: true,
            });
        }
    }

    for preset in &presets {
        if have_file.contains(&preset.file) {
            continue;
        }
        let mut extra = String::new();
        let part = download_dir().join(format!("{}.part", preset.file));
        if part.exists() {
            extra = format!(", a medias: {} GB", md_core::gib(disk_bytes(&part)));
        }
        let bytes = (preset.size_gib * md_core::GIB as f64) as u64;
        let label = format!(
            "⬇ {}  [~{:.1}{} GB] — {}{}{}",
            preset.label,
            preset.size_gib,
            extra,
            preset.note,
            md_core::agent_tag(&preset.file),
            md_core::ram_warn(bytes, total),
        );
        entries.push(ModelEntry {
            label,
            reference: format!("dl:{}/{}", preset.repo, preset.file),
            category: preset.category.clone(),
            local: false,
        });
    }

    entries
}

/// Bytes de una referencia (archivo local o `dl:…`).
fn ref_bytes(reference: &str) -> u64 {
    if reference.starts_with("dl:") {
        let path = download_dir().join(md_core::dl_file(reference));
        if path.is_file() {
            return file_size(&path);
        }
        return md_core::preset_bytes(reference);
    }
    file_size(Path::new(reference))
}

// =============================================================================
// Selección de modelos (menú de dos niveles)
// =============================================================================

enum Filter {
    None,
    Category(String),
    Downloaded,
    All,
}

/// Menú de dos niveles (categoría → modelo). Devuelve la referencia elegida
/// (`"-"` = ninguno) o `None` al cancelar. `first` coloca una categoría primero.
fn pick_model(
    texts: &Texts,
    title: &str,
    first: Option<&str>,
    entries: &[ModelEntry],
) -> Option<String> {
    let none = texts.t("escritorio.ai_ninguno");
    let back = texts.t("escritorio.ai_volver");
    let all_label = texts.t("escritorio.ai_todos");
    let dl_label = texts.t("escritorio.ai_descargados");

    loop {
        // --- nivel 1: categorías ---
        let mut lines: Vec<(String, Filter)> = vec![(none.clone(), Filter::None)];
        let mut categories: Vec<(String, usize)> = Vec::new();
        for cat in md_core::CATEGORIES
            .iter()
            .copied()
            .chain(std::iter::once(md_core::CAT_LOCAL))
        {
            let count = entries.iter().filter(|entry| entry.category == cat).count();
            if count > 0 {
                categories.push((cat.to_owned(), count));
            }
        }
        if let Some(first) = first
            && let Some((cat, count)) = categories.iter().find(|(cat, _)| cat == first)
        {
            lines.push((format!("{cat}  ({count})"), Filter::Category(cat.clone())));
        }
        for (cat, count) in &categories {
            if Some(cat.as_str()) == first {
                continue;
            }
            lines.push((format!("{cat}  ({count})"), Filter::Category(cat.clone())));
        }
        let local_count = entries.iter().filter(|entry| entry.local).count();
        if local_count > 0 {
            lines.push((format!("{dl_label}  ({local_count})"), Filter::Downloaded));
        }
        lines.push((format!("{all_label}  ({})", entries.len()), Filter::All));

        let labels: Vec<String> = lines.iter().map(|(label, _)| label.clone()).collect();
        let choice = choose(title, &labels)?;
        let Some((_, filter)) = lines.iter().find(|(label, _)| *label == choice) else {
            continue;
        };

        let indices: Vec<usize> = match filter {
            Filter::None => return Some("-".to_owned()),
            Filter::Category(cat) => entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| &entry.category == cat)
                .map(|(index, _)| index)
                .collect(),
            Filter::Downloaded => entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| entry.local)
                .map(|(index, _)| index)
                .collect(),
            Filter::All => (0..entries.len()).collect(),
        };

        // --- nivel 2: modelos de esa categoría ---
        let mut labels = vec![back.clone()];
        labels.extend(indices.iter().map(|&index| entries[index].label.clone()));
        let choice = choose(title, &labels)?;
        if choice == back {
            continue;
        }
        if let Some(&index) = indices
            .iter()
            .find(|&&index| entries[index].label == choice)
        {
            return Some(entries[index].reference.clone());
        }
        return None;
    }
}

fn pick_context(texts: &Texts) -> Option<u32> {
    let options = [
        (texts.t("escritorio.ai_ctx_16k"), 16384u32),
        (texts.t("escritorio.ai_ctx_8k"), 8192),
        (texts.t("escritorio.ai_ctx_4k"), 4096),
        (texts.t("escritorio.ai_ctx_32k"), 32768),
    ];
    let labels: Vec<String> = options.iter().map(|(label, _)| label.clone()).collect();
    let choice = choose(&texts.t("escritorio.ai_contexto_titulo"), &labels)?;
    options
        .iter()
        .find(|(label, _)| *label == choice)
        .map(|(_, value)| *value)
}

fn label_or_none(reference: &str, none_label: &str) -> String {
    if reference == "-" {
        none_label.to_owned()
    } else {
        md_core::friendly_name(reference)
    }
}

fn last_entry_valid(reference: &str) -> bool {
    reference == "-" || reference.starts_with("dl:") || Path::new(reference).is_file()
}

fn read_last() -> Option<(String, String, u32)> {
    let text = std::fs::read_to_string(last_file()).ok()?;
    let mut lines = text.lines();
    let big = lines.next().unwrap_or("-").to_owned();
    let small = lines.next().unwrap_or("-").to_owned();
    let ctx = lines
        .next()
        .and_then(|line| line.parse().ok())
        .unwrap_or(16384);
    Some((big, small, ctx))
}

fn write_last(big: &str, small: &str, ctx: u32) {
    let _ = std::fs::create_dir_all(conf_dir());
    let _ = std::fs::write(last_file(), format!("{big}\n{small}\n{ctx}\n"));
}

// =============================================================================
// Comandos
// =============================================================================

fn check_deps(texts: &Texts) -> bool {
    let mut missing: Vec<&str> = Vec::new();
    for cmd in ["llama-server", "curl"] {
        if !command_exists(cmd) {
            missing.push(cmd);
        }
    }
    if missing.is_empty() {
        return true;
    }
    let programas = missing.join(" ");
    notify(
        &texts.t("escritorio.ai_faltan_programas"),
        &texts.t_fill(
            "escritorio.ai_faltan_programas_cuerpo",
            &[("programas", &programas)],
        ),
    );
    false
}

fn stop(target: StopTarget) {
    match target {
        StopTarget::Chat => stop_server(&big_pid_file()),
        StopTarget::Fim => stop_server(&small_pid_file()),
        StopTarget::All => {
            stop_server(&big_pid_file());
            stop_server(&small_pid_file());
        }
    }
}

fn prompt_alive() -> bool {
    read_pid_file(&prompt_pid_file()).is_some_and(pid_exists)
}

fn cancel_prompt() {
    if let Some(pid) = read_pid_file(&prompt_pid_file()) {
        let _ = Command::new("kill").arg(pid.to_string()).status();
    }
    let _ = std::fs::remove_file(prompt_pid_file());
}

fn write_prompt_pid() {
    let _ = std::fs::create_dir_all(state_dir());
    let _ = std::fs::write(prompt_pid_file(), std::process::id().to_string());
}

fn clear_prompt_pid() {
    let _ = std::fs::remove_file(prompt_pid_file());
}

/// Mismo atajo: cancela la descarga, o las preguntas, o apaga los servidores,
/// o inicia el menú.
fn toggle() -> Result<(), Box<dyn std::error::Error>> {
    let texts = Texts::load();

    if let Some(pid) = read_pid_file(&download_pid_file())
        && pid_exists(pid)
    {
        let _ = Command::new("kill").arg(pid.to_string()).status();
        notify(
            &texts.t("escritorio.ai_descarga_cancelada"),
            &texts.t("escritorio.ai_descarga_cancelada_cuerpo"),
        );
        return Ok(());
    }

    if prompt_alive() {
        cancel_prompt();
        return Ok(());
    }

    if is_running(&big_pid_file()) || is_running(&small_pid_file()) {
        stop(StopTarget::All);
        notify(
            &texts.t("escritorio.ai_apagado"),
            &texts.t("escritorio.ai_apagado_cuerpo"),
        );
    } else {
        start()?;
    }
    Ok(())
}

/// Pregunta los modelos, descarga lo que falte y arranca los servidores.
fn start() -> Result<(), Box<dyn std::error::Error>> {
    let texts = Texts::load();
    let env = load_models_env();

    if !check_deps(&texts) {
        return Ok(());
    }
    if is_running(&big_pid_file()) || is_running(&small_pid_file()) {
        notify(&texts.t("escritorio.ai_ya_activo"), "");
        return Ok(());
    }

    write_prompt_pid();

    let entries = scan_models();
    let mut big = "-".to_owned();
    let mut small = "-".to_owned();
    let mut ctx: u32 = 16384;

    if let Some((last_big, last_small, last_ctx)) = read_last()
        && last_entry_valid(&last_big)
        && last_entry_valid(&last_small)
    {
        let repeat = texts.t_fill(
            "escritorio.ai_repetir",
            &[
                (
                    "chat",
                    &label_or_none(&last_big, &texts.t("escritorio.ai_sin_chat")),
                ),
                (
                    "fim",
                    &label_or_none(&last_small, &texts.t("escritorio.ai_sin_fim")),
                ),
                ("ctx", &last_ctx.to_string()),
            ],
        );
        let manual = texts.t("escritorio.ai_manual");
        let labels = vec![repeat.clone(), manual];
        match choose(&texts.t("escritorio.ai_titulo"), &labels) {
            Some(choice) if choice == repeat => {
                big = last_big;
                small = last_small;
                ctx = last_ctx;
            }
            Some(_) => {}
            None => {
                clear_prompt_pid();
                return Ok(());
            }
        }
    }

    if big == "-" {
        let chat_title = texts.t_fill(
            "escritorio.ai_chat_titulo",
            &[("puerto", &env.chat_port.to_string())],
        );
        let Some(big_ref) = pick_model(&texts, &chat_title, Some(md_core::CAT_CODE), &entries)
        else {
            clear_prompt_pid();
            return Ok(());
        };
        big = big_ref;

        let fim_title = texts.t_fill(
            "escritorio.ai_fim_titulo",
            &[("puerto", &env.fim_port.to_string())],
        );
        let Some(small_ref) = pick_model(&texts, &fim_title, Some(md_core::CAT_FIM), &entries)
        else {
            clear_prompt_pid();
            return Ok(());
        };
        small = small_ref;

        if big != "-" {
            let Some(context) = pick_context(&texts) else {
                clear_prompt_pid();
                return Ok(());
            };
            ctx = context;
        }
    }

    if big == "-" && small == "-" {
        notify(&texts.t("escritorio.ai_sin_modelos"), "");
        clear_prompt_pid();
        return Ok(());
    }

    // --- ¿Cabe en RAM? ---
    let mut need = RAM_MARGIN_GIB * md_core::GIB;
    if big != "-" {
        need += ref_bytes(&big);
    }
    if small != "-" {
        need += ref_bytes(&small);
    }
    let avail = mem_available();
    if need > avail {
        let title = texts.t_fill(
            "escritorio.ai_ram_titulo",
            &[
                ("necesito", &md_core::gib(need)),
                ("libre", &md_core::gib(avail)),
            ],
        );
        let labels = vec![
            texts.t("escritorio.ai_continuar"),
            texts.t("escritorio.ai_cancelar"),
        ];
        match choose(&title, &labels) {
            Some(choice) if choice == labels[0] => {}
            _ => {
                clear_prompt_pid();
                return Ok(());
            }
        }
    }

    write_last(&big, &small, ctx);

    // --- Descargar lo que falte (el atajo cancela) ---
    clear_prompt_pid();
    let mut big_path = "-".to_owned();
    let mut small_path = "-".to_owned();
    if big != "-" {
        let Some(path) = resolve_ref(&texts, &big) else {
            return Ok(());
        };
        big_path = path;
    }
    if small != "-" {
        let Some(path) = resolve_ref(&texts, &small) else {
            return Ok(());
        };
        small_path = path;
    }

    notify(
        &texts.t("escritorio.ai_iniciando"),
        &texts.t("escritorio.ai_cargando"),
    );

    let mut ports: Vec<u16> = Vec::new();
    let mut pid_files: Vec<PathBuf> = Vec::new();
    let mut logs: Vec<PathBuf> = Vec::new();
    let mut names: Vec<String> = Vec::new();

    if big_path != "-" {
        let log = big_log();
        start_server(&ServerSpec {
            pid_file: &big_pid_file(),
            model: &big_path,
            port: env.chat_port,
            ctx,
            threads: env.threads,
            log: &log,
            ngl: &env.ngl,
            jinja: true,
        })?;
        ports.push(env.chat_port);
        pid_files.push(big_pid_file());
        logs.push(log);
        names.push(format!(
            "Chat :{} → {}",
            env.chat_port,
            md_core::friendly_name(&big)
        ));
    }
    if small_path != "-" {
        let log = small_log();
        start_server(&ServerSpec {
            pid_file: &small_pid_file(),
            model: &small_path,
            port: env.fim_port,
            ctx: env.fim_ctx,
            threads: env.fim_threads,
            log: &log,
            ngl: &env.ngl,
            jinja: false,
        })?;
        ports.push(env.fim_port);
        pid_files.push(small_pid_file());
        logs.push(log);
        names.push(format!(
            "Autocomplete :{} → {}",
            env.fim_port,
            md_core::friendly_name(&small)
        ));
    }

    let mut ok_lines: Vec<String> = Vec::new();
    let mut fail_lines: Vec<String> = Vec::new();
    for i in 0..ports.len() {
        match wait_ready(ports[i], &pid_files[i]) {
            WaitOutcome::Ready => ok_lines.push(names[i].clone()),
            WaitOutcome::Stopped => {
                clear_prompt_pid();
                return Ok(());
            }
            WaitOutcome::Died => {
                let _ = std::fs::remove_file(&pid_files[i]);
                fail_lines.push(format!("{} se cerró: {}", names[i], tail_text(&logs[i])));
            }
            WaitOutcome::Timeout => fail_lines.push(format!(
                "{}: sin respuesta (mira {})",
                names[i],
                logs[i].display()
            )),
        }
    }

    if fail_lines.is_empty() {
        notify(&texts.t("escritorio.ai_listo"), &ok_lines.join("\n"));
    } else {
        notify(&texts.t("escritorio.ai_problemas"), &fail_lines.join("\n"));
    }
    Ok(())
}

/// Muestra qué está corriendo (salida de terminal, en español).
fn status() -> Result<(), Box<dyn std::error::Error>> {
    let env = load_models_env();
    let mut any = false;
    if is_running(&big_pid_file()) {
        println!(
            "chat         :{}   PID {}",
            env.chat_port,
            read_pid_file(&big_pid_file()).unwrap_or(0)
        );
        any = true;
    }
    if is_running(&small_pid_file()) {
        println!(
            "autocomplete :{}   PID {}",
            env.fim_port,
            read_pid_file(&small_pid_file()).unwrap_or(0)
        );
        any = true;
    }
    if read_pid_file(&download_pid_file()).is_some_and(pid_exists) {
        println!("descargando… (log: {})", download_log().display());
        any = true;
    }
    if !any {
        println!("apagado");
    }
    Ok(())
}

/// Código HTTP de una URL (con `curl`, opcionalmente HEAD + seguir redirecciones).
fn http_code(url: &str, head: bool, max_time: u32) -> u16 {
    let mut cmd = Command::new("curl");
    cmd.args([
        "-s",
        "-o",
        "/dev/null",
        "-w",
        "%{http_code}",
        "--max-time",
        &max_time.to_string(),
    ]);
    if head {
        cmd.args(["-I", "-L"]);
    }
    cmd.arg(url);
    cmd.output()
        .ok()
        .and_then(|out| String::from_utf8_lossy(&out.stdout).trim().parse().ok())
        .unwrap_or(0)
}

fn llama_server_version() -> String {
    let out = Command::new("llama-server").arg("--version").output().ok();
    let Some(out) = out else {
        return "instalado".to_owned();
    };
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    text.lines()
        .find(|line| line.to_lowercase().contains("version"))
        .unwrap_or("instalado")
        .to_owned()
}

/// Diagnóstico para instalaciones nuevas (salida de terminal, en español).
fn doctor() -> Result<bool, Box<dyn std::error::Error>> {
    let mut problems = 0u32;
    println!("== LLM local: diagnóstico ==");

    if command_exists("llama-server") {
        println!("✔ llama-server: {}", llama_server_version());
        if supports_device_flag() {
            println!("✔ --device soportado (CPU pura con NGL=0)");
        } else {
            println!("· --device no existe en esta versión (se omite solo)");
        }
    } else {
        println!("✘ llama-server no está instalado (paquete llama-cpp)");
        problems += 1;
    }

    if command_exists("aria2c") {
        println!("✔ aria2c: descargas con {DL_CONNECTIONS} conexiones en paralelo");
    } else {
        println!(
            "· aria2c no instalado: se descarga con 1 conexión (instala el paquete aria2 para más velocidad)"
        );
    }

    {
        let cmd = "curl";
        if command_exists(cmd) {
            println!("✔ {cmd}");
        } else {
            println!("✘ falta {cmd}");
            problems += 1;
        }
    }

    let code = http_code(&hf_base(), false, 15);
    if matches!(code, 200 | 301 | 302) {
        println!("✔ Hugging Face responde (HTTP {code})");
    } else {
        println!(
            "✘ No se puede llegar a {} (HTTP {}). Revisa red, DNS, proxy o certificados (ca-certificates).",
            hf_base(),
            if code == 0 {
                "sin respuesta".to_owned()
            } else {
                code.to_string()
            }
        );
        problems += 1;
    }

    for preset in md_core::presets() {
        let url = format!("{}/{}/resolve/main/{}", hf_base(), preset.repo, preset.file);
        let code = http_code(&url, true, 20);
        if code == 200 {
            println!(
                "✔ {} existe en Hugging Face (~{:.1} GB)",
                preset.file, preset.size_gib
            );
        } else {
            println!(
                "✘ {} → HTTP {} (el archivo cambió de nombre o no hay red)",
                preset.file,
                if code == 0 {
                    "sin respuesta".to_owned()
                } else {
                    code.to_string()
                }
            );
            problems += 1;
        }
    }

    let _ = std::fs::create_dir_all(download_dir());
    let free = free_space(&download_dir()).unwrap_or(0);
    println!(
        "· Espacio libre en {}: {} GB",
        download_dir().display(),
        md_core::gib(free)
    );
    println!("· RAM libre: {} GB", md_core::gib(mem_available()));
    println!(
        "· Logs: {} | {} | {} | {}",
        menu_log().display(),
        download_log().display(),
        big_log().display(),
        small_log().display()
    );

    if problems == 0 {
        println!("Todo en orden.");
    } else {
        println!("Problemas encontrados: {problems}");
    }
    Ok(problems == 0)
}

/// Arranque sin menús desde `models.env` (salida de terminal, en español).
fn up(target: UpTarget, model: Option<&str>) -> Result<bool, Box<dyn std::error::Error>> {
    let env = load_models_env();
    let mut chat_model = env.chat_model.clone();
    if let Some(model) = model {
        chat_model = model.to_owned();
    }
    let (want_chat, want_fim) = match target {
        UpTarget::Chat => (true, false),
        UpTarget::Fim => (false, true),
        UpTarget::All => (true, true),
    };

    for cmd in ["llama-server", "curl"] {
        if !command_exists(cmd) {
            eprintln!("Falta '{cmd}' (paquete llama-cpp / curl).");
            return Ok(false);
        }
    }

    let mut need = RAM_MARGIN_GIB * md_core::GIB;
    if want_chat && !is_running(&big_pid_file()) {
        if !Path::new(&chat_model).is_file() {
            eprintln!(
                "No existe CHAT_MODEL: '{chat_model}'. Edita {} o descárgalo con el menú (SUPER+ALT+SHIFT+L).",
                models_env_path().display()
            );
            return Ok(false);
        }
        need += file_size(Path::new(&chat_model));
    }
    if want_fim && !is_running(&small_pid_file()) {
        if !Path::new(&env.fim_model).is_file() {
            eprintln!(
                "No existe FIM_MODEL: '{}'. Edita {} o descárgalo con el menú (SUPER+ALT+SHIFT+L).",
                env.fim_model,
                models_env_path().display()
            );
            return Ok(false);
        }
        need += file_size(Path::new(&env.fim_model));
    }

    let avail = mem_available();
    if need > avail && std::env::var("AI_FORCE").as_deref() != Ok("1") {
        eprintln!(
            "Hacen falta ~{} GB y hay {} GB libres. Cierra apps o baja CHAT_CTX en {} (o AI_FORCE=1 para forzar).",
            md_core::gib(need),
            md_core::gib(avail),
            models_env_path().display()
        );
        return Ok(false);
    }

    let mut ports: Vec<u16> = Vec::new();
    let mut pid_files: Vec<PathBuf> = Vec::new();
    let mut logs: Vec<PathBuf> = Vec::new();
    let mut names: Vec<String> = Vec::new();

    if want_chat {
        if is_running(&big_pid_file()) {
            println!("chat ya está activo en :{}", env.chat_port);
        } else {
            let log = big_log();
            start_server(&ServerSpec {
                pid_file: &big_pid_file(),
                model: &chat_model,
                port: env.chat_port,
                ctx: env.chat_ctx,
                threads: env.threads,
                log: &log,
                ngl: &env.ngl,
                jinja: true,
            })?;
            ports.push(env.chat_port);
            pid_files.push(big_pid_file());
            logs.push(log);
            names.push(format!(
                "chat :{} (ctx {}, {} hilos)",
                env.chat_port, env.chat_ctx, env.threads
            ));
        }
    }
    if want_fim {
        if is_running(&small_pid_file()) {
            println!("autocompletado ya está activo en :{}", env.fim_port);
        } else {
            let log = small_log();
            start_server(&ServerSpec {
                pid_file: &small_pid_file(),
                model: &env.fim_model,
                port: env.fim_port,
                ctx: env.fim_ctx,
                threads: env.fim_threads,
                log: &log,
                ngl: &env.ngl,
                jinja: false,
            })?;
            ports.push(env.fim_port);
            pid_files.push(small_pid_file());
            logs.push(log);
            names.push(format!(
                "autocompletado :{} (ctx {}, {} hilos)",
                env.fim_port, env.fim_ctx, env.fim_threads
            ));
        }
    }
    if ports.is_empty() {
        return Ok(true);
    }

    println!("Cargando el modelo en memoria (los grandes tardan un poco)…");
    let mut failed = false;
    for i in 0..ports.len() {
        match wait_ready(ports[i], &pid_files[i]) {
            WaitOutcome::Ready => println!("listo: {}", names[i]),
            WaitOutcome::Died => {
                let _ = std::fs::remove_file(&pid_files[i]);
                eprintln!("se cerró: {}. Últimas líneas del log:", names[i]);
                let _ = std::fs::read_to_string(&logs[i]).map(|text| {
                    for line in text.lines().rev().take(4).collect::<Vec<_>>().iter().rev() {
                        eprintln!("  {line}");
                    }
                });
                failed = true;
            }
            _ => {
                eprintln!("sin respuesta: {} (mira {})", names[i], logs[i].display());
                failed = true;
            }
        }
    }
    Ok(!failed)
}

/// Lista los modelos ya descargados que sirven para chat (para `md ai dev`).
/// Una línea por modelo: `categoría|ruta|nombre|bytes|etiqueta-agente`.
fn list_local() -> Result<(), Box<dyn std::error::Error>> {
    let entries = scan_models();
    for cat in md_core::CATEGORIES
        .iter()
        .copied()
        .chain(std::iter::once(md_core::CAT_LOCAL))
    {
        if cat == md_core::CAT_FIM {
            continue;
        }
        for entry in entries
            .iter()
            .filter(|entry| entry.local && entry.category == cat)
        {
            let base = Path::new(&entry.reference)
                .file_name()
                .map(|name| name.to_string_lossy().into_owned())
                .unwrap_or_default();
            println!(
                "{}|{}|{}|{}|{}",
                cat,
                entry.reference,
                md_core::friendly_name(&entry.reference),
                ref_bytes(&entry.reference),
                md_core::agent_tag(&base),
            );
        }
    }
    Ok(())
}

// ==============================================================================
// CONFIG (OpenCode)
// ==============================================================================

fn opencode_config_path() -> PathBuf {
    config_dir().join("opencode/opencode.json")
}

fn ai_home() -> PathBuf {
    std::env::var_os("MD_AI_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            // .../scripts -> .../ai
            // Fallback: ~/.config no; use installer's ai dir unknown; try common.
            home_dir().join(".config/ai")
        })
}

fn model_alias(reference: &str) -> String {
    md_core::friendly_name(reference)
}

fn config() -> Result<(), Box<dyn std::error::Error>> {
    let env = load_models_env();
    if env.chat_model.is_empty() {
        eprintln!("define CHAT_MODEL en {}", models_env_path().display());
        return Ok(());
    }
    let id = model_alias(&env.chat_model);
    let path = opencode_config_path();
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if !path.exists() {
        let content = format!(
            r#"{{
  "$schema": "https://opencode.ai/config.json",
  "provider": {{
    "llamacpp": {{
      "npm": "@ai-sdk/openai-compatible",
      "name": "llama.cpp (local)",
      "options": {{ "baseURL": "http://127.0.0.1:{}/v1" }},
      "models": {{
        "{}": {{
          "name": "{}",
          "limit": {{ "context": {}, "output": 4096 }}
        }}
      }}
    }}
  }}
}}"#,
            env.chat_port, id, id, env.chat_ctx
        );
        std::fs::write(&path, content)?;
        println!("creado {}", path.display());
        return Ok(());
    }
    // If jq available, update; else print hint.
    if command_exists("jq") {
        // Use oc_set_local logic via jq
        let ctx = env.chat_ctx;
        let url = format!("http://127.0.0.1:{}/v1", env.chat_port);
        let filter = r#".["$schema"] //= "https://opencode.ai/config.json" | if ((.model // "") | startswith("llamacpp/")) then del(.model) else . end | .provider.llamacpp = {npm:"@ai-sdk/openai-compatible",name:"llama.cpp (local)",options:{baseURL:$url},models:{($id):{name:$id,limit:{context:$ctx,output:4096}}}}"#;
        let tmp = std::env::temp_dir().join("opencode.json.tmp");
        let status = Command::new("jq")
            .args([
                "--arg",
                "id",
                &id,
                "--arg",
                "url",
                &url,
                "--argjson",
                "ctx",
                &ctx.to_string(),
                filter,
                path.to_str().unwrap_or(""),
            ])
            .output()?;
        if status.status.success() {
            std::fs::write(&tmp, &status.stdout)?;
            let _ = std::fs::rename(&tmp, &path);
            println!("actualizado el proveedor 'llamacpp' en {}", path.display());
        } else {
            eprintln!("jq falló; {} quedó intacto.", path.display());
        }
    } else {
        eprintln!("{} ya existe y falta jq para actualizarlo.", path.display());
    }
    Ok(())
}

// ==============================================================================
// BENCH
// ==============================================================================
fn bench(target: UpTarget) -> Result<(), Box<dyn std::error::Error>> {
    let env = load_models_env();
    let model = match target {
        UpTarget::Chat => env.chat_model.clone(),
        UpTarget::Fim => env.fim_model.clone(),
        UpTarget::All => env.chat_model.clone(),
    };
    if !command_exists("llama-bench") {
        eprintln!("no encuentro llama-bench (revisa el paquete llama-cpp).");
        return Ok(());
    }
    if !Path::new(&model).is_file() {
        eprintln!(
            "modelo no encontrado: '{}' (edita {})",
            model,
            models_env_path().display()
        );
        return Ok(());
    }
    // check servers running via status
    if is_running(&big_pid_file()) || is_running(&small_pid_file()) {
        eprintln!("hay servidores encendidos; ejecuta primero: md ai stop");
        return Ok(());
    }
    let _ = std::fs::create_dir_all(log_dir());
    let stamp = crate::util::now_stamp();
    let out = log_dir().join(format!("bench-{stamp}.md"));
    let threads = if target == UpTarget::Fim {
        env.fim_threads
    } else {
        env.threads
    };
    println!(
        "Midiendo {} con -t {},{} (CPU pura). Puede tardar varios minutos...",
        model,
        threads,
        threads * 2
    );
    let mut cmd = Command::new("llama-bench");
    cmd.args([
        "-m",
        &model,
        "-ngl",
        "0",
        "-t",
        &format!("{},{}", threads, threads * 2),
        "-p",
        "512",
        "-n",
        "128",
        "-o",
        "md",
    ]);
    if supports_device_flag() {
        cmd.args(["--device", "none"]);
    }
    let output = cmd.output()?;
    let _ = std::fs::write(&out, &output.stdout);
    let _ = std::io::Write::write_all(&mut std::io::stdout(), &output.stdout);
    println!();
    println!(
        "Guardado en {}  (pp = lectura del prompt, tg = generación; anota los tokens/s de tg).",
        out.display()
    );
    Ok(())
}

// ==============================================================================
// NEW (proyecto nuevo)
// ==============================================================================
fn new_project(
    name: &str,
    public: bool,
    local: bool,
    desc: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    if !name
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_')
    {
        eprintln!("el nombre solo puede llevar letras, números, punto, guion y guion bajo.");
        return Ok(());
    }
    let tpl = ai_home().join("templates/project");
    // fallback to installer ai dir
    let tpl = if tpl.is_dir() {
        tpl
    } else {
        home_dir().join(".config/ai/templates/project")
    };
    if !tpl.is_dir() {
        eprintln!(
            "no encuentro la plantilla en {} (ejecuta el instalador).",
            tpl.display()
        );
        return Ok(());
    }
    let dir = std::env::current_dir()?.join(name);
    if dir.exists() {
        eprintln!("{} ya existe.", name);
        return Ok(());
    }
    std::fs::create_dir_all(&dir)?;
    // copy template
    copy_dir_all(&tpl, &dir)?;
    // replace __PROJECT__
    replace_in_md(&dir, "__PROJECT__", name);

    // Instala la skill base `find-skills` (y la espeja para OpenCode).
    if command_exists("npx") {
        // Skills por defecto para nuevos proyectos.
        let defaults: &[(&str, &str)] = &[
            ("vercel-labs/skills", "find-skills"),
            ("mattpocock/skills", "tdd"),
            ("mattpocock/skills", "code-review"),
            ("mattpocock/skills", "improve-codebase-architecture"),
            ("obra/superpowers", "systematic-debugging"),
        ];
        for (package, skill) in defaults {
            let mut cmd = if command_exists("timeout") {
                let mut cmd = Command::new("timeout");
                cmd.arg("60").arg("npx");
                cmd
            } else {
                Command::new("npx")
            };
            let status = cmd
                .current_dir(&dir)
                .args([
                    "skills",
                    "add",
                    *package,
                    "--skill",
                    *skill,
                    "--agent",
                    "opencode",
                    "-y",
                ])
                .stdout(std::process::Stdio::null())
                .status();
            if !status.is_ok_and(|s| s.success()) {
                eprintln!(
                    "aviso: no se pudo instalar {} ({}); se usa la copia de la plantilla si existe.",
                    skill, package
                );
            }
        }
    } else {
        eprintln!("aviso: npx no está instalado; se usa la copia de la plantilla para las skills.");
    }
    let agents_skills = dir.join(".agents/skills");
    let opencode_dir = dir.join(".opencode");
    let opencode_skills = opencode_dir.join("skills");
    if agents_skills.is_dir() {
        let _ = std::fs::create_dir_all(&opencode_dir);
        if !opencode_skills.exists() {
            #[cfg(unix)]
            {
                if std::os::unix::fs::symlink("../.agents/skills", &opencode_skills).is_err() {
                    let _ = copy_dir_all(&agents_skills, &opencode_skills);
                }
            }
            #[cfg(not(unix))]
            {
                let _ = copy_dir_all(&agents_skills, &opencode_skills);
            }
        }
    }

    // git init
    let _ = Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .arg(&dir)
        .status();
    let _ = Command::new("git")
        .args(["-C"])
        .arg(&dir)
        .args(["add", "-A"])
        .status();
    let commit = Command::new("git")
        .args(["-C"])
        .arg(&dir)
        .args(["commit", "-q", "-m", "chore: estructura inicial (ai-new)"])
        .status();
    if !commit.is_ok_and(|s| s.success()) {
        eprintln!(
            "el proyecto quedó en ./{} pero el primer commit falló.",
            name
        );
        return Ok(());
    }
    println!("creado ./{}/ (rama main, primer commit hecho)", name);
    println!("skills: .agents/skills (find-skills instalado) + espejo .opencode/skills");

    if local {
        println!("modo --local: sin repo en GitHub.");
    } else if !command_exists("gh") {
        eprintln!("aviso: gh no está instalado; el proyecto queda solo local.");
    } else {
        let auth = Command::new("gh")
            .args(["auth", "status", "--hostname", "github.com"])
            .status();
        if !auth.is_ok_and(|s| s.success()) {
            eprintln!(
                "aviso: gh no tiene sesión; el proyecto queda solo local. Entra con: gh auth login"
            );
        } else {
            let visibility = if public { "--public" } else { "--private" };
            let source = format!("./{}", name);
            let mut args = vec![
                "repo",
                "create",
                name,
                visibility,
                "--source",
                &source,
                "--remote=origin",
                "--push",
            ];
            if let Some(d) = desc {
                args.push("--description");
                args.push(d);
            }
            let status = Command::new("gh").args(&args).status();
            if status.is_ok_and(|s| s.success()) {
                println!("repo creado en GitHub.");
            } else {
                eprintln!(
                    "aviso: no se pudo crear el repo en GitHub (¿ya existe '{}' en tu cuenta?); queda local.",
                    name
                );
            }
        }
    }
    println!();
    println!("siguiente:");
    println!("  1) llévalo a tu Project de Claude (modo PLANEAR) y pega aquí SPEC.md y tasks.md");
    println!("  2) cd {} && md ai code", name);
    Ok(())
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    Ok(())
}

fn replace_in_md(dir: &Path, from: &str, to: &str) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                replace_in_md(&path, from, to);
            } else if path.extension().and_then(|e| e.to_str()) == Some("md")
                && let Ok(text) = std::fs::read_to_string(&path)
            {
                let _ = std::fs::write(&path, text.replace(from, to));
            }
        }
    }
}

// ==============================================================================
// CODE (abre OpenCode con el modelo elegido)
// ==============================================================================
fn code(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if !command_exists("opencode") {
        eprintln!("opencode no está instalado (sudo pacman -S opencode).");
        return Ok(());
    }
    let env = load_models_env();
    let chat_port = env.chat_port;
    let chat_ctx = env.chat_ctx;
    // List local models
    let entries = scan_models();
    // Build labels for local models
    let mut labels: Vec<String> = Vec::new();
    let mut refs: Vec<String> = Vec::new();
    for entry in entries.iter().filter(|e| e.local) {
        labels.push(entry.label.clone());
        refs.push(entry.reference.clone());
    }
    // "free" option
    labels.push("☁ Otro proveedor…  (elegir dentro de OpenCode con /models)".to_owned());
    refs.push(String::new());

    if labels.len() <= 1 {
        // only free option? actually if no local models, labels has free only
    }

    let choice = match choose("OpenCode: ¿qué modelo usar?", &labels) {
        Some(c) => c,
        None => {
            println!("cancelado.");
            return Ok(());
        }
    };
    let idx = labels
        .iter()
        .position(|l| *l == choice)
        .unwrap_or(usize::MAX);
    if idx == usize::MAX || idx >= refs.len() {
        eprintln!("opción no reconocida.");
        return Ok(());
    }
    let reference = &refs[idx];
    if reference.is_empty() {
        // free
        // remove model if points to llamacpp
        let path = opencode_config_path();
        if path.exists() && command_exists("jq") {
            let filter =
                r#"if ((.model // "") | startswith("llamacpp/")) then del(.model) else . end"#;
            let out = Command::new("jq").arg(filter).arg(&path).output()?;
            if out.status.success() {
                let tmp = std::env::temp_dir().join("oc.json.tmp");
                let _ = std::fs::write(&tmp, &out.stdout);
                let _ = std::fs::rename(&tmp, &path);
            }
        }
        // reload
        let _ = Command::new("opencode").arg("reload").status();
        // exec opencode with args
        let status = Command::new("opencode").args(args).status()?;
        std::process::exit(status.code().unwrap_or(0));
    }

    // local model
    let name = model_alias(reference);
    // ensure server running
    if !is_running(&big_pid_file()) {
        let log = big_log();
        start_server(&ServerSpec {
            pid_file: &big_pid_file(),
            model: reference,
            port: chat_port,
            ctx: chat_ctx,
            threads: env.threads,
            log: &log,
            ngl: &env.ngl,
            jinja: true,
        })?;
        // wait
        let _ = wait_ready(chat_port, &big_pid_file());
    }
    // set local model in opencode (select=1)
    if command_exists("jq") {
        let path = opencode_config_path();
        let url = format!("http://127.0.0.1:{}/v1", chat_port);
        let filter = r#".["$schema"] //= "https://opencode.ai/config.json" | .model = ("llamacpp/" + $id) | .provider.llamacpp = {npm:"@ai-sdk/openai-compatible",name:"llama.cpp (local)",options:{baseURL:$url},models:{($id):{name:$id,limit:{context:$ctx,output:4096}}}}"#;
        let out = Command::new("jq")
            .args([
                "--arg",
                "id",
                &name,
                "--arg",
                "url",
                &url,
                "--argjson",
                "ctx",
                &chat_ctx.to_string(),
                filter,
                path.to_str().unwrap_or(""),
            ])
            .output()?;
        if out.status.success() {
            let tmp = std::env::temp_dir().join("oc.json.tmp");
            let _ = std::fs::write(&tmp, &out.stdout);
            let _ = std::fs::rename(&tmp, &path);
        }
    }
    let _ = Command::new("opencode").arg("reload").status();
    let status = Command::new("opencode").args(args).status()?;
    std::process::exit(status.code().unwrap_or(0));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_split_model_parts() {
        assert!(!is_non_first_split("modelo-Q4_K_M.gguf"));
        assert!(!is_non_first_split("modelo-00001-of-00005.gguf"));
        assert!(is_non_first_split("modelo-00002-of-00005.gguf"));
        assert!(!is_non_first_split("modelo.gguf"));
    }

    #[test]
    fn validates_last_selection() {
        assert!(last_entry_valid("-"));
        assert!(last_entry_valid("dl:repo/archivo.gguf"));
        assert!(!last_entry_valid("/ruta/que/no/existe.gguf"));
    }
}
