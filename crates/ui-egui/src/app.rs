//! The ScoreCraft shell: toolbar, reader and shell commands. Platform-free; the desktop and web
//! apps supply a [`Platform`] and call [`App::ui`] every frame.

use egui::{Align, Color32, Context, Layout, Rect, RichText, Sense, Stroke, Ui, Vec2, pos2, vec2};
use scorecraft_engine::{EngineError, Session};
use serde_json::{Value, json};

use crate::reader::ReaderCanvas;
use crate::settings::Settings;
use crate::state::{Theme, UiState};
use crate::theme::Tokens;

/// What the shell needs from the platform.
pub trait Platform {
    /// Asks the user for a score file. `None` if cancelled or there is no picker.
    fn pick_score(&mut self) -> Option<String> {
        None
    }
    /// Holds or releases the screen awake.
    fn set_keep_awake(&mut self, on: bool) -> Result<(), String>;
    /// Stores the settings for the next launch.
    fn save_settings(&mut self, _settings: &Settings) {}
}

/// A platform that does nothing (tests, snapshots).
pub struct NoPlatform;

impl Platform for NoPlatform {
    fn set_keep_awake(&mut self, _on: bool) -> Result<(), String> {
        Ok(())
    }
}

/// The shell commands (`ui.*`, AGENTS.md rule 2). Listed in `shell-commands.toml` for parity.
pub const SHELL_COMMANDS: &[(&str, &str, &str)] = &[
    ("ui.view.fullScreen", "Full Screen", r#"{"on":bool?}"#),
    (
        "ui.power.keepAwake",
        "Keep Screen Awake",
        r#"{"on":bool=true}"#,
    ),
    ("ui.theme.set", "Theme", r#"{"theme":"light|dark"}"#),
    (
        "ui.chrome.toggle",
        "Show or Hide Controls",
        r#"{"on":bool?}"#,
    ),
];

pub struct App {
    pub session: Session,
    pub state: UiState,
    pub settings: Settings,
    reader: ReaderCanvas,
    platform: Box<dyn Platform>,
    /// Full-screen state last sent to the window.
    sent_full_screen: Option<bool>,
    /// The theme last applied to egui.
    applied_theme: Option<Theme>,
}

fn bool_param(params: &Value, key: &str) -> Result<Option<bool>, EngineError> {
    match params.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::Bool(b)) => Ok(Some(*b)),
        Some(_) => Err(EngineError::BadParams {
            command: "ui".into(),
            message: format!("`{key}` must be true or false"),
        }),
    }
}

impl App {
    pub fn new(platform: Box<dyn Platform>, settings: Settings) -> App {
        let mut session = Session::new();
        settings.apply_to_session(&mut session);
        let mut state = UiState::new();
        state.theme = settings.theme;
        state.keep_awake.wanted = settings.keep_awake;
        App {
            session,
            state,
            settings,
            reader: ReaderCanvas::new(),
            platform,
            sent_full_screen: None,
            applied_theme: None,
        }
    }

    /// Opens a score file and applies the saved reader preferences.
    pub fn open_path(&mut self, path: &str) -> Result<Value, EngineError> {
        let r = self
            .session
            .dispatch("score.open", &json!({ "path": path }))?;
        self.reader.clear();
        self.settings.apply_to_reader(&mut self.session);
        self.settings.last_score = Some(path.to_owned());
        self.save();
        Ok(r)
    }

    fn save(&mut self) {
        self.settings.capture(&mut self.session, &self.state);
        self.platform.save_settings(&self.settings);
    }

    /// Runs a shell (`ui.*`) or engine command by id: what the toolbar, menus and the control
    /// channel call.
    pub fn command(&mut self, id: &str, params: &Value) -> Result<Value, EngineError> {
        let r = if id.starts_with("ui.") {
            self.shell(id, params)
        } else {
            self.session.dispatch(id, params)
        };
        let changes_prefs = id.starts_with("view.layout")
            || id.starts_with("view.turn.halfPage")
            || id.starts_with("view.turn.alternating")
            || id == "view.zoom.landscapeFit"
            || id == "input.pedal.map"
            || id == "ui.theme.set"
            || id == "ui.power.keepAwake";
        if r.is_ok() && changes_prefs {
            self.save();
        }
        r
    }

