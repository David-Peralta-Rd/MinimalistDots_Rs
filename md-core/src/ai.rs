//! Lógica pura de la IA local: catálogo de modelos descargables, nombres
//! "amigables", etiquetas de agente, avisos de RAM y parseo de `models.env`.
//!
//! Reemplaza la parte de datos de `llm_menu.sh`. Aquí no hay subprocesos ni
//! red: solo estructuras y funciones puras. La orquestación (menús, descargas,
//! `llama-server`) vive en `md`.

use std::collections::BTreeMap;
use std::path::Path;

/// Gigabyte en bytes (usado para convertir los tamaños aproximados de RAM).
pub const GIB: u64 = 1073741824;

// --- Categorías del menú (el orden es el orden en pantalla) ------------------
pub const CAT_CODE: &str = "Programación";
pub const CAT_THINK: &str = "Razonamiento / pensamiento";
pub const CAT_GEN: &str = "General y texto";
pub const CAT_LIGHT: &str = "Ligeros (poca RAM)";
pub const CAT_FIM: &str = "Autocompletado (FIM)";
pub const CAT_LOCAL: &str = "Otros descargados";

/// Categorías fijas en el orden en que se muestran (sin `Otros descargados`).
pub const CATEGORIES: &[&str] = &[CAT_CODE, CAT_THINK, CAT_GEN, CAT_LIGHT, CAT_FIM];

/// Modelo que el menú puede descargar solo (archivo exacto de Hugging Face).
///
/// Formato por línea: `Categoría|Etiqueta|repo|archivo.gguf|GB aprox.|nota`.
/// Para añadir uno nuevo basta con una línea más. El `doctor` comprueba que cada
/// archivo exista en Hugging Face.
#[derive(Debug, Clone, PartialEq)]
pub struct Preset {
    pub category: String,
    pub label: String,
    pub repo: String,
    pub file: String,
    pub size_gib: f64,
    pub note: String,
}

