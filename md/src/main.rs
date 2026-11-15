//! `md`: herramientas de escritorio de MinimalistDots (menús, OSD, utilidades).

use clap::{Parser, Subcommand};

mod binds;
mod launcher;
mod menu;
mod process;
mod volume;

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
    /// Diálogo "Acerca de" (AboutSlint).
    About,

    /// Lanzador de aplicaciones (lista los `.desktop` instalados).
    Launcher,

    /// Visor de atajos de teclado agrupados por categoría.
    Binds,

    /// Gestor de procesos: lista y mata procesos del usuario.
    Process,

    /// Cambia el volumen con `wpctl` y muestra un OSD transitorio.
    Volume {
        /// `up` sube, `down` baja, `mute` alterna el silencio.
        action: volume::Action,
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
        Command::About => md_ui::about(&md_ui::SlintHost)?,
        Command::Launcher => launcher::run()?,
        Command::Binds => binds::run()?,
        Command::Process => process::run()?,
        Command::Volume { action } => volume::run(action)?,
        Command::Menu {
            name,
            prompt,
            raw,
            delete_cmd,
            wipe_cmd,
        } => {
            menu::run(&menu::Args {
                name,
                prompt: prompt.unwrap_or_default(),
                raw,
                delete_cmd,
                wipe_cmd,
            })?;
        }
    }
    Ok(())
}
