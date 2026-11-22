//! `md`: herramientas de escritorio de MinimalistDots (menús, OSD, utilidades).

use clap::{Parser, Subcommand};

mod ai;
mod binds;
mod clip;
mod clock;
mod emoji;
mod i18n;
mod language;
mod launcher;
mod menu;
mod notifd;
mod notify;
mod pomodoro;
mod process;
mod quickshell;
mod record;
mod shot;
mod util;
mod volume;
mod wallpaper;
mod welcome;

#[derive(Parser)]
#[command(
    name = "md",
    version,
    about = "Herramientas de escritorio de MinimalistDots"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Lanzador de aplicaciones (lista los `.desktop` instalados).
    Launcher,

    /// Historial del portapapeles (cliphist): elegir para copiar, borrar.
    Clip,

    /// Selector de emojis: elige uno y lo copia al portapapeles.
    Emoji,

    /// Visor de atajos de teclado agrupados por categoría.
    Binds,

    /// Gestor de procesos: lista y mata procesos del usuario.
    Process,

    /// Selector de fondos de pantalla (hyprpaper).
    Wallpaper {
        /// Imprime la lista en JSON (para QuickShell).
        #[arg(long)]
        json: bool,

        /// Aplica el fondo indicado (ruta) y sale.
        #[arg(long, value_name = "PATH")]
        apply: Option<String>,
    },

    /// Notificaciones: servidor residente, historial y control.
    Notify {
        #[command(subcommand)]
        command: notify::NotifyCommand,
    },

    /// Captura de pantalla con `grimblast` (área, OCR, QR…).
    Shot {
        /// `printscreen`, `snip`, `snapfreeze`, `monitor`, `scan` (OCR) o `qr`.
        mode: shot::Mode,
    },

    /// Graba la pantalla con `gpu-screen-recorder`.
    Record {
        /// `snip` graba un área, `monitor` la pantalla, `toggle` detiene.
        mode: record::Mode,

        /// Incluye el audio del sistema (por defecto no).
        #[arg(short, long)]
        audio: bool,
    },

    /// Cambia el volumen con `wpctl` y muestra un OSD transitorio.
    Volume {
        /// `up` sube, `down` baja, `mute` alterna el silencio.
        action: volume::Action,
    },

    /// Temporizador Pomodoro (concentración/descanso).
    Pomodoro {
        #[command(subcommand)]
        command: Option<pomodoro::PomodoroCommand>,
    },

    /// Reloj de escritorio persistente.
    Clock {
        /// Imprime un color de texto con contraste para el fondo actual (JSON).
        #[arg(long)]
        color: bool,
    },

    /// Cambia el idioma de los menús y avisos.
    Language,

    /// Pantalla de bienvenida (idioma, atajos, novedades, GitHub).
    Welcome {
        /// Alterna el estado de bienvenida y sale.
        #[arg(long)]
        toggle: bool,
    },

    /// IA local: menú de modelos, servidores `llama-server`, diagnóstico.
    Ai {
        #[command(subcommand)]
        command: ai::AiCommand,
    },

    /// Menú dmenu genérico: lee líneas por stdin e imprime la elegida.
    Menu {
        /// Nombre del menú (define el app-id `md.menu.<nombre>`).
        name: String,

        /// Texto que se muestra junto a la caja de búsqueda.
        #[arg(long)]
        prompt: Option<String>,

        /// La línea completa es la etiqueta (sin separar miniatura por TAB).
        #[arg(long)]
        raw: bool,

        /// Comando para borrar una entrada (recibe la línea por stdin).
        #[arg(long, value_name = "CMD")]
        delete_cmd: Option<String>,

        /// Comando para borrar todas las entradas.
        #[arg(long, value_name = "CMD")]
        wipe_cmd: Option<String>,
    },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Command::Launcher => launcher::run()?,
        Command::Clip => clip::run()?,
        Command::Emoji => emoji::run()?,
        Command::Binds => binds::run()?,
        Command::Process => process::run()?,
        Command::Wallpaper { json, apply } => {
            if json {
                wallpaper::list_json()?;
            } else if let Some(path) = apply {
                wallpaper::apply_path(std::path::Path::new(&path))?;
            } else {
                wallpaper::run()?;
            }
        }
        Command::Notify { command } => notify::run(command)?,
        Command::Shot { mode } => shot::run(mode)?,
        Command::Record { mode, audio } => record::run(mode, audio)?,
        Command::Volume { action } => volume::run(action)?,
        Command::Pomodoro { command } => {
            pomodoro::run(command.unwrap_or(pomodoro::PomodoroCommand::Toggle))?;
        }
        Command::Clock { color } => {
            if color {
                clock::print_color()?;
            } else {
                clock::run()?;
            }
        }
        Command::Language => language::run()?,
        Command::Welcome { toggle } => welcome::run(toggle)?,
        Command::Ai { command } => ai::run(command)?,
        Command::Menu {
            name,
            prompt,
            raw,
            delete_cmd,
            wipe_cmd,
        } => {
            let _ = name;
            menu::run(&menu::Args {
                prompt: prompt.unwrap_or_default(),
                raw,
                delete_cmd,
                wipe_cmd,
            })?;
        }
    }
    Ok(())
}
