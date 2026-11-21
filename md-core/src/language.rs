//! Catálogos de idioma en TOML (`lang/<BCP47>.toml`).
//!
//! Un catálogo es un archivo TOML con una sección `[meta]` (`codigo`, `nombre`,
//! `base` opcional) y el resto de secciones con claves de texto en español.
//! La cadena de respaldo es: código → base → `es` → la propia clave.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Metadatos obligatorios de un catálogo de idioma.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Meta {
    pub codigo: String,
    pub nombre: String,
    #[serde(default)]
    pub base: Option<String>,
}

/// Un catálogo de idioma completo: metadatos + secciones de traducciones.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Catalog {
    pub meta: Meta,
    #[serde(flatten)]
    pub sections: BTreeMap<String, BTreeMap<String, String>>,
}

impl std::str::FromStr for Catalog {
    type Err = CatalogError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        toml::from_str(text).map_err(CatalogError::from)
    }
}

impl Catalog {
    /// Carga un catálogo desde un archivo.
    pub fn load(path: &Path) -> Result<Self, CatalogError> {
        let text = fs::read_to_string(path)?;
        text.parse()
    }

    /// Devuelve el texto de una clave, si existe en este catálogo.
    pub fn get(&self, section: &str, key: &str) -> Option<&str> {
        self.sections
            .get(section)
            .and_then(|map| map.get(key))
            .map(String::as_str)
    }
}

/// Error al leer o interpretar un catálogo.
#[derive(Debug, thiserror::Error)]
pub enum CatalogError {
    #[error("no se pudo leer el catálogo: {0}")]
    Io(#[from] std::io::Error),
    #[error("catálogo inválido: {0}")]
    Toml(#[from] toml::de::Error),
}

/// Sufijo de idioma base de un código BCP 47 (`es-CO` → `es`).
pub fn base_of(codigo: &str) -> &str {
    match codigo.split_once('-') {
        Some((base, _)) => base,
        None => codigo,
    }
}

/// Lee el idioma guardado en un `settings.toml` (`idioma = "es"`), o `None` si
/// falta la clave o el contenido no es un TOML válido. Lo usa `md` al arrancar
/// para elegir el catálogo (ADR-0004: el cambio vale al reiniciar).
pub fn read_settings_language(text: &str) -> Option<String> {
    let value: toml::Value = toml::from_str(text).ok()?;
    value
        .get("idioma")
        .and_then(toml::Value::as_str)
        .map(str::to_owned)
}

/// Sustituye los marcadores `{nombre}` de `text` con sus valores (v1: sin
/// plurales ni género; ADR-0004 los deja a Fluent si hicieran falta).
pub fn fill(text: &str, values: &[(&str, &str)]) -> String {
    let mut result = text.to_owned();
    for (name, value) in values {
        result = result.replace(&format!("{{{name}}}"), value);
    }
    result
}

/// Traduce claves aplicando la cadena de respaldo código → base → `es` → clave.
#[derive(Debug, Clone, Default)]
pub struct Translator {
    chain: Vec<Catalog>,
}

impl Translator {
    /// Devuelve el texto de `clave` (formato `seccion.clave`) o la propia clave
    /// si no hay traducción en ningún catálogo de la cadena.
    pub fn t(&self, key: &str) -> String {
        let (section, name) = key.split_once('.').unwrap_or((key, ""));
        for catalog in &self.chain {
            if let Some(value) = catalog.get(section, name) {
                return value.to_owned();
            }
        }
        key.to_owned()
    }
}

/// Carga la cadena de catálogos para `codigo` desde `dir`, con caché en disco
/// invalidada por los mtimes de los archivos `.toml` del directorio.
pub fn load_chain(dir: &Path, codigo: &str) -> Result<Translator, CatalogError> {
    let fingerprint = lang_cache_fingerprint(dir);
    if let Some(path) = lang_cache_path(codigo)
        && let Ok(text) = std::fs::read_to_string(&path)
        && let Ok(cache) = serde_json::from_str::<LangCache>(&text)
        && cache.fingerprint == fingerprint
    {
        return Ok(Translator { chain: cache.chain });
    }

    let translator = load_chain_uncached(dir, codigo)?;

    if let Some(path) = lang_cache_path(codigo) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let cache = LangCache {
            fingerprint,
            chain: translator.chain.clone(),
        };
        if let Ok(text) = serde_json::to_string(&cache) {
            let tmp = path.with_extension("json.tmp");
            if std::fs::write(&tmp, &text).is_ok() {
                let _ = std::fs::rename(&tmp, path);
            }
        }
    }

    Ok(translator)
}

/// Implementación sin caché de `load_chain`.
fn load_chain_uncached(dir: &Path, codigo: &str) -> Result<Translator, CatalogError> {
    let mut chain = Vec::new();

    let primary = dir.join(format!("{codigo}.toml"));
    if primary.exists() {
        let catalog = Catalog::load(&primary)?;
        let base = catalog
            .meta
            .base
            .clone()
            .unwrap_or_else(|| base_of(codigo).to_owned());
        chain.push(catalog);

        if base != codigo {
            let base_path = dir.join(format!("{base}.toml"));
            if base_path.exists() {
                chain.push(Catalog::load(&base_path)?);
            }
        }
    }

    let has_es = chain.iter().any(|catalog| catalog.meta.codigo == "es");
    if codigo != "es" && !has_es {
        let es_path = dir.join("es.toml");
        if es_path.exists() {
            chain.push(Catalog::load(&es_path)?);
        }
    }

    Ok(Translator { chain })
}

#[derive(serde::Serialize, serde::Deserialize)]
struct LangCache {
    fingerprint: u64,
    chain: Vec<Catalog>,
}

fn lang_cache_dir() -> Option<std::path::PathBuf> {
    let base = std::env::var_os("XDG_CACHE_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| std::path::PathBuf::from(home).join(".cache"))
        })?;
    Some(base.join("md").join("lang"))
}