/// Líneas del catálogo (idénticas a las de `llm_menu.sh`).
const PRESET_LINES: &[&str] = &[
    // --- Programación (chat / edición / agentes) ---
    "Programación|Qwen2.5-Coder 7B Instruct Q4_K_M|bartowski/Qwen2.5-Coder-7B-Instruct-GGUF|Qwen2.5-Coder-7B-Instruct-Q4_K_M.gguf|4.7|el mejor equilibrio calidad/RAM",
    "Programación|DeepSeek-Coder 6.7B Instruct Q4_K_M|TheBloke/deepseek-coder-6.7B-instruct-GGUF|deepseek-coder-6.7b-instruct.Q4_K_M.gguf|4.1|clásico, 87 lenguajes",
    "Programación|Qwen2.5-Coder 3B Instruct Q4_K_M|bartowski/Qwen2.5-Coder-3B-Instruct-GGUF|Qwen2.5-Coder-3B-Instruct-Q4_K_M.gguf|1.9|rápido, tareas sencillas",
    "Programación|Qwen2.5-Coder 14B Instruct Q4_K_M|bartowski/Qwen2.5-Coder-14B-Instruct-GGUF|Qwen2.5-Coder-14B-Instruct-Q4_K_M.gguf|9.0|más listo, ~10 GB de RAM",
    "Programación|DeepSeek-Coder-V2-Lite 16B MoE Q4_K_M|bartowski/DeepSeek-Coder-V2-Lite-Instruct-GGUF|DeepSeek-Coder-V2-Lite-Instruct-Q4_K_M.gguf|10.4|MoE: rápido para su tamaño",
    "Programación|Qwen3-Coder 30B-A3B Q4_K_M|unsloth/Qwen3-Coder-30B-A3B-Instruct-GGUF|Qwen3-Coder-30B-A3B-Instruct-Q4_K_M.gguf|18.6|el más capaz; ~21 GB con contexto 16K",
    // --- Razonamiento / pensamiento ---
    "Razonamiento / pensamiento|Qwen3 8B Q4_K_M|unsloth/Qwen3-8B-GGUF|Qwen3-8B-Q4_K_M.gguf|5.0|razona + chat, muy versátil",
    "Razonamiento / pensamiento|DeepSeek-R1-Distill-Qwen 7B Q4_K_M|bartowski/DeepSeek-R1-Distill-Qwen-7B-GGUF|DeepSeek-R1-Distill-Qwen-7B-Q4_K_M.gguf|4.7|mates y lógica",
    "Razonamiento / pensamiento|Qwen3 14B Q4_K_M|unsloth/Qwen3-14B-GGUF|Qwen3-14B-Q4_K_M.gguf|9.0|razonamiento fuerte",
    "Razonamiento / pensamiento|DeepSeek-R1-Distill-Qwen 14B Q4_K_M|bartowski/DeepSeek-R1-Distill-Qwen-14B-GGUF|DeepSeek-R1-Distill-Qwen-14B-Q4_K_M.gguf|9.0|mates, lógica, análisis",
    "Razonamiento / pensamiento|Phi-4 14B Q4_K_M|bartowski/phi-4-GGUF|phi-4-Q4_K_M.gguf|9.1|muy bueno en STEM",
    "Razonamiento / pensamiento|gpt-oss 20B MXFP4|ggml-org/gpt-oss-20b-GGUF|gpt-oss-20b-mxfp4.gguf|12.1|MoE (3.6B activos), pesado en RAM",
    // --- General y texto ---
    "General y texto|Qwen2.5 7B Instruct Q4_K_M|bartowski/Qwen2.5-7B-Instruct-GGUF|Qwen2.5-7B-Instruct-Q4_K_M.gguf|4.7|buen español, multilingüe",
    "General y texto|Llama 3.1 8B Instruct Q4_K_M|bartowski/Meta-Llama-3.1-8B-Instruct-GGUF|Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf|4.9|generalista muy probado",
    "General y texto|Mistral 7B Instruct v0.3 Q4_K_M|bartowski/Mistral-7B-Instruct-v0.3-GGUF|Mistral-7B-Instruct-v0.3-Q4_K_M.gguf|4.4|redacción y resúmenes",
    "General y texto|Gemma 2 9B it Q4_K_M|bartowski/gemma-2-9b-it-GGUF|gemma-2-9b-it-Q4_K_M.gguf|5.8|texto muy natural",
    "General y texto|Gemma 3 12B it Q4_K_M|unsloth/gemma-3-12b-it-GGUF|gemma-3-12b-it-Q4_K_M.gguf|7.3|texto y multilingüe de calidad",
    "General y texto|Qwen2.5 14B Instruct Q4_K_M|bartowski/Qwen2.5-14B-Instruct-GGUF|Qwen2.5-14B-Instruct-Q4_K_M.gguf|9.0|generalista grande",
    // --- Ligeros (poca RAM) ---
    "Ligeros (poca RAM)|Qwen3 4B Instruct 2507 Q4_K_M|unsloth/Qwen3-4B-Instruct-2507-GGUF|Qwen3-4B-Instruct-2507-Q4_K_M.gguf|2.5|sorprendente para su tamaño",
    "Ligeros (poca RAM)|Gemma 3 4B it Q4_K_M|unsloth/gemma-3-4b-it-GGUF|gemma-3-4b-it-Q4_K_M.gguf|2.5|texto y multilingüe",
    "Ligeros (poca RAM)|Llama 3.2 3B Instruct Q4_K_M|bartowski/Llama-3.2-3B-Instruct-GGUF|Llama-3.2-3B-Instruct-Q4_K_M.gguf|2.0|rápido para tareas simples",
    "Ligeros (poca RAM)|Phi-3.5 mini 3.8B Q4_K_M|bartowski/Phi-3.5-mini-instruct-GGUF|Phi-3.5-mini-instruct-Q4_K_M.gguf|2.4|razona bien para ser pequeño",
    "Ligeros (poca RAM)|Llama 3.2 1B Instruct Q4_K_M|bartowski/Llama-3.2-1B-Instruct-GGUF|Llama-3.2-1B-Instruct-Q4_K_M.gguf|0.8|mínimo, solo tareas triviales",
    // --- Autocompletado (modelo base para :8081, NO son de chat) ---
    "Autocompletado (FIM)|Qwen2.5-Coder 1.5B Q8_0|ggml-org/Qwen2.5-Coder-1.5B-Q8_0-GGUF|qwen2.5-coder-1.5b-q8_0.gguf|1.7|recomendado: rápido y preciso",
    "Autocompletado (FIM)|Qwen2.5-Coder 0.5B Q8_0|ggml-org/Qwen2.5-Coder-0.5B-Q8_0-GGUF|qwen2.5-coder-0.5b-q8_0.gguf|0.7|ultraligero",
    "Autocompletado (FIM)|Qwen2.5-Coder 3B Q8_0|ggml-org/Qwen2.5-Coder-3B-Q8_0-GGUF|qwen2.5-coder-3b-q8_0.gguf|3.3|más preciso, más lento",
];

