//! Screenshot capture — portal/grim on Linux, screencapture on macOS,
//! honest WIP stub on Windows (annotate existing PNGs still works).

use std::path::Path;

use anyhow::Result;
#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
use anyhow::bail;

/// Capture mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureMode {
    /// Interactive region.
    Region,
    /// Full / active output.
    Full,
}

/// Capture a PNG and write it to `output`.
pub async fn capture_to_png(mode: CaptureMode, output: &Path) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        linux::capture_to_png(mode, output).await
    }
    #[cfg(target_os = "macos")]
    {
        macos::capture_to_png(mode, output).await
    }
    #[cfg(target_os = "windows")]
    {
        windows::capture_to_png(mode, output).await
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
    {
        let _ = (mode, output);
        bail!("capture is not supported on this platform yet");
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use std::path::{Path, PathBuf};
    use std::process::Stdio;

    use anyhow::{anyhow, bail, Context, Result};
    use ashpd::desktop::screenshot::{AvailableTargets, Screenshot};
    use tokio::process::Command;
    use url::Url;

    use super::CaptureMode;

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
        let missing = missing_fallback_tools(mode);
        if !missing.is_empty() {
            bail!("{}", fallback_tools_missing_message(&missing));
        }

        if let Some(parent) = output.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("create parent dir {}", parent.display()))?;
        }

        match mode {
            CaptureMode::Region => {
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

    fn missing_fallback_tools(mode: CaptureMode) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if !command_on_path("grim") {
            missing.push("grim");
        }
        if mode == CaptureMode::Region && !command_on_path("slurp") {
            missing.push("slurp");
        }
        missing
    }

    pub(super) fn fallback_tools_missing_message(missing: &[&str]) -> String {
        format!(
            "grim/slurp fallback unavailable — missing on PATH: {}\n\
             See README.md → Runtime dependencies.\n\
             Fedora: sudo dnf install grim slurp wl-clipboard",
            missing.join(", ")
        )
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

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn fallback_message_lists_tools_and_fedora_hint() {
            let msg = fallback_tools_missing_message(&["grim", "slurp"]);
            assert!(msg.contains("missing on PATH: grim, slurp"));
            assert!(msg.contains("README.md → Runtime dependencies"));
            assert!(msg.contains("sudo dnf install grim slurp wl-clipboard"));
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use std::path::Path;

    use anyhow::{bail, Context, Result};
    use tokio::process::Command;

    use super::CaptureMode;

    /// Best-effort capture via the built-in `screencapture` CLI.
    pub async fn capture_to_png(mode: CaptureMode, output: &Path) -> Result<()> {
        if let Some(parent) = output.parent() {
            tokio::fs::create_dir_all(parent)
                .await
                .with_context(|| format!("create parent dir {}", parent.display()))?;
        }

        let mut cmd = Command::new("screencapture");
        cmd.arg("-x"); // no shutter sound
        match mode {
            CaptureMode::Region => {
                // Interactive selection (mouse drag).
                cmd.arg("-i");
            }
            CaptureMode::Full => {
                // Capture the main display without UI chrome.
                cmd.arg("-m");
            }
        }
        cmd.arg(output);

        let status = cmd.status().await.context("run screencapture")?;
        if !status.success() {
            bail!(
                "screencapture exited with {status} (selection cancelled, or Screen Recording permission denied in System Settings → Privacy)"
            );
        }
        if !output.is_file() {
            bail!(
                "screencapture finished but {} is missing (cancelled?)",
                output.display()
            );
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use std::path::Path;

    use anyhow::{bail, Result};

    use super::CaptureMode;

    /// Capture is WIP on Windows. Annotate / save / clipboard still work on an existing PNG.
    pub async fn capture_to_png(mode: CaptureMode, output: &Path) -> Result<()> {
        let _ = (mode, output);
        bail!(
            "Windows screen capture is not wired yet in v0.9 (Windows Graphics Capture planned).\n\
             Workarounds:\n\
             • Capture with Snipping Tool / Win+Shift+S, then: hypershot annotate path.png\n\
             • Or: hypershot annotate existing.png\n\
             See README → Platform support."
        );
    }
}