fn lang_cache_path(codigo: &str) -> Option<std::path::PathBuf> {
    let safe = codigo.replace(['/', '\\'], "_");
    lang_cache_dir().map(|dir| dir.join(format!("{safe}.json")))
}

fn lang_cache_fingerprint(dir: &Path) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    if let Ok(read_dir) = std::fs::read_dir(dir) {
        let mut paths: Vec<_> = read_dir.flatten().map(|e| e.path()).collect();
        paths.sort();
        for path in paths {
            path.hash(&mut hasher);
            if let Ok(meta) = std::fs::metadata(&path)
                && let Ok(mtime) = meta.modified()
            {
                mtime.hash(&mut hasher);
            }
        }
    }
    hasher.finish()
}

/// Lista los idiomas disponibles en `dir`, como pares `(codigo, nombre)` ordenados.
pub fn available_languages(dir: &Path) -> Result<Vec<(String, String)>, CatalogError> {
    let mut languages = Vec::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("toml") {
            continue;
        }
        let catalog = Catalog::load(&path)?;
        languages.push((catalog.meta.codigo, catalog.meta.nombre));
    }
    languages.sort();
    Ok(languages)
}

/// Resultado de validar todos los catálogos de un directorio.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CatalogReport {
    /// Códigos de idioma encontrados.
    pub catalogs: Vec<String>,
    /// Problemas detectados (mensajes legibles).
    pub errors: Vec<String>,
}

impl CatalogReport {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }
}

/// Conjunto de claves `seccion.clave` de un catálogo.
fn flat_keys(catalog: &Catalog) -> BTreeSet<String> {
    let mut keys = BTreeSet::new();
    for (section, map) in &catalog.sections {
        for key in map.keys() {
            keys.insert(format!("{section}.{key}"));
        }
    }
    keys
}

