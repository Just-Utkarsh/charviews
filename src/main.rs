mod camera_list;
mod terminal_spawn;
mod tui;
mod viewer;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "charviews",
    version,
    about = "Charviews — TUI ASCII camera viewer (charviews)"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run ASCII viewer only (used when spawned in a new terminal)
    Viewer {
        /// Camera index (0, 1, …) or device path (e.g. /dev/video0)
        #[arg(long)]
        device: Option<String>,
        /// Target frames per second
        #[arg(long, default_value_t = 30)]
        fps: u32,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Viewer { device, fps }) => {
            viewer::run(device.as_deref(), fps)?;
        }
        None => {
            tui::run()?;
        }
    }

    Ok(())
}
