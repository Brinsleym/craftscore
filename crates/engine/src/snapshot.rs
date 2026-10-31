//! A picture of what the reader shows, composed on the CPU from the cached page bitmaps
//! (`view.snapshot`). Agents and headless checks use it to see a turn without a window; the app's
//! `ui.screenshot` captures the real window instead.

use base64::Engine as _;
use scorecraft_geom::{Rect, Rotation};
use scorecraft_pdf::Bitmap;
use serde_json::{Value, json};

use crate::commands::{CommandSpec, bad, opt_str, params_object, reject_unknown_keys};
use crate::reader::FrameSlot;
use crate::score_cmds::score_open;
use crate::{EngineError, Session};

/// Largest snapshot, in pixels.
const MAX_PIXELS: u64 = 64_000_000;

pub fn specs() -> Vec<CommandSpec> {
    vec![CommandSpec {
        id: "view.snapshot",
        label: "Reader Snapshot",
        menu: "",
        shortcut: None,
        params: r##"{"out":path?,"background":"#rrggbb"="#808080"}"##,
        ours: true,
        enabled: score_open,
        run: snapshot,
    }]
}

/// Where turned-page fractions `(u, v)` come from in the unturned bitmap, for a page turned
/// clockwise by `r`.
pub fn unturn(r: Rotation, u: f64, v: f64) -> (f64, f64) {
    match r {
        Rotation::R0 => (u, v),
        Rotation::R90 => (v, 1.0 - u),
        Rotation::R180 => (1.0 - u, 1.0 - v),
        Rotation::R270 => (1.0 - v, u),
    }
}

fn parse_colour(id: &str, s: &str) -> Result<[u8; 3], EngineError> {
    let hex = s.strip_prefix('#').filter(|h| h.len() == 6 && h.is_ascii());
    let byte = |i: usize| {
        hex.and_then(|h| h.get(i..i + 2))
            .and_then(|b| u8::from_str_radix(b, 16).ok())
    };
    match (byte(0), byte(2), byte(4)) {
        (Some(r), Some(g), Some(b)) => Ok([r, g, b]),
        _ => Err(bad(id, "`background` must be #rrggbb".into())),
    }
}

/// Bilinear sample of `bm` at fractions `(x, y)`, premultiplied RGBA.
fn sample(bm: &Bitmap, x: f64, y: f64) -> [f64; 4] {
    let fx = (x * f64::from(bm.width) - 0.5).clamp(0.0, f64::from(bm.width.saturating_sub(1)));
    let fy = (y * f64::from(bm.height) - 0.5).clamp(0.0, f64::from(bm.height.saturating_sub(1)));
    let (x0, y0) = (fx.floor() as u32, fy.floor() as u32);
    let (x1, y1) = (
        (x0 + 1).min(bm.width.saturating_sub(1)),
        (y0 + 1).min(bm.height.saturating_sub(1)),
    );
    let (tx, ty) = (fx - f64::from(x0), fy - f64::from(y0));
    let px = |x, y| bm.pixel(x, y).unwrap_or([0, 0, 0, 0]);
    let (a, b, c, d) = (px(x0, y0), px(x1, y0), px(x0, y1), px(x1, y1));
    let mut out = [0.0; 4];
    for (o, (((a, b), c), d)) in out.iter_mut().zip(a.iter().zip(b).zip(c).zip(d)) {
        let top = f64::from(*a) * (1.0 - tx) + f64::from(b) * tx;
        let bottom = f64::from(c) * (1.0 - tx) + f64::from(d) * tx;
        *o = top * (1.0 - ty) + bottom * ty;
    }
    out
}

/// Composes `slots` onto a `width` × `height` canvas (device pixels; `scale` maps the slots'
/// frontend pixels to them). Returns premultiplied RGBA bytes.
pub fn compose(slots: &[FrameSlot], width: u32, height: u32, scale: f64, bg: [u8; 3]) -> Vec<u8> {
    let mut out: Vec<u8> =
        std::iter::repeat_n([bg[0], bg[1], bg[2], 255], width as usize * height as usize)
            .flatten()
            .collect();
    for slot in slots {
        let Some(bm) = slot.bitmap.as_deref() else {
            continue;
        };
        let d = Rect::new(
            slot.dst.x0 * scale,
            slot.dst.y0 * scale,
            slot.dst.x1 * scale,
            slot.dst.y1 * scale,
        );
        if !(d.width() > 0.0 && d.height() > 0.0) {
            continue;
        }
        let xs = d.x0.max(0.0).floor() as u32..(d.x1.min(f64::from(width))).ceil().max(0.0) as u32;
        let ys = d.y0.max(0.0).floor() as u32..(d.y1.min(f64::from(height))).ceil().max(0.0) as u32;
        for py in ys {
            let fy = (f64::from(py) + 0.5 - d.y0) / d.height();
            if !(0.0..=1.0).contains(&fy) {
                continue;
            }
            let v = slot.src.y0 + fy * slot.src.height();
            for px in xs.clone() {
                let fx = (f64::from(px) + 0.5 - d.x0) / d.width();
                if !(0.0..=1.0).contains(&fx) {
                    continue;
                }
                let u = slot.src.x0 + fx * slot.src.width();
                let (bx, by) = unturn(slot.rotation, u, v);
                let s = sample(bm, bx, by);
                let i = (py as usize * width as usize + px as usize) * 4;
                if let Some(dst) = out.get_mut(i..i + 4) {
                    let a = s[3] / 255.0;
                    for c in 0..3 {
                        if let (Some(o), Some(sc)) = (dst.get_mut(c), s.get(c)) {
                            *o = (sc + f64::from(*o) * (1.0 - a)).round().clamp(0.0, 255.0) as u8;
                        }
                    }
                }
            }
        }
    }
    out
}

