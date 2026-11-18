use std::fs;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR")?);
    let palette_path = manifest_dir.join("../theme/palette.toml");
    let palette_text = fs::read_to_string(&palette_path)?;
    let out_slint = manifest_dir.join("ui/colors.slint");
    let palette: toml::Table = palette_text.parse()?;

    // Mapeo clave TOML (español) -> identificador Slint (inglés).
    let mapping: &[(&str, &str)] = &[
        ("fondo", "fondo"),
        ("superficie", "superficie"),
        ("seleccion", "seleccion"),
        ("borde_inactivo", "borde_inactivo"),
        ("borde_activo", "borde_activo"),
        ("texto", "texto"),
        ("texto_suave", "texto_suave"),
        ("acento_azul", "acento_azul"),
        ("acento_verde", "acento_verde"),
        ("acento_rojo", "acento_rojo"),
        ("acento_amarillo", "acento_amarillo"),
        ("acento_magenta", "acento_magenta"),
        ("acento_cian", "acento_cian"),
        ("hover", "hover"),
        ("tarjeta", "tarjeta"),
        ("texto_pausado", "texto_pausado"),
        ("texto_contraste_claro", "contraste_claro"),
        ("texto_contraste_oscuro", "contraste_oscuro"),
        ("overlay", "overlay"),
    ];

    let mut lines = Vec::new();
    lines.push("// Generado desde theme/palette.toml. NO EDITES A MANO.".to_string());
    lines.push("export global Colors {".to_string());
    for &(toml_key, slint_key) in mapping {
        let value = palette
            .get(toml_key)
            .and_then(|v| v.as_str())
            .ok_or_else(|| format!("falta clave {toml_key} en palette.toml"))?;
        lines.push(format!("    out property <color> {slint_key}: {value};"));
    }
    lines.push("}".to_string());
    fs::write(&out_slint, lines.join("\n") + "\n")?;

    // Recompilar si cambia la paleta.
    println!("cargo:rerun-if-changed={}", palette_path.display());
    println!("cargo:rerun-if-changed={}", manifest_dir.join("ui/app.slint").display());

    slint_build::compile(manifest_dir.join("ui/app.slint").to_str().unwrap())?;
    Ok(())
}