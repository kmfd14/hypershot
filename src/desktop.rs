//! XDG autostart + optional “replace default screenshot” desktop entry.
//!
//! Honest limits: these helpers never rebind PrintScreen on GNOME/KDE. They only
//! write/remove user-local `.desktop` files. Compositor binds stay manual.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::config::Config;

const AUTOSTART_NAME: &str = "lightsnip.desktop";
const REPLACE_NAME: &str = "lightsnip-screenshot.desktop";

/// Apply desktop side effects for the saved config (autostart + replace-default).
pub fn apply_desktop_integrations(cfg: &Config) -> Result<Vec<String>> {
    let mut notes = Vec::new();
    notes.extend(sync_autostart(cfg.run_at_startup)?);
    notes.extend(sync_replace_default(cfg.replace_default_screenshot)?);
    if cfg.stay_in_tray && cfg.run_at_startup {
        notes.push(
            "Autostart launches `tray` so the icon returns after login.".into(),
        );
    } else if cfg.stay_in_tray {
        notes.push(
            "Stay in tray is on: run `cargo run --release -- tray` (or Quit from the tray) to show the icon now."
                .into(),
        );
    }
    Ok(notes)
}

fn sync_autostart(enabled: bool) -> Result<Vec<String>> {
    let path = autostart_path()?;
    if enabled {
        let exe = current_exe_string()?;
        let body = desktop_file(
            "HyperShot",
            "HyperShot tray (autostart; binary: lightsnip)",
            &format!("{exe} tray"),
            false,
        );
        write_desktop(&path, &body)?;
        Ok(vec![format!("Autostart written: {}", path.display())])
    } else if path.is_file() {
        fs::remove_file(&path)
            .with_context(|| format!("remove autostart {}", path.display()))?;
        Ok(vec![format!("Autostart removed: {}", path.display())])
    } else {
        Ok(Vec::new())
    }
}

fn sync_replace_default(enabled: bool) -> Result<Vec<String>> {
    let path = applications_path()?.join(REPLACE_NAME);
    if enabled {
        let exe = current_exe_string()?;
        // Categories + Keywords help some launchers list this as a screenshot action.
        // It does NOT steal the Print key; bind that in your DE if you want.
        let body = format!(
            "\
[Desktop Entry]
Type=Application
Name=HyperShot Screenshot
Comment=Region capture with HyperShot (does not rebind PrintScreen)
Exec={exe} region
Terminal=false
Categories=Utility;Graphics;Photography;
Keywords=screenshot;snip;capture;wayland;
StartupNotify=false
NoDisplay=false
"
        );
        write_desktop(&path, &body)?;
        Ok(vec![
            format!("Screenshot desktop entry written: {}", path.display()),
            "Replace-default does not rebind PrintScreen on GNOME/KDE. Assign a custom shortcut to this command, or bind it in Sway/Hyprland.".into(),
        ])
    } else if path.is_file() {
        fs::remove_file(&path)
            .with_context(|| format!("remove replace-default {}", path.display()))?;
        Ok(vec![format!(
            "Screenshot desktop entry removed: {}",
            path.display()
        )])
    } else {
        Ok(Vec::new())
    }
}

fn desktop_file(name: &str, comment: &str, exec: &str, no_display: bool) -> String {
    format!(
        "\
[Desktop Entry]
Type=Application
Name={name}
Comment={comment}
Exec={exec}
Terminal=false
Categories=Utility;
X-GNOME-Autostart-enabled=true
StartupNotify=false
NoDisplay={no_display}
"
    )
}

fn write_desktop(path: &Path, body: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, body).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

fn autostart_path() -> Result<PathBuf> {
    let base = xdg_config_home()?;
    Ok(base.join("autostart").join(AUTOSTART_NAME))
}

fn applications_path() -> Result<PathBuf> {
    let home = directories::UserDirs::new()
        .context("resolve home directory")?
        .home_dir()
        .to_path_buf();
    let data = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join(".local").join("share"));
    Ok(data.join("applications"))
}

fn xdg_config_home() -> Result<PathBuf> {
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME") {
        return Ok(PathBuf::from(dir));
    }
    let home = directories::UserDirs::new()
        .context("resolve home directory")?
        .home_dir()
        .to_path_buf();
    Ok(home.join(".config"))
}

fn current_exe_string() -> Result<String> {
    let exe = std::env::current_exe().context("resolve current executable")?;
    Ok(exe.display().to_string())
}

/// Spawn a detached child of this binary with `args` (used by tray + settings).
pub fn spawn_self(args: &[&str]) -> Result<()> {
    let exe = std::env::current_exe().context("resolve current executable")?;
    Command::new(&exe)
        .args(args)
        .spawn()
        .with_context(|| format!("spawn {} {:?}", exe.display(), args))?;
    Ok(())
}

/// Try to start the tray if stay_in_tray is on (best-effort; ignore duplicate failures).
pub fn maybe_spawn_tray(cfg: &Config) -> Result<()> {
    if !cfg.stay_in_tray {
        return Ok(());
    }
    match spawn_self(&["tray"]) {
        Ok(()) => Ok(()),
        Err(err) => {
            bail!("could not spawn tray: {err:#}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn desktop_file_contains_exec() {
        let body = desktop_file("lightsnip", "test", "/tmp/lightsnip tray", true);
        assert!(body.contains("Exec=/tmp/lightsnip tray"));
        assert!(body.contains("NoDisplay=true"));
    }
}