/// Analiza una línea del catálogo (`Categoría|Etiqueta|repo|archivo|GB|nota`).
/// Devuelve `None` si la línea no tiene los seis campos o el tamaño no es un
/// número.
pub fn parse_preset(line: &str) -> Option<Preset> {
    let mut parts = line.splitn(6, '|');
    let category = parts.next()?.to_owned();
    let label = parts.next()?.to_owned();
    let repo = parts.next()?.to_owned();
    let file = parts.next()?.to_owned();
    let size_gib = parts.next()?.parse::<f64>().ok()?;
    let note = parts.next()?.to_owned();
    Some(Preset {
        category,
        label,
        repo,
        file,
        size_gib,
        note,
    })
}

/// Catálogo completo de modelos descargables (omite líneas mal formadas).
pub fn presets() -> Vec<Preset> {
    PRESET_LINES
        .iter()
        .filter_map(|line| parse_preset(line))
        .collect()
}

/// Nombre "amigable" de una referencia a `.gguf`: basename sin `.gguf` y sin el
/// prefijo `…-GGUF_` (el alias que `llama-server` publica como id de modelo).
pub fn friendly_name(reference: &str) -> String {
    let base = Path::new(reference)
        .file_stem()
        .map(|stem| stem.to_string_lossy().into_owned())
        .unwrap_or_else(|| reference.to_owned());
    match base.rfind("-GGUF_") {
        Some(index) => base[index + "-GGUF_".len()..].to_owned(),
        None => base,
    }
}

/// Etiqueta orientativa de agente (tool calling). Por familia de modelo, no
/// medida en el equipo. Vacío = no se sabe. Recibe el nombre del archivo.
pub fn agent_tag(name: &str) -> &'static str {
    let lower = name.to_lowercase();
    if lower.starts_with("qwen2.5-coder-7b-instruct")
        || lower.starts_with("deepseek-coder")
        || lower.starts_with("deepseek-r1-distill")
    {
        " · NO sirve de agente"
    } else if lower.starts_with("qwen3-")
        || lower.starts_with("gpt-oss-")
        || lower.starts_with("meta-llama-3.1-")
        || lower.starts_with("mistral-7b-instruct-v0.3")
        || lower.starts_with("qwen2.5-7b-instruct")
        || lower.starts_with("qwen2.5-14b-instruct")
    {
        " · agente ✔"
    } else {
        ""
    }
}

/// Formatea bytes en gibibytes con un decimal (como `awk '%.1f'`).
pub fn gib(bytes: u64) -> String {
    format!("{:.1}", bytes as f64 / GIB as f64)
}

/// Aviso de RAM: un modelo que ocupa más del ~65 % de la RAM total del equipo
/// acaba en swap. Devuelve el sufijo a añadir a la etiqueta (o vacío).
pub fn ram_warn(bytes: u64, total_bytes: u64) -> &'static str {
    if bytes > total_bytes.saturating_mul(65) / 100 {
        " ⚠ pesado para tu RAM"
    } else {
        ""
    }
}

// --- Referencias `dl:repo/archivo.gguf` --------------------------------------

/// Archivo (solo el nombre) de una referencia `dl:repo/archivo.gguf`.
pub fn dl_file(reference: &str) -> String {
    let rest = reference.strip_prefix("dl:").unwrap_or(reference);
    Path::new(rest)
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| rest.to_owned())
}

