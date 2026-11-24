//! lightsnip - Wayland screenshot CLI (Milestone 5: settings window).

mod annotate;
mod capture;
mod config;
mod desktop;
mod output;
mod settings;
mod tray;
mod ui;

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

use annotate::{run_annotate_overlay, AnnotateOutcome};
use capture::{capture_to_png, CaptureMode};
use config::Config;
use output::{copy_png_to_clipboard, resolve_output_path};
use settings::{resolve_save_dir, run_settings_window, want_clipboard};

#[derive(Debug, Parser)]
#[command(
    name = "lightsnip",
    about = "Wayland screenshot tool with annotation + settings (Milestone 5)",
    version
)]
struct Cli {
    /// PNG output path (overrides --save-dir / default dir).
    #[arg(short, long, value_name = "PATH", global = true)]
    output: Option<PathBuf>,

    /// Directory for timestamped PNGs when --output is omitted
    /// (default: config, else $LIGHTSNIP_SAVE_DIR, else ~/Pictures/lightsnip).
    #[arg(long, value_name = "DIR", global = true)]
    save_dir: Option<PathBuf>,

    /// Copy PNG to the Wayland clipboard via wl-copy (overrides config).
    #[arg(long, global = true, action = clap::ArgAction::SetTrue, conflicts_with = "no_clipboard")]
    clipboard: bool,

    /// Do not copy the PNG to the clipboard (overrides config).
    #[arg(long = "no-clipboard", global = true, action = clap::ArgAction::SetTrue, conflicts_with = "clipboard")]
    no_clipboard: bool,

    /// Skip the annotate overlay (keep raw capture; still save + clipboard).
    #[arg(long = "no-annotate", global = true, action = clap::ArgAction::SetTrue)]
    no_annotate: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Capture a region (portal interactive picker, or slurp → grim), then annotate.
    Region,
    /// Capture the full / active output, then annotate.
    Full,
    /// Open the annotate overlay on an existing PNG.
    Annotate {
        /// PNG to annotate (overwritten on Save).
        path: PathBuf,
    },
    /// Open the settings window (TOML config; no system install required).
    Settings,
    /// Run the StatusNotifierItem tray (Region / Full / Settings / Quit).
    Tray,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let cfg = Config::load().unwrap_or_else(|err| {
        eprintln!("config load failed ({err:#}); using defaults");
        Config::default()
    });
    let want_clipboard = want_clipboard(cli.clipboard, cli.no_clipboard, &cfg);

    match cli.command {
        Commands::Region => {
            run_capture(
                CaptureMode::Region,
                cli.output,
                cli.save_dir,
                &cfg,
                cli.no_annotate,
                want_clipboard,
            )
            .await?;
        }
        Commands::Full => {
            run_capture(
                CaptureMode::Full,
                cli.output,
                cli.save_dir,
                &cfg,
                cli.no_annotate,
                want_clipboard,
            )
            .await?;
        }
        Commands::Annotate { path } => {
            if !path.is_file() {
                bail!("PNG not found: {}", path.display());
            }
            let work_path = if let Some(out) = cli.output.as_ref() {
                tokio::fs::copy(&path, out)
                    .await
                    .with_context(|| {
                        format!("copy {} → {} for annotate", path.display(), out.display())
                    })?;
                out.clone()
            } else {
                path
            };

            match run_annotate_overlay(&work_path)? {
                AnnotateOutcome::Saved => {
                    finish_output(&work_path, want_clipboard).await?;
                }
                AnnotateOutcome::Cancelled => {
                    eprintln!("annotate cancelled");
                }
            }
        }
        Commands::Settings => {
            run_settings_window()?;
        }
        Commands::Tray => {
            tray::run_tray().await?;
        }
    }

    Ok(())
}

async fn run_capture(
    mode: CaptureMode,
    output: Option<PathBuf>,
    save_dir: Option<PathBuf>,
    cfg: &Config,
    no_annotate: bool,
    want_clipboard: bool,
) -> Result<()> {
    let dir = resolve_save_dir(save_dir, cfg)?;
    let output = resolve_output_path(output, Some(dir))?;
    capture_to_png(mode, &output).await?;

    if !no_annotate {
        match run_annotate_overlay(&output) {
            Ok(AnnotateOutcome::Saved) => {}
            Ok(AnnotateOutcome::Cancelled) => {
                let _ = tokio::fs::remove_file(&output).await;
                eprintln!("annotate cancelled");
                return Ok(());
            }
            Err(err) => {
                eprintln!("annotate overlay unavailable ({err:#}); keeping raw capture");
            }
        }
    }

    finish_output(&output, want_clipboard).await
}

async fn finish_output(path: &std::path::Path, want_clipboard: bool) -> Result<()> {
    if want_clipboard {
        match copy_png_to_clipboard(path).await {
            Ok(true) => {}
            Ok(false) => {
                eprintln!(
                    "clipboard skipped: wl-copy not found on PATH\n\
                     See README.md → Runtime dependencies.\n\
                     Fedora: sudo dnf install wl-clipboard"
                );
            }
            Err(err) => {
                eprintln!("clipboard copy failed: {err}");
            }
        }
    }
    println!("{}", path.display());
    Ok(())
}