/// Marcadores `{nombre}` presentes en un texto.
fn markers(text: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut rest = text;
    while let Some(start) = rest.find('{') {
        rest = &rest[start + 1..];
        match rest.find('}') {
            Some(end) => {
                let name = &rest[..end];
                if !name.is_empty() {
                    found.insert(name.to_owned());
                }
                rest = &rest[end + 1..];
            }
            None => break,
        }
    }
    found
}

/// Valida los catálogos TOML de `dir` tomando `es` como referencia.
///
/// Comprueba `[meta]`, claves faltantes o sobrantes respecto a `es` y que los
/// marcadores `{nombre}` de cada clave coincidan entre idiomas.
pub fn validate_language_catalogs(dir: &Path) -> Result<CatalogReport, CatalogError> {
    let mut report = CatalogReport::default();
    let mut catalogs = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("toml") {
            continue;
        }
        match Catalog::load(&path) {
            Ok(catalog) => {
                if catalog.meta.codigo.is_empty() {
                    report
                        .errors
                        .push(format!("{}: [meta].codigo vacío", path.display()));
                }
                if catalog.meta.nombre.is_empty() {
                    report
                        .errors
                        .push(format!("{}: [meta].nombre vacío", path.display()));
                }
                report.catalogs.push(catalog.meta.codigo.clone());
                catalogs.push(catalog);
            }
            Err(error) => {
                report.errors.push(format!("{}: {error}", path.display()));
            }
        }
    }

    let reference = catalogs.iter().find(|catalog| catalog.meta.codigo == "es");
    match reference {
        None => report
            .errors
            .push("no existe el catálogo de referencia 'es.toml'".to_owned()),
        Some(reference) => {
            let reference_keys = flat_keys(reference);
            for catalog in catalogs
                .iter()
                .filter(|catalog| catalog.meta.codigo != "es")
            {
                let keys = flat_keys(catalog);
                for missing in reference_keys.difference(&keys) {
                    report
                        .errors
                        .push(format!("{}: falta la clave {missing}", catalog.meta.codigo));
                }
                for extra in keys.difference(&reference_keys) {
                    report
                        .errors
                        .push(format!("{}: clave sobrante {extra}", catalog.meta.codigo));
                }
                for key in reference_keys.intersection(&keys) {
                    let (section, name) = key.split_once('.').unwrap_or((key, ""));
                    let reference_value = reference.get(section, name);
                    let value = catalog.get(section, name);
                    if let (Some(a), Some(b)) = (reference_value, value)
                        && markers(a) != markers(b)
                    {
                        report.errors.push(format!(
                            "{}: los marcadores de {key} no coinciden con 'es'",
                            catalog.meta.codigo
                        ));
                    }
                }
            }
        }
    }

    report.catalogs.sort();
    report.errors.sort();
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::TempDir;

    const ES: &str = "\
[meta]
codigo = \"es\"
nombre = \"Español\"

[comun]
bienvenido = \"Bienvenido a MinimalistDots\"
";

    const EN: &str = "\
[meta]
codigo = \"en\"
nombre = \"English\"