    fn shell(&mut self, id: &str, params: &Value) -> Result<Value, EngineError> {
        if !(params.is_null() || params.is_object()) {
            return Err(EngineError::BadParams {
                command: id.chars().take(64).collect(),
                message: "params must be an object".into(),
            });
        }
        match id {
            "ui.view.fullScreen" => {
                let on = bool_param(params, "on")?.unwrap_or(!self.state.full_screen);
                self.state.full_screen = on;
                self.state.chrome = !on;
            }
            "ui.power.keepAwake" => {
                self.state.keep_awake.wanted = bool_param(params, "on")?.unwrap_or(true);
            }
            "ui.theme.set" => {
                let t = params
                    .get("theme")
                    .and_then(Value::as_str)
                    .and_then(Theme::from_id)
                    .ok_or_else(|| EngineError::BadParams {
                        command: id.into(),
                        message: "`theme` must be light or dark".into(),
                    })?;
                self.state.theme = t;
            }
            "ui.chrome.toggle" => {
                self.state.chrome = bool_param(params, "on")?.unwrap_or(!self.state.chrome);
            }
            _ => return Err(EngineError::UnknownCommand(id.chars().take(128).collect())),
        }
        serde_json::to_value(&self.state).map_err(|e| EngineError::Io(e.to_string()))
    }

    /// Keeps the screen awake while wanted and a score is open (READ-9).
    fn sync_keep_awake(&mut self) {
        let want = self.state.keep_awake.wanted && self.session.has_score();
        if want == self.state.keep_awake.active {
            return;
        }
        match self.platform.set_keep_awake(want) {
            Ok(()) => {
                self.state.keep_awake.active = want;
                self.state.keep_awake.error = None;
            }
            Err(e) => {
                // Don't retry every frame: record it and stop wanting until asked again.
                self.state.keep_awake.error = Some(e);
                self.state.keep_awake.wanted = false;
            }
        }
    }

