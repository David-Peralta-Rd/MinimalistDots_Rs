//! Subcomando `clip`: historial de `cliphist` (elegir para copiar, borrar).
//!
//! Reemplaza a `md_cliphist_list` + `md_menu clipboard`. Versión sin miniaturas:
//! las entradas de imagen se muestran con su marcador `[[ binary data … ]]`.

use std::io::{Read, Write};
use std::process::{Command, Stdio};

use md_core::{MenuEntry, MenuMode, parse_clip_list};
use md_ui::{MenuConfig, MenuEvent, SlintHost, run_menu};

use crate::i18n::Texts;

/// Ejecuta el historial de portapapeles.
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let texts = Texts::load();
    let mut entries = list()?;

    loop {
        // `entries` y `items` comparten el mismo orden: el índice que devuelve
        // el menú se corresponde con la entrada del historial.
        let items: Vec<MenuEntry> = entries
            .iter()
            .map(|entry| MenuEntry::new(entry.preview.clone()))
            .collect();

        let config = MenuConfig {
            app_id: "md.clip".to_owned(),
            title: texts.t("escritorio.portapapeles"),
            prompt: "📋".to_owned(),
            accent: md_core::Color::new(0x7a, 0xa2, 0xf7), // azul
            row_height: 36.0,
            width: 700.0,
            height: 420.0,
            can_delete: true,
            can_wipe: true,
            submit_empty: false,
        };

        match run_menu(&SlintHost, &items, MenuMode::Dmenu, &config)? {
            MenuEvent::Selected(index) => {
                if let Some(entry) = entries.get(index) {
                    copy(&entry.line)?;
                }
                return Ok(());
            }
            MenuEvent::Delete(index) => {
                if let Some(entry) = entries.get(index) {
                    let line = entry.line.clone();
                    entries.remove(index);
                    delete(&line)?;
                }
            }
            MenuEvent::Wipe => {
                entries.clear();
                wipe()?;
            }
            MenuEvent::Cancelled | MenuEvent::Typed(_) => return Ok(()),
        }
    }
}

/// Lista el historial desde `cliphist list`.
fn list() -> Result<Vec<md_core::ClipEntry>, std::io::Error> {
    let output = Command::new("cliphist").arg("list").output()?;
    if !output.status.success() {
        return Err(std::io::Error::other("cliphist list terminó con error"));
    }
    Ok(parse_clip_list(&String::from_utf8_lossy(&output.stdout)))
}

/// Copia la entrada al portapapeles: `cliphist decode` → `wl-copy`.
fn copy(line: &str) -> Result<(), std::io::Error> {
    let decoded = decode(line)?;
    let mut wl = Command::new("wl-copy").stdin(Stdio::piped()).spawn()?;
    if let Some(mut stdin) = wl.stdin.take() {
        stdin.write_all(&decoded)?;
    }
    let status = wl.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("wl-copy terminó con error"))
    }
}

/// Decodifica una entrada (devuelve su contenido).
fn decode(line: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut clip = Command::new("cliphist")
        .arg("decode")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = clip.stdin.take() {
        stdin.write_all(line.as_bytes())?;
        stdin.write_all(b"\n")?;
    }
    // El `stdin` se cierra aquí (fin de `if let`); después se lee la salida.
    let mut decoded = Vec::new();
    if let Some(mut stdout) = clip.stdout.take() {
        stdout.read_to_end(&mut decoded)?;
    }
    let status = clip.wait()?;
    if status.success() {
        Ok(decoded)
    } else {
        Err(std::io::Error::other("cliphist decode terminó con error"))
    }
}

/// Borra una entrada.
fn delete(line: &str) -> Result<(), std::io::Error> {
    let mut clip = Command::new("cliphist")
        .arg("delete")
        .stdin(Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = clip.stdin.take() {
        stdin.write_all(line.as_bytes())?;
        stdin.write_all(b"\n")?;
    }
    let status = clip.wait()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("cliphist delete terminó con error"))
    }
}

/// Borra todo el historial.
fn wipe() -> Result<(), std::io::Error> {
    let status = Command::new("cliphist").arg("wipe").status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other("cliphist wipe terminó con error"))
    }
}