fn snapshot(s: &mut Session, params: &Value) -> Result<Value, EngineError> {
    const ID: &str = "view.snapshot";
    let p = params_object(ID, params)?;
    reject_unknown_keys(ID, p, &["out", "background"])?;
    let bg = match opt_str(ID, p, "background")? {
        Some(c) => parse_colour(ID, c)?,
        None => [128, 128, 128],
    };
    let out = opt_str(ID, p, "out")?;
    if let Some(path) = out {
        crate::view_cmds::check_png_path(ID, path)?;
    }
    let (vp, ppp) = {
        let r = s.reader_mut(ID)?;
        (r.viewport, r.pixels_per_point)
    };
    let (w, h) = (
        (vp.width * ppp).round().max(1.0) as u32,
        (vp.height * ppp).round().max(1.0) as u32,
    );
    if u64::from(w) * u64::from(h) > MAX_PIXELS {
        return Err(bad(
            ID,
            format!("the reader area is over {MAX_PIXELS} pixels"),
        ));
    }
    let slots = s.reader_frame(true).unwrap_or_default();
    let rgba = compose(&slots, w, h, ppp, bg);
    let png = scorecraft_codecs::png::encode_premultiplied(w, h, &rgba)
        .map_err(|e| EngineError::Io(format!("encoding PNG: {e}")))?;
    let mut result = json!({ "width": w, "height": h, "out": out });
    match out {
        Some(path) => std::fs::write(path, &png).map_err(|e| {
            EngineError::Io(format!("{}: {e}", crate::commands::truncate(path, 256)))
        })?,
        None => crate::commands::set(
            &mut result,
            "png",
            json!(base64::engine::general_purpose::STANDARD.encode(&png)),
        ),
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use scorecraft_testkit::pdf::{PageSpec, PdfBuilder};

    #[test]
    fn unturn_inverts_each_rotation() {
        // The turned page's top-left corner comes from the unturned bitmap's corner the turn
        // brought there.
        assert_eq!(unturn(Rotation::R90, 0.0, 0.0), (0.0, 1.0));
        assert_eq!(unturn(Rotation::R180, 0.0, 0.0), (1.0, 1.0));
        assert_eq!(unturn(Rotation::R270, 0.0, 0.0), (1.0, 0.0));
        assert_eq!(unturn(Rotation::R0, 0.25, 0.75), (0.25, 0.75));
    }

    #[test]
    fn snapshot_shows_the_page_on_the_background() {
        let mut s = Session::new();
        let bytes = PdfBuilder::new().pages(2, PageSpec::default()).build();
        s.open_score_bytes(Arc::new(bytes), None, "t.pdf").unwrap();
        s.dispatch("view.viewport.set", &json!({ "width": 400, "height": 300 }))
            .unwrap();
        let r = s
            .dispatch("view.snapshot", &json!({ "background": "#ff0000" }))
            .unwrap();
        assert_eq!(
            (r["width"].as_u64(), r["height"].as_u64()),
            (Some(400), Some(300))
        );
        let png = base64::engine::general_purpose::STANDARD
            .decode(r["png"].as_str().unwrap())
            .unwrap();
        assert_eq!(&png[..4], b"\x89PNG");
        let slots = s.reader_frame(false).unwrap();
        let rgba = compose(&slots, 400, 300, 1.0, [255, 0, 0]);
        // The letterbox at the left edge is the background; the page centre is not red.
        assert_eq!(&rgba[..4], &[255, 0, 0, 255]);
        let c = (150 * 400 + 200) * 4;
        assert_ne!(&rgba[c..c + 3], &[255, 0, 0]);
        for bad_p in [json!({ "background": "red" }), json!({ "out": "x.jpg" })] {
            assert_eq!(
                s.dispatch("view.snapshot", &bad_p).unwrap_err().code(),
                "bad_params"
            );
        }
    }
}