    /// One frame of the whole shell, in the window's root `ui`.
    pub fn ui(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let tokens = Tokens::for_theme(self.state.theme);
        if self.applied_theme != Some(self.state.theme) {
            tokens.apply(self.state.theme, &ctx);
            self.applied_theme = Some(self.state.theme);
        }
        if self.sent_full_screen != Some(self.state.full_screen) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(self.state.full_screen));
            self.sent_full_screen = Some(self.state.full_screen);
        }
        self.sync_keep_awake();
        self.shortcuts(&ctx);
        self.dropped_files(&ctx);

        if self.state.chrome {
            egui::Panel::top("toolbar")
                .exact_size(tokens.toolbar_height)
                .resizable(false)
                .frame(
                    egui::Frame::NONE
                        .fill(tokens.toolbar_bg)
                        .inner_margin(egui::Margin::symmetric(8, 4)),
                )
                .show(ui, |ui| self.toolbar(ui, &tokens));
        }
        if let Some(msg) = self.state.message.clone() {
            egui::Panel::bottom("message")
                .resizable(false)
                .frame(
                    egui::Frame::NONE
                        .fill(tokens.toolbar_bg)
                        .inner_margin(egui::Margin::same(8)),
                )
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(RichText::new(msg).color(tokens.error));
                        if ui.button("Dismiss").clicked() {
                            self.state.message = None;
                        }
                    });
                });
        }
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(tokens.reader_bg))
            .show(ui, |ui| {
                if !self.session.has_score() {
                    self.empty(ui, &tokens);
                    return;
                }
                let keys = !ctx.egui_wants_keyboard_input();
                let out = self.reader.show(ui, &mut self.session, &tokens, keys);
                if out.toggle_chrome {
                    self.state.chrome = !self.state.chrome;
                }
                if let Some(e) = out.error {
                    self.state.message = Some(e);
                }
            });
    }

    fn report(&mut self, r: Result<Value, EngineError>) {
        match r {
            Ok(_) => self.state.message = None,
            Err(e) => self.state.message = Some(e.to_string()),
        }
    }

    /// Shell shortcuts; turn keys go to the reader's pedal map instead.
    fn shortcuts(&mut self, ctx: &Context) {
        let (open, full, esc) = ctx.input(|i| {
            (
                i.modifiers.command && i.key_pressed(egui::Key::O),
                i.key_pressed(egui::Key::F11),
                i.key_pressed(egui::Key::Escape),
            )
        });
        if open {
            self.pick_and_open();
        }
        if full {
            let r = self.command("ui.view.fullScreen", &json!({}));
            self.report(r);
        }
        if esc && self.state.full_screen {
            let r = self.command("ui.view.fullScreen", &json!({ "on": false }));
            self.report(r);
        }
    }

    fn dropped_files(&mut self, ctx: &Context) {
        let dropped = ctx.input(|i| i.raw.dropped_files.first().map(|f| f.path().to_path_buf()));
        if let Some(path) = dropped {
            let r = self.open_path(&path.to_string_lossy());
            self.report(r);
        }
    }

    fn pick_and_open(&mut self) {
        if let Some(path) = self.platform.pick_score() {
            let r = self.open_path(&path);
            self.report(r);
        }
    }

    fn empty(&mut self, ui: &mut Ui, tokens: &Tokens) {
        ui.with_layout(
            Layout::centered_and_justified(egui::Direction::TopDown),
            |ui| {
                ui.label(
                    RichText::new("Open a score (Ctrl+O) or drop a PDF here")
                        .size(20.0)
                        .color(Color32::from_gray(230)),
                );
            },
        );
        let _ = tokens;
    }

    fn icon_button(
        &mut self,
        ui: &mut Ui,
        tokens: &Tokens,
        label: &str,
        selected: bool,
        icon: Icon,
    ) -> bool {
        let style = tokens.icon_button();
        let size = Vec2::splat(tokens.button);
        let r = craft_ui::buttons::IconButton::new(label, size, &style)
            .selected(selected)
            .show(ui, |ui, rect, colour| paint_icon(ui, rect, colour, icon))
            .on_hover_text(label);
        r.clicked()
    }

    fn toolbar(&mut self, ui: &mut Ui, tokens: &Tokens) {
        let st = self.session.reader_state().unwrap_or(Value::Null);
        let s = |k: &str| st.get(k).and_then(Value::as_str).unwrap_or("").to_owned();
        let layout = s("layout");
        let mode = s("mode");
        let fit = st
            .pointer("/zoom/fit")
            .and_then(Value::as_str)
            .unwrap_or("page")
            .to_owned();
        let zoom = st
            .pointer("/zoom/factor")
            .and_then(Value::as_f64)
            .unwrap_or(1.0);
        let has = self.session.has_score();
        let mut run: Vec<(&str, Value)> = Vec::new();
        ui.horizontal_centered(|ui| {
            if self.icon_button(ui, tokens, "Open Score…", false, Icon::Open) {
                self.pick_and_open();
            }
            ui.add_enabled_ui(has, |ui| {
                if self.icon_button(ui, tokens, "Previous Page", false, Icon::Previous) {
                    run.push(("view.turn.previous", json!({})));
                }
                let page = st.get("page").and_then(Value::as_u64).unwrap_or(0);
                let count = st.get("pageCount").and_then(Value::as_u64).unwrap_or(0);
                ui.label(
                    RichText::new(format!("{page} / {count}"))
                        .size(16.0)
                        .color(tokens.text),
                );
                if self.icon_button(ui, tokens, "Next Page", false, Icon::Next) {
                    run.push(("view.turn.next", json!({})));
                }
                ui.separator();
                if self.icon_button(ui, tokens, "Single Page", layout == "single", Icon::Single) {
                    run.push(("view.layout.single", json!({})));
                }
                if self.icon_button(ui, tokens, "Two Pages", layout == "twoPage", Icon::Two) {
                    run.push(("view.layout.twoPage", json!({})));
                }
                if layout == "twoPage" {
                    let fa = st
                        .get("firstAlone")
                        .and_then(Value::as_bool)
                        .unwrap_or(false);
                    if self.icon_button(ui, tokens, "First Page Alone", fa, Icon::FirstAlone) {
                        run.push(("view.layout.twoPage", json!({ "firstAlone": !fa })));
                    }
                }
                ui.separator();
                let half = mode == "halfPage";
                if self.icon_button(ui, tokens, "Half-Page Turns", half, Icon::Half) {
                    run.push(("view.turn.halfPage", json!({ "on": !half })));
                }
                let alt = mode == "alternating";
                if self.icon_button(ui, tokens, "Alternating Turns", alt, Icon::Alternate) {
                    run.push(("view.turn.alternating", json!({ "on": !alt })));
                }
                ui.separator();
                if self.icon_button(
                    ui,
                    tokens,
                    "Landscape Page Zoom",
                    fit == "width",
                    Icon::FitWidth,
                ) {
                    run.push(("view.zoom.landscapeFit", json!({ "on": fit != "width" })));
                }
                if self.icon_button(ui, tokens, "Zoom Out", false, Icon::ZoomOut) {
                    run.push(("view.zoom.set", json!({ "zoom": (zoom / 1.25).max(1.0) })));
                }
                if self.icon_button(ui, tokens, "Zoom In", false, Icon::ZoomIn) {
                    run.push(("view.zoom.set", json!({ "zoom": (zoom * 1.25).min(8.0) })));
                }
            });
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let dark = self.state.theme == Theme::Dark;
                if self.icon_button(ui, tokens, "Dark Theme", dark, Icon::Theme) {
                    run.push((
                        "ui.theme.set",
                        json!({ "theme": if dark { "light" } else { "dark" } }),
                    ));
                }
                let awake = self.state.keep_awake.wanted;
                let awake_label = match &self.state.keep_awake.error {
                    Some(e) => format!("Keep Screen Awake ({e})"),
                    None => "Keep Screen Awake".to_owned(),
                };
                if self.icon_button(ui, tokens, &awake_label, awake, Icon::Awake) {
                    run.push(("ui.power.keepAwake", json!({ "on": !awake })));
                }
                if self.icon_button(
                    ui,
                    tokens,
                    "Full Screen (F11)",
                    self.state.full_screen,
                    Icon::FullScreen,
                ) {
                    run.push(("ui.view.fullScreen", json!({})));
                }
            });
        });
        for (id, p) in run {
            let r = self.command(id, &p);
            self.report(r);
        }
    }
}