/// Repositorio (`repo`) de una referencia `dl:repo/archivo.gguf`.
pub fn dl_repo(reference: &str) -> String {
    let rest = reference.strip_prefix("dl:").unwrap_or(reference);
    Path::new(rest)
        .parent()
        .map(|parent| parent.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Bytes aproximados de una referencia `dl:…` según el catálogo (`0` si no está).
pub fn preset_bytes(reference: &str) -> u64 {
    for preset in presets() {
        let candidate = format!("dl:{}/{}", preset.repo, preset.file);
        if candidate == reference {
            return (preset.size_gib * GIB as f64) as u64;
        }
    }
    0
}

// --- `models.env` (configuración compartida con el módulo de IA) -------------

/// Configuración de IA ya resuelta (`models.env` + valores por defecto).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelsEnv {
    pub chat_model: String,
    pub chat_port: u16,
    pub chat_ctx: u32,
    pub threads: u32,
    pub fim_model: String,
    pub fim_port: u16,
    pub fim_ctx: u32,
    pub fim_threads: u32,
    /// Capas a la GPU: `"0"` = solo CPU; `""` = que decida llama.cpp.
    pub ngl: String,
}

impl Default for ModelsEnv {
    fn default() -> Self {
        Self {
            chat_model: String::new(),
            chat_port: 8080,
            chat_ctx: 16384,
            threads: 6,
            fim_model: String::new(),
            fim_port: 8081,
            fim_ctx: 4096,
            fim_threads: 2,
            ngl: "0".to_owned(),
        }
    }
}

/// Expande `$NOMBRE` y `${NOMBRE}` en `text` con los valores de `get`. Las
/// variables sin definir quedan tal cual.
pub fn expand_env_vars(text: &str, get: &impl Fn(&str) -> Option<String>) -> String {
    let bytes = text.as_bytes();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'$' && i + 1 < bytes.len() {
            if bytes[i + 1] == b'{' {
                // ${NOMBRE}
                if let Some(end) = text[i + 2..].find('}') {
                    let name = &text[i + 2..i + 2 + end];
                    if let Some(value) = get(name) {
                        out.push_str(&value);
                    } else {
                        out.push_str(&text[i..i + 2 + end + 1]);
                    }
                    i += 2 + end + 1;
                    continue;
                }
            } else {
                // $NOMBRE (letras, dígitos y guion bajo)
                let start = i + 1;
                let mut end = start;
                while end < bytes.len()
                    && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'_')
                {
                    end += 1;
                }
                if end > start {
                    let name = &text[start..end];
                    if let Some(value) = get(name) {
                        out.push_str(&value);
                    } else {
                        out.push_str(&text[i..end]);
                    }
                    i = end;
                    continue;
                }
            }
        }
        let ch = text[i..].chars().next().unwrap_or('\u{fffd}');
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Quita las comillas dobles o simples que rodean un valor de `models.env`.
fn strip_quotes(value: &str) -> &str {
    let value = value.trim();
    if value.len() >= 2 {
        let first = value.as_bytes()[0];
        let last = value.as_bytes()[value.len() - 1];
        if (first == b'"' && last == b'"') || (first == b'\'' && last == b'\'') {
            return &value[1..value.len() - 1];
        }
    }
    value
}

/// Analiza un `models.env` (sintaxis bash `CLAVE=valor`, comentarios con `#`,
/// comillas opcionales) y devuelve la configuración con variables expandidas y
/// valores por defecto. Las claves desconocidas se ignoran.
pub fn parse_models_env(text: &str, get: &impl Fn(&str) -> Option<String>) -> ModelsEnv {
    let mut vars: BTreeMap<String, String> = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let value = strip_quotes(value);
        vars.insert(key.trim().to_owned(), expand_env_vars(value, get));
    }

    let get_int = |name: &str| vars.get(name).and_then(|value| value.parse::<u32>().ok());
    let get_port = |name: &str| vars.get(name).and_then(|value| value.parse::<u16>().ok());

    ModelsEnv {
        chat_model: vars.get("CHAT_MODEL").cloned().unwrap_or_default(),
        chat_port: get_port("CHAT_PORT").unwrap_or(8080),
        chat_ctx: get_int("CHAT_CTX").unwrap_or(16384),
        threads: get_int("THREADS").unwrap_or(6),
        fim_model: vars.get("FIM_MODEL").cloned().unwrap_or_default(),
        fim_port: get_port("FIM_PORT").unwrap_or(8081),
        fim_ctx: get_int("FIM_CTX").unwrap_or(4096),
        fim_threads: get_int("FIM_THREADS").unwrap_or(2),
        ngl: vars.get("NGL").cloned().unwrap_or_else(|| "0".to_owned()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_every_shipped_preset() {
        let parsed = presets();
        assert_eq!(parsed.len(), PRESET_LINES.len());
        for preset in &parsed {
            assert!(!preset.category.is_empty());
            assert!(!preset.label.is_empty());
            assert!(!preset.repo.is_empty());
            assert!(!preset.file.is_empty());
            assert!(preset.size_gib > 0.0);
            assert!(!preset.note.is_empty());
        }
    }

    #[test]
    fn friendly_name_strips_extension_and_gguf_prefix() {
        assert_eq!(
            friendly_name("/ruta/Qwen2.5-Coder-7B-Instruct-Q4_K_M.gguf"),
            "Qwen2.5-Coder-7B-Instruct-Q4_K_M"
        );
        assert_eq!(friendly_name("algo-GGUF_Base-Q4_0.gguf"), "Base-Q4_0");
        assert_eq!(friendly_name("sin_extension"), "sin_extension");
    }

    #[test]
    fn agent_tag_classifies_families() {
        assert_eq!(
            agent_tag("Qwen2.5-Coder-7B-Instruct-Q4_K_M.gguf"),
            " · NO sirve de agente"
        );
        assert_eq!(
            agent_tag("deepseek-r1-distill-qwen-7b.gguf"),
            " · NO sirve de agente"
        );
        assert_eq!(agent_tag("Qwen3-8B-Q4_K_M.gguf"), " · agente ✔");
        assert_eq!(
            agent_tag("Meta-Llama-3.1-8B-Instruct-Q4_K_M.gguf"),
            " · agente ✔"
        );
        assert_eq!(
            agent_tag("mistral-7b-instruct-v0.3-Q4_K_M.gguf"),
            " · agente ✔"
        );
        assert_eq!(agent_tag("modelo-desconocido.gguf"), "");
    }

    #[test]
    fn gib_formats_one_decimal() {
        assert_eq!(gib(GIB), "1.0");
        assert_eq!(gib(5046586572), "4.7");
        assert_eq!(gib(0), "0.0");
    }

    #[test]
    fn ram_warn_triggers_above_65_percent() {
        assert_eq!(ram_warn(7 * GIB, 10 * GIB), " ⚠ pesado para tu RAM");
        assert_eq!(ram_warn(6 * GIB, 10 * GIB), "");
    }

    #[test]
    fn dl_ref_helpers_split_repo_and_file() {
        let reference = "dl:unsloth/Qwen3-8B-GGUF/Qwen3-8B-Q4_K_M.gguf";
        assert_eq!(dl_file(reference), "Qwen3-8B-Q4_K_M.gguf");
        assert_eq!(dl_repo(reference), "unsloth/Qwen3-8B-GGUF");
    }

    #[test]
    fn preset_bytes_looks_up_the_catalog() {
        let reference = "dl:unsloth/Qwen3-8B-GGUF/Qwen3-8B-Q4_K_M.gguf";
        assert_eq!(preset_bytes(reference), (5.0 * GIB as f64) as u64);
        assert_eq!(preset_bytes("dl:no/existe.gguf"), 0);
    }

    #[test]
    fn expand_env_vars_handles_home_and_braces() {
        let get = |name: &str| match name {
            "HOME" => Some("/home/ana".to_owned()),
            "XDG" => Some("/xdg".to_owned()),
            _ => None,
        };
        assert_eq!(
            expand_env_vars("$HOME/models/a.gguf", &get),
            "/home/ana/models/a.gguf"
        );
        assert_eq!(expand_env_vars("${XDG}/ai", &get), "/xdg/ai");
        // Variable sin definir: queda tal cual.
        assert_eq!(expand_env_vars("$NOPE/x", &get), "$NOPE/x");
        // Texto sin variables: intacto.
        assert_eq!(expand_env_vars("hola", &get), "hola");
    }

    #[test]
    fn parse_models_env_applies_defaults_and_overrides() {
        let get = |name: &str| match name {
            "HOME" => Some("/home/ana".to_owned()),
            _ => None,
        };
        let text = "\
# comentario
CHAT_MODEL=\"$HOME/models/chat.gguf\"
CHAT_PORT=9090
FIM_MODEL='/home/ana/models/fim.gguf'
";
        let env = parse_models_env(text, &get);
        assert_eq!(env.chat_model, "/home/ana/models/chat.gguf");
        assert_eq!(env.chat_port, 9090);
        assert_eq!(env.fim_model, "/home/ana/models/fim.gguf");
        // Sin definir: valores por defecto.
        assert_eq!(env.chat_ctx, 16384);
        assert_eq!(env.threads, 6);
        assert_eq!(env.fim_port, 8081);
        assert_eq!(env.fim_ctx, 4096);
        assert_eq!(env.fim_threads, 2);
        assert_eq!(env.ngl, "0");
    }

    #[test]
    fn parse_models_env_ignores_bad_lines_and_values() {
        let get = |_: &str| None;
        let text = "esto no es una asignacion\nCHAT_CTX=no-numero\nCHAT_PORT=999999\n";
        let env = parse_models_env(text, &get);
        // Los valores inválidos caen a los valores por defecto.
        assert_eq!(env.chat_ctx, 16384);
        assert_eq!(env.chat_port, 8080);
    }
}
