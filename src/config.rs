//! TOML config under the user config directory (`directories` + `toml`).

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Env var override for the default save directory (beats config TOML).
pub const SAVE_DIR_ENV: &str = "LIGHTSNIP_SAVE_DIR";

/// On-disk settings. Missing file → defaults.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Default PNG folder when `--save-dir` / env are unset. Empty → Pictures/lightsnip.
    pub save_directory: PathBuf,
    /// Copy to clipboard after Save when neither `--clipboard` nor `--no-clipboard` is set.
    pub save_to_clipboard: bool,
    /// Write an XDG autostart entry (opt-in; off by default).
    pub run_at_startup: bool,
    /// Install a user `.desktop` helper for “screenshot” launchers (off by default; does not
    /// rebind PrintScreen on GNOME/KDE).
    pub replace_default_screenshot: bool,
    /// Keep a StatusNotifierItem tray icon when the tray command is running.
    pub stay_in_tray: bool,
    /// Suggested compositor bind command (docs only; Wayland has no portable global grab).
    pub shortcut_bind_command: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            save_directory: default_pictures_lightsnip(),
            save_to_clipboard: true,
            run_at_startup: false,
            replace_default_screenshot: false,
            stay_in_tray: false,
            shortcut_bind_command: default_shortcut_bind_command(),
        }
    }
}

impl Config {
    /// Load from disk, or return defaults if the file is missing.
    pub fn load() -> Result<Self> {
        let path = config_path()?;
        if !path.is_file() {
            return Ok(Self::default());
        }
        let text = fs::read_to_string(&path)
            .with_context(|| format!("read config {}", path.display()))?;
        let cfg: Config = toml::from_str(&text)
            .with_context(|| format!("parse config {}", path.display()))?;
        Ok(cfg)
    }

    /// Persist to the user config path (creates parent dirs).
    pub fn save(&self) -> Result<PathBuf> {
        let path = config_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create config dir {}", parent.display()))?;
        }
        let text = toml::to_string_pretty(self).context("serialize config TOML")?;
        fs::write(&path, text).with_context(|| format!("write config {}", path.display()))?;
        Ok(path)
    }

    /// Save directory after env override: `LIGHTSNIP_SAVE_DIR` wins over config.
    pub fn effective_save_dir(&self) -> PathBuf {
        if let Ok(dir) = std::env::var(SAVE_DIR_ENV) {
            let trimmed = dir.trim();
            if !trimmed.is_empty() {
                return PathBuf::from(trimmed);
            }
        }
        if self.save_directory.as_os_str().is_empty() {
            default_pictures_lightsnip()
        } else {
            self.save_directory.clone()
        }
    }
}

/// `~/.config/lightsnip/config.toml` (or platform equivalent via `ProjectDirs`).
pub fn config_path() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("dev", "kmfd14", "lightsnip")
        .context("resolve config directory (HOME unset?)")?;
    Ok(dirs.config_dir().join("config.toml"))
}

/// `~/Pictures/lightsnip` (or `/tmp/lightsnip` if home lookup fails).
pub fn default_pictures_lightsnip() -> PathBuf {
    if let Some(user_dirs) = directories::UserDirs::new() {
        if let Some(pictures) = user_dirs.picture_dir() {
            return pictures.join("lightsnip");
        }
        return user_dirs.home_dir().join("Pictures").join("lightsnip");
    }
    PathBuf::from("/tmp/lightsnip")
}

/// Suggested bind using the running binary when possible.
pub fn default_shortcut_bind_command() -> String {
    match std::env::current_exe() {
        Ok(exe) => format!("{} region", shell_quote(&exe)),
        Err(_) => "cargo run --release -- region".to_string(),
    }
}

fn shell_quote(path: &Path) -> String {
    let s = path.display().to_string();
    if s.chars().any(|c| c.is_whitespace() || matches!(c, '"' | '\'' | '$' | '`')) {
        format!("\"{}\"", s.replace('"', "\\\""))
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_clipboard_on() {
        assert!(Config::default().save_to_clipboard);
    }

    #[test]
    fn toml_roundtrip() {
        let mut cfg = Config::default();
        cfg.save_to_clipboard = false;
        cfg.run_at_startup = true;
        cfg.stay_in_tray = true;
        cfg.save_directory = PathBuf::from("/tmp/snips");
        cfg.shortcut_bind_command = "lightsnip region".into();
        let text = toml::to_string_pretty(&cfg).unwrap();
        let back: Config = toml::from_str(&text).unwrap();
        assert_eq!(back, cfg);
    }

    #[test]
    fn empty_save_directory_falls_back() {
        let cfg = Config {
            save_directory: PathBuf::new(),
            ..Config::default()
        };
        let dir = cfg.effective_save_dir();
        assert!(!dir.as_os_str().is_empty());
    }
}
