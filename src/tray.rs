//! StatusNotifierItem tray (ksni) for “Stay in taskbar / tray”.
//!
//! Linux only (freedesktop/KDE SNI D-Bus). macOS / Windows: stub error with a
//! clear message — annotate + settings still work without a tray.

use anyhow::Result;

#[cfg(target_os = "linux")]
mod linux {
    use anyhow::{Context, Result};
    use ksni::TrayMethods;

    use crate::desktop::spawn_self;

    #[derive(Debug)]
    struct HypershotTray;

    impl ksni::Tray for HypershotTray {
        fn id(&self) -> String {
            "hypershot".into()
        }

        fn title(&self) -> String {
            "HyperShot".into()
        }

        fn icon_name(&self) -> String {
            // Common freedesktop theme name; falls back to text title if missing.
            "camera-photo".into()
        }

        fn tool_tip(&self) -> ksni::ToolTip {
            ksni::ToolTip {
                title: "HyperShot".into(),
                description: "Screenshot · Settings".into(),
                ..Default::default()
            }
        }

        fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
            use ksni::menu::*;
            vec![
                StandardItem {
                    label: "Screenshot".into(),
                    activate: Box::new(|_| {
                        let _ = spawn_self(&["capture"]);
                    }),
                    ..Default::default()
                }
                .into(),
                StandardItem {
                    label: "Region".into(),
                    activate: Box::new(|_| {
                        let _ = spawn_self(&["region"]);
                    }),
                    ..Default::default()
                }
                .into(),
                StandardItem {
                    label: "Full Screen".into(),
                    activate: Box::new(|_| {
                        let _ = spawn_self(&["full"]);
                    }),
                    ..Default::default()
                }
                .into(),
                MenuItem::Separator,
                StandardItem {
                    label: "Settings".into(),
                    activate: Box::new(|_| {
                        let _ = spawn_self(&["settings"]);
                    }),
                    ..Default::default()
                }
                .into(),
                StandardItem {
                    label: "Quit".into(),
                    icon_name: "application-exit".into(),
                    activate: Box::new(|_| std::process::exit(0)),
                    ..Default::default()
                }
                .into(),
            ]
        }
    }

    pub async fn run_tray() -> Result<()> {
        let tray = HypershotTray;
        let _handle = tray
            .spawn()
            .await
            .context("start StatusNotifierItem tray (is a tray watcher running?)")?;
        eprintln!("HyperShot tray running (Quit from the tray menu to exit)");
        std::future::pending::<()>().await;
        Ok(())
    }
}

/// Run the tray until Quit (or process exit).
pub async fn run_tray() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        linux::run_tray().await
    }
    #[cfg(not(target_os = "linux"))]
    {
        anyhow::bail!(
            "tray is Linux-only (StatusNotifierItem / ksni).\n\
             On macOS / Windows use: hypershot capture | region | full | settings | annotate <png>"
        );
    }
}
