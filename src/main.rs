//! lightsnip — Wayland screenshot CLI (Milestone 2: save + clipboard).

mod capture;
mod output;

use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use capture::{capture_to_png, CaptureMode};
use output::{copy_png_to_clipboard, resolve_output_path};

#[derive(Debug, Parser)]
#[command(
    name = "lightsnip",
    about = "Wayland screenshot tool with annotation (save + clipboard — Milestone 2)",
    version
)]
struct Cli {
    /// PNG output path (overrides --save-dir / default dir).
    #[arg(short, long, value_name = "PATH", global = true)]
    output: Option<PathBuf>,

    /// Directory for timestamped PNGs when --output is omitted
    /// (default: $LIGHTSNIP_SAVE_DIR or ~/Pictures/lightsnip).
    #[arg(long, value_name = "DIR", global = true)]
    save_dir: Option<PathBuf>,

    /// Copy PNG to the Wayland clipboard via wl-copy (default when neither flag is set).
    #[arg(long, global = true, action = clap::ArgAction::SetTrue, conflicts_with = "no_clipboard")]
    clipboard: bool,

    /// Do not copy the PNG to the clipboard.
    #[arg(long = "no-clipboard", global = true, action = clap::ArgAction::SetTrue, conflicts_with = "clipboard")]
    no_clipboard: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Capture a region (portal interactive picker, or slurp → grim).
    Region,
    /// Capture the full / active output.
    Full,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mode = match cli.command {
        Commands::Region => CaptureMode::Region,
        Commands::Full => CaptureMode::Full,
    };

    let output = resolve_output_path(cli.output, cli.save_dir)?;
    capture_to_png(mode, &output).await?;

    // Clipboard on by default; --no-clipboard disables; --clipboard affirms (no-op vs default).
    let want_clipboard = cli.clipboard || !cli.no_clipboard;
    if want_clipboard {
        match copy_png_to_clipboard(&output).await {
            Ok(true) => {}
            Ok(false) => {
                eprintln!("clipboard skipped: wl-copy not found (install wl-clipboard)");
            }
            Err(err) => {
                eprintln!("clipboard copy failed: {err}");
            }
        }
    }

    println!("{}", output.display());
    Ok(())
}