/// The toolbar's icons, painted with shapes so the shell ships no icon assets.
#[derive(Debug, Clone, Copy)]
enum Icon {
    Open,
    Previous,
    Next,
    Single,
    Two,
    FirstAlone,
    Half,
    Alternate,
    FitWidth,
    ZoomIn,
    ZoomOut,
    FullScreen,
    Awake,
    Theme,
}

fn paint_icon(ui: &Ui, rect: Rect, c: Color32, icon: Icon) {
    let p = ui.painter();
    let r = rect.shrink(rect.width() * 0.28);
    let line = Stroke::new(1.8, c);
    let page = |x0: f32, x1: f32| Rect::from_min_max(pos2(x0, r.top()), pos2(x1, r.bottom()));
    match icon {
        Icon::Open => {
            let body = Rect::from_min_max(pos2(r.left(), r.top() + r.height() * 0.25), r.max);
            p.rect_stroke(body, 2.0, line, egui::StrokeKind::Middle);
            p.line_segment(
                [
                    pos2(r.left(), body.top()),
                    pos2(r.left() + r.width() * 0.4, r.top()),
                ],
                line,
            );
            p.line_segment(
                [
                    pos2(r.left() + r.width() * 0.4, r.top()),
                    pos2(r.center().x, body.top()),
                ],
                line,
            );
        }
        Icon::Previous | Icon::Next => {
            let (a, b) = if matches!(icon, Icon::Next) {
                (r.left(), r.right())
            } else {
                (r.right(), r.left())
            };
            let pts = vec![pos2(a, r.top()), pos2(b, r.center().y), pos2(a, r.bottom())];
            p.add(egui::Shape::convex_polygon(pts, c, Stroke::NONE));
        }
        Icon::Single => {
            p.rect_stroke(
                page(r.left() + r.width() * 0.2, r.right() - r.width() * 0.2),
                1.0,
                line,
                egui::StrokeKind::Middle,
            );
        }
        Icon::Two | Icon::FirstAlone => {
            let m = r.center().x;
            p.rect_stroke(page(r.left(), m - 1.0), 1.0, line, egui::StrokeKind::Middle);
            if matches!(icon, Icon::Two) {
                p.rect_stroke(
                    page(m + 1.0, r.right()),
                    1.0,
                    line,
                    egui::StrokeKind::Middle,
                );
            } else {
                p.text(
                    pos2(m + r.width() * 0.25, r.center().y),
                    egui::Align2::CENTER_CENTER,
                    "1",
                    egui::FontId::proportional(r.height() * 0.8),
                    c,
                );
            }
        }
        Icon::Half => {
            let pg = page(r.left() + r.width() * 0.2, r.right() - r.width() * 0.2);
            p.rect_stroke(pg, 1.0, line, egui::StrokeKind::Middle);
            p.rect_filled(
                Rect::from_min_max(pg.min, pos2(pg.right(), pg.center().y)),
                1.0,
                c.gamma_multiply(0.5),
            );
        }
        Icon::Alternate => {
            let m = r.center().x;
            p.rect_filled(page(r.left(), m - 1.0), 1.0, c.gamma_multiply(0.5));
            p.rect_stroke(
                page(m + 1.0, r.right()),
                1.0,
                line,
                egui::StrokeKind::Middle,
            );
        }
        Icon::FitWidth => {
            let y = r.center().y;
            p.line_segment([pos2(r.left(), y), pos2(r.right(), y)], line);
            for (x, d) in [(r.left(), 1.0), (r.right(), -1.0)] {
                p.line_segment([pos2(x, y), pos2(x + d * 5.0, y - 5.0)], line);
                p.line_segment([pos2(x, y), pos2(x + d * 5.0, y + 5.0)], line);
            }
            p.line_segment([pos2(r.left(), r.top()), pos2(r.left(), r.bottom())], line);
            p.line_segment(
                [pos2(r.right(), r.top()), pos2(r.right(), r.bottom())],
                line,
            );
        }
        Icon::ZoomIn | Icon::ZoomOut => {
            let rad = r.width() * 0.38;
            let ctr = r.center() - vec2(2.0, 2.0);
            p.circle_stroke(ctr, rad, line);
            p.line_segment([ctr + vec2(rad * 0.7, rad * 0.7), r.right_bottom()], line);
            p.line_segment(
                [ctr - vec2(rad * 0.5, 0.0), ctr + vec2(rad * 0.5, 0.0)],
                line,
            );
            if matches!(icon, Icon::ZoomIn) {
                p.line_segment(
                    [ctr - vec2(0.0, rad * 0.5), ctr + vec2(0.0, rad * 0.5)],
                    line,
                );
            }
        }
        Icon::FullScreen => {
            let k = r.width() * 0.35;
            for (corner, dx, dy) in [
                (r.left_top(), 1.0, 1.0),
                (r.right_top(), -1.0, 1.0),
                (r.left_bottom(), 1.0, -1.0),
                (r.right_bottom(), -1.0, -1.0),
            ] {
                p.line_segment([corner, corner + vec2(dx * k, 0.0)], line);
                p.line_segment([corner, corner + vec2(0.0, dy * k)], line);
            }
        }
        Icon::Awake => {
            let ctr = r.center();
            p.circle_stroke(ctr, r.width() * 0.22, line);
            for i in 0..8 {
                let a = i as f32 * std::f32::consts::FRAC_PI_4;
                let d = vec2(a.cos(), a.sin());
                p.line_segment(
                    [ctr + d * r.width() * 0.34, ctr + d * r.width() * 0.5],
                    line,
                );
            }
        }
        Icon::Theme => {
            let ctr = r.center();
            let rad = r.width() * 0.45;
            p.circle_stroke(ctr, rad, line);
            let half: Vec<egui::Pos2> = (0..=16)
                .map(|i| {
                    let a = -std::f32::consts::FRAC_PI_2 + i as f32 * std::f32::consts::PI / 16.0;
                    ctr + vec2(a.cos(), a.sin()) * rad
                })
                .collect();
            p.add(egui::Shape::convex_polygon(half, c, Stroke::NONE));
        }
    }
    let _ = Sense::hover();
}

