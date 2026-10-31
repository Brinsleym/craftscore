//! The reader canvas: draws the engine's frame and turns taps, swipes, keys, pinches and scrolls
//! into engine commands. It holds textures only; where the reader is lives in the engine.

use std::collections::HashMap;
use std::sync::Arc;

use egui::epaint::Vertex;
use egui::{
    Color32, ColorImage, Context, Event, Key, Mesh, Pos2, Rect, Response, Sense, Shape,
    TextureHandle, TextureOptions, Ui, Vec2, pos2,
};
use scorecraft_engine::{FrameSlot, Session, unturn};
use serde_json::{Value, json};

use crate::theme::Tokens;

/// What the canvas asks the shell to do after a frame.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Outcome {
    /// A tap in the middle third: show or hide the controls.
    pub toggle_chrome: bool,
    /// An engine error to show.
    pub error: Option<String>,
}

#[derive(Default)]
pub struct ReaderCanvas {
    /// One texture per file page: the bitmap it was made from (by address) and the texture.
    textures: HashMap<usize, (usize, TextureHandle)>,
    /// Pointer press position while a drag or tap is in progress.
    press: Option<Pos2>,
    /// The drag panned the zoomed page, so its release is not a swipe.
    panned: bool,
    /// Last reader area sent to the engine (points) and pixels per point.
    sent_viewport: Option<(Vec2, f32)>,
}

/// The engine's name for an egui key, with modifiers.
pub fn chord_name(key: Key, m: egui::Modifiers) -> String {
    let mut s = String::new();
    if m.command || m.ctrl {
        s.push_str("Ctrl+");
    }
    if m.alt {
        s.push_str("Alt+");
    }
    if m.shift {
        s.push_str("Shift+");
    }
    s.push_str(key.name());
    s
}

/// Records an engine error for the shell.
fn note(out: &mut Outcome, r: Result<Value, scorecraft_engine::EngineError>) -> Option<Value> {
    match r {
        Ok(v) => Some(v),
        Err(e) => {
            out.error = Some(e.to_string());
            None
        }
    }
}

impl ReaderCanvas {
    pub fn new() -> ReaderCanvas {
        ReaderCanvas::default()
    }

    /// Whether every visible page has its bitmap at the right scale (nothing still rendering).
    pub fn pages_ready(session: &mut Session) -> bool {
        session
            .reader_frame(false)
            .is_none_or(|slots| slots.iter().all(|s| s.bitmap.is_some()))
    }

    /// Forgets every texture (a new score).
    pub fn clear(&mut self) {
        self.textures.clear();
        // The engine's new reader starts with a default viewport: send ours again.
        self.sent_viewport = None;
        self.press = None;
    }

    /// Draws the reader in `ui`'s remaining space and handles its input. `keys` is false while
    /// a text field or dialog has the keyboard.
    pub fn show(
        &mut self,
        ui: &mut Ui,
        session: &mut Session,
        tokens: &Tokens,
        keys: bool,
    ) -> Outcome {
        let mut out = Outcome::default();
        let rect = ui.available_rect_before_wrap();
        let response = ui.allocate_rect(rect, Sense::click_and_drag());
        ui.painter().rect_filled(rect, 0.0, tokens.reader_bg);
        if !session.has_score() {
            return out;
        }
        let ctx = ui.ctx().clone();
        self.sync_viewport(&ctx, rect, session, &mut out);
        self.input(&ctx, &response, rect, session, keys, &mut out);
        self.paint(ui, &ctx, rect, session, tokens);
        out
    }

    fn sync_viewport(
        &mut self,
        ctx: &Context,
        rect: Rect,
        session: &mut Session,
        out: &mut Outcome,
    ) {
        let vp = (rect.size(), ctx.pixels_per_point());
        if self.sent_viewport != Some(vp) && vp.0.x >= 1.0 && vp.0.y >= 1.0 {
            note(
                out,
                session.dispatch(
                    "view.viewport.set",
                    &json!({ "width": vp.0.x, "height": vp.0.y, "pixelsPerPoint": vp.1 }),
                ),
            );
            self.sent_viewport = Some(vp);
        }
    }

    fn zoom_factor(session: &mut Session) -> f64 {
        session
            .reader_state()
            .and_then(|s| s.pointer("/zoom/factor").and_then(Value::as_f64))
            .unwrap_or(1.0)
    }

