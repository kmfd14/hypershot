//! Save path resolution and cross-platform clipboard copy.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{Datelike, Local, Timelike};
use image::RgbaImage;

use crate::config::{default_filename_pattern, Config};

/// Resolve where to write the PNG.
///
/// Precedence: `--output PATH` → patterned file under `--save-dir` /
/// `HYPERSHOT_SAVE_DIR` / config `save_directory` / `~/Pictures/hypershot`.
pub fn resolve_output_path(output: Option<PathBuf>, save_dir: Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = output {
        return Ok(path);
    }
    let dir = save_dir.unwrap_or_else(default_save_dir);
    let pattern = Config::load()
        .map(|c| c.filename_pattern)
        .unwrap_or_else(|_| default_filename_pattern());
    Ok(unique_path_in_dir(&dir, &render_filename_pattern(&pattern)))
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

/// Build a PNG file name from a strftime-style pattern (config / Settings → File names).
///
/// Supported tokens: `%Y` `%y` `%m` `%d` `%H` `%M` `%S` `%%`. Empty → default pattern.
/// Appends `.png` when missing. Path separators become `_`.
pub fn render_filename_pattern(pattern: &str) -> String {
    let pat = pattern.trim();
    let pat = if pat.is_empty() {
        default_filename_pattern()
    } else {
        pat.to_string()
    };

    let now = Local::now();
    let mut out = String::with_capacity(pat.len() + 8);
    let bytes = pat.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 1 < bytes.len() {
            let piece = match bytes[i + 1] {
                b'Y' => format!("{:04}", now.year()),
                b'y' => format!("{:02}", now.year().rem_euclid(100)),
                b'm' => format!("{:02}", now.month()),
                b'd' => format!("{:02}", now.day()),
                b'H' => format!("{:02}", now.hour()),
                b'M' => format!("{:02}", now.minute()),
                b'S' => format!("{:02}", now.second()),
                b'%' => "%".to_string(),
                other => format!("%{}", other as char),
            };
            out.push_str(&piece);
            i += 2;
        } else {
            out.push(bytes[i] as char);
            i += 1;
        }
    }

    let sanitized: String = out
        .chars()
        .map(|c| if matches!(c, '/' | '\\' | '\0') { '_' } else { c })
        .collect();
    if sanitized.to_ascii_lowercase().ends_with(".png") {
        sanitized
    } else {
        format!("{sanitized}.png")
    }
}

/// File name for dialogs / capture using the live config pattern.
pub fn default_capture_filename() -> String {
    let pattern = Config::load()
        .map(|c| c.filename_pattern)
        .unwrap_or_else(|_| default_filename_pattern());
    render_filename_pattern(&pattern)
}

/// Prefer an unused path: `name.png`, then `name_2.png`, …
pub fn unique_path_in_dir(dir: &Path, filename: &str) -> PathBuf {
    let path = dir.join(filename);
    if !path.exists() {
        return path;
    }
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "screenshot".into());
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    for n in 2..10_000 {
        let candidate = dir.join(format!("{stem}_{n}{ext}"));
        if !candidate.exists() {
            return candidate;
        }
    }
    dir.join(format!("{stem}_{}{ext}", std::process::id()))
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
    fn save_dir_gets_patterned_name() {
        let path = resolve_output_path(None, Some(PathBuf::from("/tmp/snips"))).unwrap();
        assert_eq!(path.parent(), Some(Path::new("/tmp/snips")));
        let name = path.file_name().unwrap().to_string_lossy();
        assert!(name.ends_with(".png"));
        assert!(!name.is_empty());
    }

    #[test]
    fn render_pattern_default_tokens() {
        let name = render_filename_pattern("screenshot_%d%m%y");
        assert!(name.starts_with("screenshot_"));
        assert!(name.ends_with(".png"));
        // screenshot_ + 6 digits + .png
        let stem = name.trim_end_matches(".png");
        let digits = stem.rsplit('_').next().unwrap_or("");
        assert_eq!(digits.len(), 6);
        assert!(digits.chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn render_pattern_appends_png_and_sanitizes() {
        assert!(render_filename_pattern("a/b").ends_with(".png"));
        assert!(!render_filename_pattern("a/b").contains('/'));
        assert_eq!(
            render_filename_pattern("already.png").to_ascii_lowercase(),
            "already.png"
        );
        assert_eq!(render_filename_pattern("100%%"), "100%.png");
    }

    #[test]
    fn unique_path_skips_existing() {
        let dir = std::env::temp_dir().join(format!("hypershot-unique-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let first = dir.join("shot.png");
        std::fs::write(&first, b"x").unwrap();
        let next = unique_path_in_dir(&dir, "shot.png");
        assert_eq!(next, dir.join("shot_2.png"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
