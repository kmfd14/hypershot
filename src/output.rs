//! Save path resolution and Wayland clipboard copy.

use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// Env var override for the default save directory (Milestone 2; settings later).
pub const SAVE_DIR_ENV: &str = "LIGHTSNIP_SAVE_DIR";

/// Resolve where to write the PNG.
///
/// Precedence: `--output PATH` → timestamped file under `--save-dir` /
/// `LIGHTSNIP_SAVE_DIR` / `~/Pictures/lightsnip`.
pub fn resolve_output_path(output: Option<PathBuf>, save_dir: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = output {
        return Ok(path);
    }
    let dir = save_dir.unwrap_or_else(default_save_dir);
    Ok(dir.join(timestamped_filename()))
}

/// Default save directory: `$LIGHTSNIP_SAVE_DIR`, else `~/Pictures/lightsnip`
/// (or `$HOME/Pictures/lightsnip` if picture dir lookup fails).
pub fn default_save_dir() -> PathBuf {
    if let Ok(dir) = std::env::var(SAVE_DIR_ENV) {
        let trimmed = dir.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    if let Some(user_dirs) = directories::UserDirs::new() {
        if let Some(pictures) = user_dirs.picture_dir() {
            return pictures.join("lightsnip");
        }
        return user_dirs.home_dir().join("Pictures").join("lightsnip");
    }
    PathBuf::from("/tmp/lightsnip")
}

fn timestamped_filename() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("lightsnip-{secs}.png")
}

/// Copy a PNG file to the Wayland clipboard via `wl-copy` when available.
///
/// Returns `Ok(true)` if copy succeeded, `Ok(false)` if `wl-copy` is missing
/// (caller should warn). Errors if `wl-copy` ran but failed.
pub async fn copy_png_to_clipboard(path: &Path) -> Result<bool> {
    if !command_on_path("wl-copy") {
        return Ok(false);
    }

    let bytes = tokio::fs::read(path)
        .await
        .with_context(|| format!("read PNG for clipboard {}", path.display()))?;

    let mut child = Command::new("wl-copy")
        .args(["--type", "image/png"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .context("spawn wl-copy")?;

    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("wl-copy stdin missing"))?;
        stdin
            .write_all(&bytes)
            .await
            .context("write PNG bytes to wl-copy")?;
        stdin.shutdown().await.context("close wl-copy stdin")?;
    }

    let output = child
        .wait_with_output()
        .await
        .context("wait for wl-copy")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "wl-copy exited with {}{}",
            output.status,
            if stderr.trim().is_empty() {
                String::new()
            } else {
                format!(": {}", stderr.trim())
            }
        );
    }
    Ok(true)
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
    fn output_flag_wins_over_save_dir() {
        let path = resolve_output_path(
            Some(PathBuf::from("/tmp/custom.png")),
            Some(PathBuf::from("/tmp/other")),
        )
        .unwrap();
        assert_eq!(path, PathBuf::from("/tmp/custom.png"));
    }

    #[test]
    fn save_dir_gets_timestamped_name() {
        let path = resolve_output_path(None, Some(PathBuf::from("/tmp/snips"))).unwrap();
        assert_eq!(path.parent(), Some(Path::new("/tmp/snips")));
        let name = path.file_name().unwrap().to_string_lossy();
        assert!(name.starts_with("lightsnip-"));
        assert!(name.ends_with(".png"));
    }
}