[comun]
bienvenido = \"Welcome to MinimalistDots\"
";

    #[test]
    fn parses_meta_and_sections() -> Result<(), CatalogError> {
        let catalog: Catalog = ES.parse()?;
        assert_eq!(catalog.meta.codigo, "es");
        assert_eq!(catalog.meta.nombre, "Español");
        assert_eq!(
            catalog.get("comun", "bienvenido"),
            Some("Bienvenido a MinimalistDots")
        );
        assert_eq!(catalog.get("comun", "inexistente"), None);
        Ok(())
    }

    #[test]
    fn lists_available_languages_sorted() -> Result<(), CatalogError> {
        let dir = TempDir::new("lang")?;
        dir.write("es.toml", ES)?;
        dir.write("en.toml", EN)?;
        let languages = available_languages(dir.path())?;
        assert_eq!(
            languages,
            vec![
                ("en".to_owned(), "English".to_owned()),
                ("es".to_owned(), "Español".to_owned()),
            ]
        );
        Ok(())
    }

    #[test]
    fn translates_with_fallback_chain() -> Result<(), CatalogError> {
        let dir = TempDir::new("chain")?;
        dir.write("es.toml", ES)?;
        dir.write(
            "es-CO.toml",
            "[meta]\ncodigo = \"es-CO\"\nnombre = \"Español (Colombia)\"\n\n[comun]\nsaludo = \"Hola parcero\"\n",
        )?;

        // es-CO define `saludo`; `bienvenido` cae a `es`.
        let translator = load_chain(dir.path(), "es-CO")?;
        assert_eq!(translator.t("comun.saludo"), "Hola parcero");
        assert_eq!(
            translator.t("comun.bienvenido"),
            "Bienvenido a MinimalistDots"
        );
        // Sin traducción en ningún catálogo: devuelve la clave.
        assert_eq!(translator.t("comun.no_existe"), "comun.no_existe");
        Ok(())
    }

    #[test]
    fn extracts_markers() {
        let set = markers("Hola {nombre}, tu correo es {correo}");
        assert!(set.contains("nombre"));
        assert!(set.contains("correo"));
        assert!(!set.contains("nombre}"));
    }

    #[test]
    fn reads_language_from_settings() {
        assert_eq!(
            read_settings_language("idioma = \"es\"\n"),
            Some("es".to_owned())
        );
        assert_eq!(
            read_settings_language("# comentario\nidioma = \"en\"\n"),
            Some("en".to_owned())
        );
        // Sin clave `idioma` o sin TOML válido: `None`.
        assert_eq!(read_settings_language("otra = \"cosa\"\n"), None);
        assert_eq!(read_settings_language("no es toml"), None);
    }

    #[test]
    fn fills_markers() {
        assert_eq!(fill("Hola {nombre}", &[("nombre", "Ana")]), "Hola Ana");
        assert_eq!(
            fill("Guardada en {ruta}", &[("ruta", "/tmp/a.png")]),
            "Guardada en /tmp/a.png"
        );
        // Marcadores sin valor quedan tal cual; valores sin marcador se ignoran.
        assert_eq!(fill("sin marcadores", &[("x", "y")]), "sin marcadores");
    }

    #[test]
    fn shipped_catalogs_are_valid() -> Result<(), CatalogError> {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../lang");
        let report = validate_language_catalogs(&dir)?;
        assert!(
            report.is_valid(),
            "catálogos de `lang/` inválidos: {:?}",
            report.errors
        );
        Ok(())
    }

    #[test]
    fn validates_catalogs() -> Result<(), CatalogError> {
        let dir = TempDir::new("validate")?;
        dir.write("es.toml", ES)?;
        dir.write("en.toml", EN)?;
        // fr: falta `bienvenido`.
        dir.write(
            "fr.toml",
            "[meta]\ncodigo = \"fr\"\nnombre = \"Français\"\n\n[comun]\notra = \"Autre\"\n",
        )?;

        let report = validate_language_catalogs(dir.path())?;
        assert!(!report.is_valid());
        assert!(
            report
                .errors
                .iter()
                .any(|e| e.contains("falta la clave comun.bienvenido"))
        );
        assert!(
            report
                .errors
                .iter()
                .any(|e| e.contains("clave sobrante comun.otra"))
        );
        Ok(())
    }

    #[test]
    fn detects_marker_mismatch() -> Result<(), CatalogError> {
        let dir = TempDir::new("markers")?;
        dir.write(
            "es.toml",
            "[meta]\ncodigo = \"es\"\nnombre = \"Español\"\n\n[comun]\nsaludo = \"Hola {nombre}\"\n",
        )?;
        dir.write(
            "en.toml",
            "[meta]\ncodigo = \"en\"\nnombre = \"English\"\n\n[comun]\nsaludo = \"Hi {name}\"\n",
        )?;
        let report = validate_language_catalogs(dir.path())?;
        assert!(
            report
                .errors
                .iter()
                .any(|e| e.contains("los marcadores de comun.saludo"))
        );
        Ok(())
    }
}
