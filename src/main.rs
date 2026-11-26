//! HyperShot (crate/binary: `hypershot`) — cross-platform screenshot CLI
//! with annotate + settings.

mod annotate;
mod capture;
mod config;
mod desktop;
mod icons;
mod mode_chooser;
mod onscreen;
mod output;
mod region_select;
mod settings;
mod text_anno;
mod tray;
mod ui;

use std::path::PathBuf;

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};

use annotate::{run_annotate_overlay, AnnotateOutcome};
use capture::{capture_to_png, CaptureMode};
use config::{
    AnnotateSessionMode, Config, DefaultCaptureMode, APP_NAME,
};
use mode_chooser::run_mode_chooser;
use onscreen::{run_onscreen_annotate_with_options, take_preview_handoff};
use output::{copy_png_to_clipboard, resolve_output_path};
use region_select::{run_region_select_and_crop, RegionSelectOutcome};
use settings::{resolve_save_dir, run_settings_window, want_clipboard};
use ui::show_copied_to_clipboard_toast;

#[derive(Debug, Parser)]
#[command(
    name = "hypershot",
    about = "HyperShot — screenshot tool with annotation + settings",
    version = env!("CARGO_PKG_VERSION")
)]
struct Cli {
    /// PNG output path (overrides --save-dir / default dir).
    #[arg(short, long, value_name = "PATH", global = true)]
    output: Option<PathBuf>,

    /// Directory for timestamped PNGs when --output is omitted
    /// (default: config, else $HYPERSHOT_SAVE_DIR, else ~/Pictures/hypershot).
    #[arg(long, value_name = "DIR", global = true)]
    save_dir: Option<PathBuf>,

