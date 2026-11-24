//! Portal-first screenshot capture with grim/slurp fallback.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use anyhow::{anyhow, bail, Context, Result};
use ashpd::desktop::screenshot::{AvailableTargets, Screenshot};
use tokio::process::Command;
use url::Url;

/// Capture mode for Milestone 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureMode {
    /// Interactive region (portal picker or slurp).
    Region,
    /// Full / active output.
    Full,
}

/// Capture a PNG and write it to `output`.
///
/// Prefers xdg-desktop-portal via `ashpd`. On failure or a missing portal,
/// falls back to `grim` (+ `slurp` for region) when those tools are on `PATH`.
pub async fn capture_to_png(mode: CaptureMode, output: &Path) -> Result<()> {
    match portal_capture(mode).await {
        Ok(portal_path) => {
            if let Some(parent) = output.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .with_context(|| format!("create parent dir {}", parent.display()))?;
            }
            tokio::fs::copy(&portal_path, output)
                .await
                .with_context(|| {
                    format!(
                        "copy portal capture {} → {}",
                        portal_path.display(),
                        output.display()
                    )
                })?;
            // Best-effort cleanup of the portal temp file.
            let _ = tokio::fs::remove_file(&portal_path).await;
            Ok(())
        }
        Err(portal_err) => {
            eprintln!(
                "portal capture failed ({portal_err}); trying grim/slurp fallback"
            );
            grim_slurp_capture(mode, output).await
        }
    }
}

async fn portal_capture(mode: CaptureMode) -> Result<PathBuf> {
    let mut request = Screenshot::request().modal(true);

    match mode {
        CaptureMode::Region => {
            request = request
                .interactive(true)
                .target(AvailableTargets::AREA | AvailableTargets::SCREEN);
        }
        CaptureMode::Full => {
            request = request
                .interactive(false)
                .target(AvailableTargets::SCREEN);
        }
    }

    let response = request
        .send()
        .await
        .context("xdg-desktop-portal Screenshot request failed")?
        .response()
        .context("portal Screenshot cancelled or denied")?;

    file_uri_to_path(response.uri().as_str())
}

fn file_uri_to_path(uri_str: &str) -> Result<PathBuf> {
    let url = Url::parse(uri_str).with_context(|| format!("parse portal URI {uri_str}"))?;
    if url.scheme() != "file" {
        bail!("portal returned non-file URI: {uri_str}");
    }
    let path = url
        .to_file_path()
        .map_err(|_| anyhow!("portal URI is not a local file path: {uri_str}"))?;
    if !path.is_file() {
        bail!("portal file missing: {}", path.display());
    }
    Ok(path)
}

async fn grim_slurp_capture(mode: CaptureMode, output: &Path) -> Result<()> {
    if !command_on_path("grim") {
        bail!(
            "grim not found on PATH (install grim for wlroots/tiling fallback)"
        );
    }

    if let Some(parent) = output.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("create parent dir {}", parent.display()))?;
    }

    match mode {
        CaptureMode::Region => {
            if !command_on_path("slurp") {
                bail!(
                    "slurp not found on PATH (needed for region selection fallback)"
                );
            }
            let geometry = Command::new("slurp")
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .output()
                .await
                .context("run slurp")?;
            if !geometry.status.success() {
                bail!("slurp exited with {}", geometry.status);
            }
            let geom = String::from_utf8(geometry.stdout)
                .context("slurp stdout was not UTF-8")?
                .trim()
                .to_owned();
            if geom.is_empty() {
                bail!("slurp returned empty geometry (selection cancelled?)");
            }
            let status = Command::new("grim")
                .args(["-g", &geom])
                .arg(output)
                .status()
                .await
                .context("run grim -g")?;
            if !status.success() {
                bail!("grim exited with {status}");
            }
        }
        CaptureMode::Full => {
            let status = Command::new("grim")
                .arg(output)
                .status()
                .await
                .context("run grim")?;
            if !status.success() {
                bail!("grim exited with {status}");
            }
        }
    }

    if !output.is_file() {
        bail!("capture finished but {} is missing", output.display());
    }
    Ok(())
}

fn command_on_path(name: &str) -> bool {
    let Some(path_var) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path_var).any(|dir| {
        let candidate = dir.join(name);
        candidate.is_file()
    })
}

/// Default PNG path under `/tmp` with a unix-timestamp suffix.
pub fn default_output_path() -> PathBuf {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    PathBuf::from(format!("/tmp/lightsnip-{secs}.png"))
}
