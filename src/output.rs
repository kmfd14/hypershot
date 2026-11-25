//! Save path resolution and cross-platform clipboard copy.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use image::RgbaImage;

/// Resolve where to write the PNG.
///
/// Precedence: `--output PATH` → timestamped file under `--save-dir` /
/// `HYPERSHOT_SAVE_DIR` / config `save_directory` / `~/Pictures/hypershot`.
pub fn resolve_output_path(output: Option<PathBuf>, save_dir: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = output {
        return Ok(path);
    }
    let dir = save_dir.unwrap_or_else(default_save_dir);
    Ok(dir.join(timestamped_filename()))
}

/// Default save directory: `$HYPERSHOT_SAVE_DIR` (or legacy `$LIGHTSNIP_SAVE_DIR`),
/// else config, else Pictures/hypershot.
pub fn default_save_dir() -> PathBuf {
    if let Some(dir) = crate::config::env_save_dir() {
        return dir;
    }
    if let Ok(cfg) = crate::config::Config::load() {
        if !cfg.save_directory.as_os_str().is_empty() {
            return cfg.save_directory;
        }
    }
    crate::config::default_pictures_hypershot()
}

fn timestamped_filename() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("hypershot-{secs}.png")
}

/// Copy a PNG file to the system clipboard (image/png via `arboard`).
///
/// Returns `Ok(true)` on success. On Linux, `arboard` uses `wl-copy` when
/// available — install `wl-clipboard` if copy fails.
pub async fn copy_png_to_clipboard(path: &Path) -> Result<bool> {
    let path = path.to_path_buf();
    tokio::task::spawn_blocking(move || copy_png_to_clipboard_blocking(&path))
        .await
        .context("clipboard worker join")?
}

fn copy_png_to_clipboard_blocking(path: &Path) -> Result<bool> {
    let bytes = std::fs::read(path)
        .with_context(|| format!("read PNG for clipboard {}", path.display()))?;
    let rgba = image::load_from_memory(&bytes)
        .with_context(|| format!("decode PNG for clipboard {}", path.display()))?
        .to_rgba8();
    let (w, h) = rgba.dimensions();
    let img = RgbaImage::from_raw(w, h, rgba.into_raw())
        .ok_or_else(|| anyhow::anyhow!("rebuild RGBA for clipboard"))?;

    let mut clipboard = arboard::Clipboard::new().context("open system clipboard")?;
    clipboard
        .set_image(arboard::ImageData {
            width: w as usize,
            height: h as usize,
            bytes: img.into_raw().into(),
        })
        .context("set clipboard image")?;
    Ok(true)
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
        assert!(name.starts_with("hypershot-"));
        assert!(name.ends_with(".png"));
    }
}
