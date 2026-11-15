//! `md`: herramientas de escritorio de MinimalistDots (menús, OSD, utilidades).

use clap::{Parser, Subcommand};

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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    match cli.command {
        Command::About => md_ui::about(&md_ui::SlintHost)?,
    }
    Ok(())
}
