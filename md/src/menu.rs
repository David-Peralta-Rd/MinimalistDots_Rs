//! Subcomando `menu`: dmenu genérico (reemplaza a `wofi --dmenu`).
//!
//! Lee líneas por stdin, muestra el menú e imprime la elegida. Soporta
//! `--raw`, miniatura por TAB/`\x1f` y borrado opcional (`--delete-cmd`/`--wipe-cmd`).

use std::io::{Read, Write};
use std::process::Stdio;

use md_core::{MenuEntry, parse_dmenu_line};

use crate::quickshell;

/// Argumentos del subcomando `menu`.
pub struct Args {
    pub prompt: String,
    pub raw: bool,
    pub delete_cmd: Option<String>,
    pub wipe_cmd: Option<String>,
}

/// Ejecuta el menú y escribe la elección en stdout.
#[derive(Debug)]
pub enum MenuResult {
    Selected(usize),
    Typed(String),
    Cancelled,
    Delete(usize),
    Wipe,
}

pub fn run_entries(
    entries: &[MenuEntry],
    prompt: &str,
    can_delete: bool,
    can_wipe: bool,
    submit_empty: bool,
    wipe_label: Option<&str>,
    confirm_wipe_label: Option<&str>,
) -> Result<MenuResult, Box<dyn std::error::Error>> {
    let json_entries: Vec<serde_json::Value> = entries
        .iter()
        .map(|e| serde_json::json!({ "label": e.label, "icon": crate::util::thumbnail(&e.icon) }))
        .collect();
    let entries_path = std::env::temp_dir().join("md-menu-entries.json");
    let result_path = std::env::temp_dir().join("md-menu-result.json");
    let _ = std::fs::remove_file(&result_path);
    let _ = std::fs::write(&entries_path, serde_json::to_string(&json_entries).unwrap_or_default());

    let can_delete_s = can_delete.to_string();
    let can_wipe_s = can_wipe.to_string();
    let submit_empty_s = submit_empty.to_string();
    let wipe_label = wipe_label.unwrap_or("🧹 Borrar todo");
    let confirm_wipe_label = confirm_wipe_label.unwrap_or("¿Seguro? Pulsa otra vez");

    let _ = quickshell::spawn_wait("Menu.qml", &[
        ("MD_MENU_ENTRIES", entries_path.to_str().unwrap_or("")),
        ("MD_MENU_RESULT", result_path.to_str().unwrap_or("")),
        ("MD_MENU_PROMPT", prompt),
        ("MD_MENU_CAN_DELETE", &can_delete_s),
        ("MD_MENU_CAN_WIPE", &can_wipe_s),
        ("MD_MENU_SUBMIT_EMPTY", &submit_empty_s),
        ("MD_MENU_WIPE_LABEL", wipe_label),
        ("MD_MENU_CONFIRM_WIPE_LABEL", confirm_wipe_label),
        ("QML_XHR_ALLOW_FILE_READ", "1"),
    ]);

    let raw = std::fs::read_to_string(&result_path).unwrap_or_default();
    let value: serde_json::Value = serde_json::from_str(&raw).unwrap_or(serde_json::json!({}));
    let _ = std::fs::remove_file(&entries_path);
    let _ = std::fs::remove_file(&result_path);

    let kind = value.get("type").and_then(|v| v.as_str()).unwrap_or("cancelled");
    match kind {
        "selected" => {
            let idx = value.get("index").and_then(|v| v.as_i64()).unwrap_or(-1);
            if idx >= 0 {
                Ok(MenuResult::Selected(idx as usize))
            } else {
                Ok(MenuResult::Cancelled)
            }
        }
        "typed" => {
            let q = value.get("query").and_then(|v| v.as_str()).unwrap_or("").to_owned();
            Ok(MenuResult::Typed(q))
        }
        "delete" => {
            let idx = value.get("index").and_then(|v| v.as_i64()).unwrap_or(-1);
            if idx >= 0 {
                Ok(MenuResult::Delete(idx as usize))
            } else {
                Ok(MenuResult::Cancelled)
            }
        }
        "wipe" => Ok(MenuResult::Wipe),
        _ => Ok(MenuResult::Cancelled),
    }
}

pub fn run(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let mut entries = read_entries(args.raw)?;

    loop {
        let result = run_entries(
            &entries,
            &args.prompt,
            args.delete_cmd.is_some(),
            args.wipe_cmd.is_some(),
            !args.raw,
            None,
            None,
        )?;

        match result {
            MenuResult::Selected(index) => {
                if let Some(entry) = entries.get(index) {
                    println!("{}", entry.label);
                    return Ok(());
                }
            }
            MenuResult::Typed(query) => {
                println!("{query}");
                return Ok(());
            }
            MenuResult::Cancelled => {
                std::process::exit(1);
            }
            MenuResult::Delete(index) => {
                if let Some(cmd) = &args.delete_cmd
                    && let Some(entry) = entries.get(index)
                {
                    let label = entry.label.clone();
                    entries.remove(index);
                    run_with_stdin(cmd, &label)?;
                }
            }
            MenuResult::Wipe => {
                if let Some(cmd) = &args.wipe_cmd {
                    entries.clear();
                    run_command(cmd)?;
                }
            }
        }
    }
}

/// Lee todas las líneas de stdin y las convierte en entradas de menú.
fn read_entries(raw: bool) -> Result<Vec<MenuEntry>, std::io::Error> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input)?;
    let mut entries: Vec<MenuEntry> = input
        .lines()
        .map(|line| parse_dmenu_line(line, raw))
        .collect();
    // La última línea vacía de `read_to_string` no debe crear una entrada.
    if entries
        .last()
        .is_some_and(|entry| entry.label.is_empty() && entry.icon.is_empty())
    {
        entries.pop();
    }
    Ok(entries)
}

/// Ejecuta `sh -c <cmd>` pasándole `input` por stdin (para borrar una entrada).
fn run_with_stdin(cmd: &str, input: &str) -> Result<(), std::io::Error> {
    let mut child = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input.as_bytes())?;
        stdin.write_all(b"\n")?;
    }
    child.wait()?;
    Ok(())
}

/// Ejecuta `sh -c <cmd>` sin entrada (para borrar todo).
fn run_command(cmd: &str) -> Result<(), std::io::Error> {
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()?;
    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "el comando terminó con código {status}"
        )))
    }
}
