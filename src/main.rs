//! lightsnip — Wayland screenshot CLI (Milestone 1: capture).

mod capture;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use capture::{capture_to_png, default_output_path, CaptureMode};

#[derive(Debug, Parser)]
#[command(
    name = "lightsnip",
    about = "Wayland screenshot tool with annotation (capture CLI — Milestone 1)",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Capture a region (portal interactive picker, or slurp → grim).
    Region {
        /// PNG output path (default: /tmp/lightsnip-<unix>.png).
        #[arg(short, long, value_name = "PATH")]
        output: Option<PathBuf>,
    },
    /// Capture the full / active output.
    Full {
        /// PNG output path (default: /tmp/lightsnip-<unix>.png).
        #[arg(short, long, value_name = "PATH")]
        output: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let (mode, output) = match cli.command {
        Commands::Region { output } => (CaptureMode::Region, output.unwrap_or_else(default_output_path)),
        Commands::Full { output } => (CaptureMode::Full, output.unwrap_or_else(default_output_path)),
    };

    capture_to_png(mode, &output).await?;
    // Print absolute-ish path for scripts / ADHD quick feedback.
    println!("{}", output.display());
    Ok(())
}