    /// Copy PNG to the clipboard (overrides config).
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
    /// Screenshot shortcut entry: Ask Screenshot Type chooser, or Default Mode.
    Capture,
    /// Capture a region, then annotate (bypasses the mode chooser).
    Region,
    /// Capture the full / active output, then annotate (bypasses the mode chooser).
    Full,
    /// Open the annotate overlay on an existing PNG.
    Annotate {
        /// PNG to annotate (written to the path chosen in the Save dialog).
        path: PathBuf,
    },
    /// Open the settings window (TOML config; no system install required).
    Settings,
    /// Run the StatusNotifierItem tray (Linux; Screenshot / Settings / Quit).
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
        Commands::Capture => {
            run_capture(
                None,
                cli.output,
                cli.save_dir,
                &cfg,
                cli.no_annotate,
                want_clipboard,
            )
            .await?;
        }
        Commands::Region => {
            run_capture(
                Some(CaptureMode::Region),
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
                Some(CaptureMode::Full),
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
                AnnotateOutcome::Saved(saved) => {
                    cleanup_replaced_capture(&work_path, &saved).await;
                    finish_output(&saved, want_clipboard, false).await?;
                }
                AnnotateOutcome::Copied(saved) => {
                    cleanup_replaced_capture(&work_path, &saved).await;
                    // Copy / Enter always forces clipboard, even if config/CLI preferred off.
                    finish_output(&saved, true, true).await?;
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

/// `forced`: explicit `region` / `full` CLI. `None` = `capture` (respect Ask / Default Mode).
async fn run_capture(
    forced: Option<CaptureMode>,
    output: Option<PathBuf>,
    save_dir: Option<PathBuf>,
    cfg: &Config,
    no_annotate: bool,
    want_clipboard: bool,
) -> Result<()> {
    let dir = resolve_save_dir(save_dir, cfg)?;
    let output = resolve_output_path(output, Some(dir))?;

    // Always capture the full output first. Chooser / region select / annotate
    // run on that frame. Full Screen opens the preview window by default.
    capture_to_png(CaptureMode::Full, &output).await?;

    let mode = match forced {
        Some(mode) => mode,
        None if cfg.ask_screenshot_type => {
            let chosen = run_mode_chooser(&output)?;
            match chosen.as_capture_mode() {
                Some(mode) => mode,
                None => {
                    let _ = tokio::fs::remove_file(&output).await;
                    eprintln!("capture cancelled");
                    return Ok(());
                }
            }
        }
        None => match cfg.default_capture_mode {
            DefaultCaptureMode::Region => CaptureMode::Region,
            DefaultCaptureMode::FullScreen => CaptureMode::Full,
        },
    };

    if no_annotate {
        if mode == CaptureMode::Region {
            match run_region_select_and_crop(&output) {
                Ok(RegionSelectOutcome::Selected(_)) => {}
                Ok(RegionSelectOutcome::Cancelled) => {
                    let _ = tokio::fs::remove_file(&output).await;
                    eprintln!("region select cancelled");
                    return Ok(());
                }
                Err(err) => {
                    let _ = tokio::fs::remove_file(&output).await;
                    bail!("{APP_NAME} region select failed: {err:#}");
                }
            }
        }
        return finish_output(&output, want_clipboard, false).await;
    }

    let mut force_clipboard = false;
    let mut final_path = output.clone();

    // Full Screen always opens the floating preview window.
    let session_mode = if mode == CaptureMode::Full {
        AnnotateSessionMode::Preview
    } else {
        cfg.annotate_session_mode
    };

    match session_mode {
        AnnotateSessionMode::OnScreen => {
            // Region: drag-select on the overlay. Full: pre-select entire frame.
            let preselect_full = mode == CaptureMode::Full;
            match run_onscreen_annotate_with_options(&output, preselect_full) {
                Ok(AnnotateOutcome::Saved(saved)) => {
                    if take_preview_handoff(&saved) {
                        // User asked to continue in the floating preview window.
                        match run_annotate_overlay(&saved) {
                            Ok(AnnotateOutcome::Saved(s2)) => {
                                cleanup_replaced_capture(&output, &s2).await;
                                final_path = s2;
                            }
                            Ok(AnnotateOutcome::Copied(s2)) => {
                                cleanup_replaced_capture(&output, &s2).await;
                                final_path = s2;
                                force_clipboard = true;
                            }
                            Ok(AnnotateOutcome::Cancelled) => {
                                cleanup_replaced_capture(&output, &saved).await;
                                final_path = saved;
                            }
                            Err(err) => {
                                eprintln!("preview window unavailable ({err:#}); keeping on-screen save");
                                cleanup_replaced_capture(&output, &saved).await;
                                final_path = saved;
                            }
                        }
                    } else {
                        cleanup_replaced_capture(&output, &saved).await;
                        final_path = saved;
                    }
                }
                Ok(AnnotateOutcome::Copied(saved)) => {
                    let _ = take_preview_handoff(&saved);
                    cleanup_replaced_capture(&output, &saved).await;
                    final_path = saved;
                    force_clipboard = true;
                }
                Ok(AnnotateOutcome::Cancelled) => {
                    let _ = tokio::fs::remove_file(&output).await;
                    eprintln!("annotate cancelled");
                    return Ok(());
                }
                Err(err) => {
                    eprintln!(
                        "on-screen annotate unavailable ({err:#}); falling back to preview window"
                    );
                    if mode == CaptureMode::Region {
                        if let Err(sel_err) = run_region_select_and_crop(&output) {
                            let _ = tokio::fs::remove_file(&output).await;
                            bail!("{APP_NAME} region select failed: {sel_err:#}");
                        }
                    }
                    match run_annotate_overlay(&output) {
                        Ok(AnnotateOutcome::Saved(saved)) => {
                            cleanup_replaced_capture(&output, &saved).await;
                            final_path = saved;
                        }
                        Ok(AnnotateOutcome::Copied(saved)) => {
                            cleanup_replaced_capture(&output, &saved).await;
                            final_path = saved;
                            force_clipboard = true;
                        }
                        Ok(AnnotateOutcome::Cancelled) => {
                            let _ = tokio::fs::remove_file(&output).await;
                            eprintln!("annotate cancelled");
                            return Ok(());
                        }
                        Err(err2) => {
                            eprintln!("annotate window unavailable ({err2:#}); keeping raw capture");
                        }
                    }
                }
            }
        }
        AnnotateSessionMode::Preview => {
            if mode == CaptureMode::Region {
                match run_region_select_and_crop(&output) {
                    Ok(RegionSelectOutcome::Selected(_)) => {}
                    Ok(RegionSelectOutcome::Cancelled) => {
                        let _ = tokio::fs::remove_file(&output).await;
                        eprintln!("region select cancelled");
                        return Ok(());
                    }
                    Err(err) => {
                        let _ = tokio::fs::remove_file(&output).await;
                        bail!("{APP_NAME} region select failed: {err:#}");
                    }
                }
            }
            match run_annotate_overlay(&output) {
                Ok(AnnotateOutcome::Saved(saved)) => {
                    cleanup_replaced_capture(&output, &saved).await;
                    final_path = saved;
                }
                Ok(AnnotateOutcome::Copied(saved)) => {
                    cleanup_replaced_capture(&output, &saved).await;
                    final_path = saved;
                    force_clipboard = true;
                }
                Ok(AnnotateOutcome::Cancelled) => {
                    let _ = tokio::fs::remove_file(&output).await;
                    eprintln!("annotate cancelled");
                    return Ok(());
                }
                Err(err) => {
                    eprintln!("annotate window unavailable ({err:#}); keeping raw capture");
                }
            }
        }
    }

    finish_output(
        &final_path,
        want_clipboard || force_clipboard,
        force_clipboard,
    )
    .await
}

/// If the user saved to a different path than the capture temp file, remove the orphan.
async fn cleanup_replaced_capture(original: &std::path::Path, saved: &std::path::Path) {
    if original != saved && original.is_file() {
        let _ = tokio::fs::remove_file(original).await;
    }
}

/// `announce_copy`: show the designed “Copied to Clipboard!” popup (Copy / Enter only).
async fn finish_output(
    path: &std::path::Path,
    want_clipboard: bool,
    announce_copy: bool,
) -> Result<()> {
    if want_clipboard {
        match copy_png_to_clipboard(path).await {
            Ok(true) => {
                if announce_copy {
                    show_copied_to_clipboard_toast();
                }
            }
            Ok(false) => {
                eprintln!("clipboard skipped (unexpected)");
            }
            Err(err) => {
                eprintln!(
                    "clipboard copy failed: {err}\n\
                     On Linux, install wl-clipboard (`wl-copy`) if copy keeps failing.\n\
                     See README.md → Runtime dependencies / Platform support."
                );
            }
        }
    }
    println!("{}", path.display());
    Ok(())
}