#[cfg(test)]
mod tests {
    use super::*;
    use scorecraft_testkit::pdf::{PageSpec, PdfBuilder};

    fn app_with_score(pages: usize) -> (App, std::path::PathBuf) {
        let dir =
            std::env::temp_dir().join(format!("scorecraft-ui-{}-{pages}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("score.pdf");
        std::fs::write(
            &path,
            PdfBuilder::new().pages(pages, PageSpec::default()).build(),
        )
        .unwrap();
        let mut app = App::new(Box::new(NoPlatform), Settings::default());
        app.open_path(&path.to_string_lossy()).unwrap();
        (app, path)
    }

    /// Runs `frames` frames of the shell headless at 1280 × 800.
    fn frames(app: &mut App, ctx: &Context, frames: usize, events: Vec<egui::Event>) {
        for i in 0..frames {
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(1280.0, 800.0))),
                events: if i == 0 { events.clone() } else { Vec::new() },
                ..Default::default()
            };
            let mut out = ctx.run_ui(input, |ui| app.ui(ui));
            // No renderer here: drop the texture uploads on purpose.
            out.textures_delta.clear();
        }
    }

    #[test]
    fn shell_commands_match_the_parity_list() {
        let toml_text = include_str!("../shell-commands.toml");
        let v: toml::Value = toml::from_str(toml_text).unwrap();
        let listed: Vec<&str> = v["commands"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| c.as_str().unwrap())
            .collect();
        let ours: Vec<&str> = SHELL_COMMANDS.iter().map(|c| c.0).collect();
        assert_eq!(listed, ours);
    }

    #[test]
    fn shell_commands_change_state_and_reject_bad_params() {
        let mut app = App::new(Box::new(NoPlatform), Settings::default());
        let r = app.command("ui.view.fullScreen", &json!({})).unwrap();
        assert_eq!(
            (r["fullScreen"].as_bool(), r["chrome"].as_bool()),
            (Some(true), Some(false))
        );
        let r = app.command("ui.chrome.toggle", &json!({})).unwrap();
        assert_eq!(r["chrome"], true);
        let r = app
            .command("ui.theme.set", &json!({ "theme": "dark" }))
            .unwrap();
        assert_eq!(r["theme"], "dark");
        let r = app
            .command("ui.power.keepAwake", &json!({ "on": false }))
            .unwrap();
        assert_eq!(r["keepAwake"]["wanted"], false);
        for (id, p) in [
            ("ui.theme.set", json!({ "theme": "neon" })),
            ("ui.view.fullScreen", json!({ "on": 1 })),
            ("ui.view.fullScreen", json!([1])),
        ] {
            assert_eq!(
                app.command(id, &p).unwrap_err().code(),
                "bad_params",
                "{id}"
            );
        }
        assert_eq!(
            app.command("ui.nope", &json!({})).unwrap_err().code(),
            "unknown_command"
        );
    }

    #[test]
    fn frames_draw_and_keys_turn() {
        let (mut app, _) = app_with_score(3);
        let ctx = Context::default();
        frames(&mut app, &ctx, 2, Vec::new());
        let st = app.session.reader_state().unwrap();
        assert_eq!(st["page"], 1);
        // The reader area is the window minus the toolbar.
        let vp = st["viewport"]["height"].as_f64().unwrap();
        assert!(vp > 700.0 && vp < 800.0, "{vp}");
        let key = egui::Event::Key {
            key: egui::Key::PageDown,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        };
        frames(&mut app, &ctx, 1, vec![key]);
        assert_eq!(app.session.reader_state().unwrap()["page"], 2);
        // A tap in the right third turns forward.
        let pos = pos2(1200.0, 400.0);
        let press = |pressed| egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        };
        frames(
            &mut app,
            &ctx,
            1,
            vec![egui::Event::PointerMoved(pos), press(true)],
        );
        frames(&mut app, &ctx, 2, vec![press(false)]);
        assert_eq!(app.session.reader_state().unwrap()["page"], 3);
    }

    #[test]
    fn keep_awake_follows_the_open_score() {
        struct Rec(std::sync::Arc<std::sync::Mutex<Vec<bool>>>);
        impl Platform for Rec {
            fn set_keep_awake(&mut self, on: bool) -> Result<(), String> {
                self.0.lock().unwrap().push(on);
                Ok(())
            }
        }
        let calls = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let mut app = App::new(Box::new(Rec(calls.clone())), Settings::default());
        let ctx = Context::default();
        frames(&mut app, &ctx, 1, Vec::new());
        assert!(
            calls.lock().unwrap().is_empty(),
            "nothing open, nothing held"
        );
        let (_, path) = app_with_score(2);
        app.open_path(&path.to_string_lossy()).unwrap();
        frames(&mut app, &ctx, 1, Vec::new());
        app.command("ui.power.keepAwake", &json!({ "on": false }))
            .unwrap();
        frames(&mut app, &ctx, 1, Vec::new());
        assert_eq!(*calls.lock().unwrap(), vec![true, false]);
    }

    #[test]
    fn a_platform_that_cannot_keep_awake_is_reported_once() {
        struct Never;
        impl Platform for Never {
            fn set_keep_awake(&mut self, _: bool) -> Result<(), String> {
                Err("not supported".into())
            }
        }
        let (_, path) = app_with_score(2);
        let mut app = App::new(Box::new(Never), Settings::default());
        app.open_path(&path.to_string_lossy()).unwrap();
        let ctx = Context::default();
        frames(&mut app, &ctx, 3, Vec::new());
        assert_eq!(app.state.keep_awake.error.as_deref(), Some("not supported"));
        assert!(!app.state.keep_awake.wanted);
    }
}