    fn input(
        &mut self,
        ctx: &Context,
        response: &Response,
        rect: Rect,
        session: &mut Session,
        keys: bool,
        out: &mut Outcome,
    ) {
        if keys {
            let events = ctx.input(|i| i.events.clone());
            for e in events {
                if let Event::Key {
                    key,
                    pressed: true,
                    repeat,
                    modifiers,
                    ..
                } = e
                {
                    let name = chord_name(key, modifiers);
                    note(
                        out,
                        session.dispatch("input.key", &json!({ "key": name, "repeat": repeat })),
                    );
                }
            }
        }
        // Pinch (or Ctrl + scroll) zooms; scrolling a zoomed page pans it.
        let (pinch, scroll) = ctx.input(|i| (i.zoom_delta(), i.smooth_scroll_delta));
        if response.hovered() || response.contains_pointer() {
            if (pinch - 1.0).abs() > 1e-3 {
                let z = (Self::zoom_factor(session) * f64::from(pinch)).clamp(1.0, 8.0);
                note(
                    out,
                    session.dispatch("view.zoom.set", &json!({ "zoom": z })),
                );
            } else if scroll != Vec2::ZERO && Self::zoom_factor(session) > 1.0 {
                note(
                    out,
                    session.dispatch("view.pan", &json!({ "dx": scroll.x, "dy": scroll.y })),
                );
            }
        }
        if response.drag_started() || (response.is_pointer_button_down_on() && self.press.is_none())
        {
            self.press = response.interact_pointer_pos();
            self.panned = false;
        }
        if response.dragged() && Self::zoom_factor(session) > 1.0 {
            let d = response.drag_delta();
            if d != Vec2::ZERO {
                self.panned = true;
                note(
                    out,
                    session.dispatch("view.pan", &json!({ "dx": d.x, "dy": d.y })),
                );
            }
        }
        if response.clicked() {
            let x = response
                .interact_pointer_pos()
                .map_or(0.0, |p| p.x - rect.min.x);
            if let Some(r) = note(out, session.dispatch("input.tap", &json!({ "x": x }))) {
                out.toggle_chrome = r.get("zone").and_then(Value::as_str) == Some("controls");
            }
            self.press = None;
        } else if response.drag_stopped() {
            if let (Some(start), Some(end), false) =
                (self.press, response.interact_pointer_pos(), self.panned)
            {
                let d = end - start;
                note(
                    out,
                    session.dispatch("input.swipe", &json!({ "dx": d.x, "dy": d.y })),
                );
            }
            self.press = None;
        }
    }

    fn texture(&mut self, ctx: &Context, slot: &FrameSlot) -> Option<egui::TextureId> {
        if let Some(bm) = &slot.bitmap {
            let addr = Arc::as_ptr(bm) as usize;
            let fresh = self
                .textures
                .get(&slot.file_page)
                .is_none_or(|(a, _)| *a != addr);
            if fresh {
                let size = [bm.width as usize, bm.height as usize];
                let image = ColorImage::from_rgba_premultiplied(size, bm.bytes());
                let tex = ctx.load_texture(
                    format!("page-{}", slot.file_page),
                    image,
                    TextureOptions::LINEAR,
                );
                self.textures.insert(slot.file_page, (addr, tex));
            }
        }
        // A texture at another scale stands in until the new one is ready.
        self.textures.get(&slot.file_page).map(|(_, t)| t.id())
    }

    fn paint(
        &mut self,
        ui: &Ui,
        ctx: &Context,
        rect: Rect,
        session: &mut Session,
        tokens: &Tokens,
    ) {
        // Native builds pre-render in the background, so a frame never waits on rendering: a
        // page not ready yet shows its last texture or a placeholder until it arrives. The web
        // build has no worker thread and renders here.
        let render_now = cfg!(target_arch = "wasm32");
        let slots = session.reader_frame(false).unwrap_or_default();
        let slots = if render_now
            && slots
                .iter()
                .any(|s| s.bitmap.is_none() && !self.textures.contains_key(&s.file_page))
        {
            session.reader_frame(true).unwrap_or_default()
        } else {
            slots
        };
        if slots.iter().any(|s| s.bitmap.is_none()) {
            ctx.request_repaint_after(std::time::Duration::from_millis(30));
        }
        let painter = ui.painter_at(rect);
        let on_screen: Vec<usize> = slots.iter().map(|s| s.file_page).collect();
        for slot in &slots {
            let d = &slot.dst;
            let dst = Rect::from_min_max(
                rect.min + egui::vec2(d.x0 as f32, d.y0 as f32),
                rect.min + egui::vec2(d.x1 as f32, d.y1 as f32),
            );
            let Some(tex) = self.texture(ctx, slot) else {
                painter.rect_filled(dst, 0.0, tokens.placeholder);
                continue;
            };
            let s = &slot.src;
            let corner = |u: f64, v: f64| {
                let (x, y) = unturn(slot.rotation, u, v);
                pos2(x as f32, y as f32)
            };
            let mut mesh = Mesh::with_texture(tex);
            for (pos, uv) in [
                (dst.left_top(), corner(s.x0, s.y0)),
                (dst.right_top(), corner(s.x1, s.y0)),
                (dst.right_bottom(), corner(s.x1, s.y1)),
                (dst.left_bottom(), corner(s.x0, s.y1)),
            ] {
                mesh.vertices.push(Vertex {
                    pos,
                    uv,
                    color: Color32::WHITE,
                });
            }
            mesh.indices.extend([0, 1, 2, 0, 2, 3]);
            painter.add(Shape::mesh(mesh));
        }
        // A half-page turn shows parts of two pages one above the other: mark the seam.
        for a in &slots {
            for b in &slots {
                if a.file_page != b.file_page
                    && (a.dst.y1 - b.dst.y0).abs() < 0.5
                    && a.dst.x0 < b.dst.x1
                    && b.dst.x0 < a.dst.x1
                {
                    let y = rect.min.y + b.dst.y0 as f32;
                    let x0 = rect.min.x + a.dst.x0.max(b.dst.x0) as f32;
                    let x1 = rect.min.x + a.dst.x1.min(b.dst.x1) as f32;
                    painter.line_segment(
                        [pos2(x0, y), pos2(x1, y)],
                        egui::Stroke::new(2.0, tokens.accent),
                    );
                }
            }
        }
        // Keep textures for what is on screen only; the engine's cache holds the rest.
        self.textures.retain(|page, _| on_screen.contains(page));
    }
}
