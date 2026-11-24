//! lightsnip — Wayland screenshot CLI (Milestone 3: annotate overlay).

mod annotate;
mod capture;
mod output;

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

use annotate::{run_annotate_overlay, AnnotateOutcome};
use capture::{capture_to_png, CaptureMode};
use output::{copy_png_to_clipboard, resolve_output_path};

#[derive(Debug, Parser)]
#[command(
    name = "lightsnip",
    about = "Wayland screenshot tool with annotation (annotate overlay — Milestone 3)",
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
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let want_clipboard = cli.clipboard || !cli.no_clipboard;

    match cli.command {
        Commands::Region => {
            run_capture(CaptureMode::Region, cli.output, cli.save_dir, cli.no_annotate, want_clipboard)
                .await?;
        }
        Commands::Full => {
            run_capture(CaptureMode::Full, cli.output, cli.save_dir, cli.no_annotate, want_clipboard)
                .await?;
        }
        Commands::Annotate { path } => {
            if !path.is_file() {
                bail!("PNG not found: {}", path.display());
            }
            // Optional redirect: annotate path, but write Save result to -o if set.
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
    }

    Ok(())
}

async fn run_capture(
    mode: CaptureMode,
    output: Option<PathBuf>,
    save_dir: Option<PathBuf>,
    no_annotate: bool,
    want_clipboard: bool,
) -> Result<()> {
    let output = resolve_output_path(output, save_dir)?;
    capture_to_png(mode, &output).await?;

    if !no_annotate {
        match run_annotate_overlay(&output) {
            Ok(AnnotateOutcome::Saved) => {}
            Ok(AnnotateOutcome::Cancelled) => {
                // Discard the raw capture so Escape cancels the whole snip.
                let _ = tokio::fs::remove_file(&output).await;
                eprintln!("annotate cancelled");
                return Ok(());
            }
            Err(err) => {
                // Cloud / headless: keep raw capture + continue to clipboard.
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
                eprintln!("clipboard skipped: wl-copy not found (install wl-clipboard)");
            }
            Err(err) => {
                eprintln!("clipboard copy failed: {err}");
            }
        }
    }
    println!("{}", path.display());
    Ok(())
}
