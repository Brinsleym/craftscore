//! scorecraft: The ScoreCraft desktop app (eframe/wgpu) with the TCP control server.
//!
//! `scorecraft [score.pdf]` opens the reader; with no file it reopens the last score. Settings
//! (reader preferences, pedal map, theme, keep-awake) live in the platform's config folder.
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::unimplemented,
    clippy::todo,
    clippy::unreachable
)]
// Release builds on Windows have no console window.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::path::PathBuf;

use scorecraft_devices::KeepAwake;
use scorecraft_ui_egui::{App, Platform, Settings};

/// The config folder: `%APPDATA%\ScoreCraft`, `~/Library/Application Support/ScoreCraft` or
/// `$XDG_CONFIG_HOME/scorecraft`.
fn config_dir() -> Option<PathBuf> {
    let env = |k: &str| {
        std::env::var_os(k)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    if cfg!(windows) {
        env("APPDATA").map(|d| d.join("ScoreCraft"))
    } else if cfg!(target_os = "macos") {
        env("HOME").map(|h| h.join("Library/Application Support/ScoreCraft"))
    } else {
        env("XDG_CONFIG_HOME")
            .or_else(|| env("HOME").map(|h| h.join(".config")))
            .map(|d| d.join("scorecraft"))
    }
}

fn settings_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("settings.json"))
}

fn load_settings() -> Settings {
    settings_path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|t| Settings::parse(&t))
        .unwrap_or_default()
}

struct Desktop {
    keep_awake: KeepAwake,
}

impl Platform for Desktop {
    fn pick_score(&mut self) -> Option<String> {
        #[cfg(any(windows, target_os = "macos"))]
        {
            rfd::FileDialog::new()
                .add_filter("PDF scores", &["pdf", "PDF"])
                .pick_file()
                .map(|p| p.to_string_lossy().into_owned())
        }
        #[cfg(not(any(windows, target_os = "macos")))]
        {
            None
        }
    }

    fn set_keep_awake(&mut self, on: bool) -> Result<(), String> {
        self.keep_awake.set(on).map_err(|e| e.to_string())
    }

    fn save_settings(&mut self, settings: &Settings) {
        let Some(path) = settings_path() else {
            return;
        };
        // Write beside, then rename: a crash mid-write never leaves a half file.
        let tmp = path.with_extension("json.tmp");
        let ok = path
            .parent()
            .is_some_and(|d| std::fs::create_dir_all(d).is_ok())
            && std::fs::write(&tmp, settings.to_json_string()).is_ok();
        if ok {
            let _ = std::fs::rename(&tmp, &path);
        }
    }
}

struct Shell {
    app: App,
}

impl eframe::App for Shell {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        // Last-resort guard (never-crash rule 7): a panic in a frame is reported and the reader
        // stays on screen.
        let app = &mut self.app;
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| app.ui(ui)));
        if r.is_err() {
            self.app.state.message =
                Some("Something went wrong drawing that frame. Your score is still open.".into());
        }
    }
}

fn main() -> std::process::ExitCode {
    let settings = load_settings();
    let arg = std::env::args().nth(1);
    let open = arg.clone().or_else(|| settings.last_score.clone());
    let mut app = App::new(
        Box::new(Desktop {
            keep_awake: KeepAwake::new(),
        }),
        settings,
    );
    if let Some(path) = open
        && let Err(e) = app.open_path(&path)
        && arg.is_some()
    {
        app.state.message = Some(e.to_string());
    }
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("ScoreCraft")
            .with_inner_size([1280.0, 860.0])
            .with_min_inner_size([480.0, 360.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    match eframe::run_native(
        "ScoreCraft",
        options,
        Box::new(|_cc| Ok(Box::new(Shell { app }))),
    ) {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("scorecraft: {e}");
            std::process::ExitCode::FAILURE
        }
    }
}
