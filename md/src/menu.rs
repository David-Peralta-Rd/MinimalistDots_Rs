//! Subcomando `menu`: dmenu genérico (reemplaza a `wofi --dmenu`).
//!
//! Lee líneas por stdin, muestra el menú e imprime la elegida. Soporta
//! `--raw`, miniatura por TAB/`\x1f` y borrado opcional (`--delete-cmd`/`--wipe-cmd`).

use std::io::{Read, Write};
use std::process::Stdio;

use md_core::{MenuEntry, MenuMode, parse_dmenu_line};
use md_ui::{MenuConfig, MenuEvent, SlintHost, run_menu};

/// Argumentos del subcomando `menu`.
pub struct Args {
    pub name: String,
    pub prompt: String,
    pub raw: bool,
    pub delete_cmd: Option<String>,
    pub wipe_cmd: Option<String>,
}

/// Ejecuta el menú y escribe la elección en stdout.
pub fn run(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let mut entries = read_entries(args.raw)?;

    loop {
        let config = MenuConfig {
            app_id: format!("md.menu.{}", args.name),
            title: format!("Menú {}", args.name),
            prompt: args.prompt.clone(),
            accent: md_core::Color::new(0x7e, 0xc7, 0xa2),
            row_height: 40.0,
            width: 600.0,
            height: 400.0,
            can_delete: args.delete_cmd.is_some(),
            can_wipe: args.wipe_cmd.is_some(),
            submit_empty: !args.raw,
            wipe_label: None,
            confirm_wipe_label: None,
        };

        match run_menu(&SlintHost, &entries, MenuMode::Dmenu, &config)? {
            MenuEvent::Selected(index) => {
                if let Some(entry) = entries.get(index) {
                    println!("{}", entry.label);
                    return Ok(());
                }
            }
            MenuEvent::Typed(query) => {
                println!("{query}");
                return Ok(());
            }
            MenuEvent::Cancelled => {
                // Igual que el antiguo `md_menu`: salida 1 al cancelar con Esc.
                std::process::exit(1);
            }
            MenuEvent::Delete(index) => {
                if let Some(cmd) = &args.delete_cmd
                    && let Some(entry) = entries.get(index)
                {
                    let label = entry.label.clone();
                    entries.remove(index);
                    run_with_stdin(cmd, &label)?;
                }
            }
            MenuEvent::Wipe => {
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
