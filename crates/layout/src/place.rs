//! Placing a [`View`] on screen: which part of which page goes where, at what scale.

use scorecraft_geom::{Rect, Size};

use crate::View;

/// A page's displayed size in points (after rotation).
pub type PageSize = Size;
/// The reader area, in the frontend's pixels.
pub type Viewport = Size;

/// `x`, `y`, `w`, `h` as a [`Rect`].
fn xywh(x: f64, y: f64, w: f64, h: f64) -> Rect {
    Rect::new(x, y, x + w, y + h)
}

/// What 100% zoom fits to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fit {
    /// The whole spread fits on screen.
    Page,
    /// The spread fills the width and scrolls vertically (READ-12, Landscape Page Zoom).
    Width,
}

impl Fit {
    pub fn id(self) -> &'static str {
        match self {
            Fit::Page => "page",
            Fit::Width => "width",
        }
    }
}

/// Lowest and highest zoom factor over the fit.
pub const ZOOM_RANGE: (f64, f64) = (1.0, 8.0);

/// Zoom and pan (READ-15). `pan` positions content larger than the screen: 0 shows its left or
/// top edge, 1 its right or bottom edge, 0.5 its middle.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Zoom {
    pub fit: Fit,
    pub factor: f64,
    pub pan: (f64, f64),
}

impl Default for Zoom {
    fn default() -> Self {
        Zoom {
            fit: Fit::Page,
            factor: 1.0,
            pan: (0.5, 0.0),
        }
    }
}

impl Zoom {
    /// Clamps the factor to [`ZOOM_RANGE`] and the pan to 0..=1; anything not a number resets.
    pub fn sanitised(self) -> Zoom {
        let factor = if self.factor.is_finite() {
            self.factor.clamp(ZOOM_RANGE.0, ZOOM_RANGE.1)
        } else {
            1.0
        };
        let unit = |v: f64, d: f64| if v.is_finite() { v.clamp(0.0, 1.0) } else { d };
        Zoom {
            fit: self.fit,
            factor,
            pan: (unit(self.pan.0, 0.5), unit(self.pan.1, 0.0)),
        }
    }

    /// Moves the content by `dx`, `dy` pixels given the `overflow` that [`place`] reported.
    pub fn pan_by(&mut self, dx: f64, dy: f64, overflow: (f64, f64)) {
        let step = |pan: f64, d: f64, over: f64| {
            if over > 0.0 && d.is_finite() {
                (pan - d / over).clamp(0.0, 1.0)
            } else {
                pan
            }
        };
        self.pan = (
            step(self.pan.0, dx, overflow.0),
            step(self.pan.1, dy, overflow.1),
        );
    }
}

/// One page part on screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Slot {
    pub page: usize,
    /// The part of the page shown, as fractions of its width and height (0..=1).
    pub src: Rect,
    /// Where it goes, in viewport pixels (may extend past the viewport when zoomed).
    pub dst: Rect,
    /// Pixels per point the whole page is drawn at.
    pub scale: f64,
}

/// A placed view.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub slots: Vec<Slot>,
    /// How far the content is wider and taller than the viewport, in pixels (0 if it fits).
    pub overflow: (f64, f64),
}

/// Places `view` in `viewport`. `sizes` holds every page's size; a page missing from it, or with
/// a size that isn't a positive number, is drawn as US Letter. `split` is where half-page views
/// cut (fraction of the page height). Nothing is placed in an empty viewport.
pub fn place(view: &View, sizes: &[PageSize], viewport: Viewport, split: f64, zoom: Zoom) -> Frame {
    let zoom = zoom.sanitised();
    let ok = |v: f64| v.is_finite() && v > 0.0;
    if !ok(viewport.width) || !ok(viewport.height) {
        return Frame {
            slots: Vec::new(),
            overflow: (0.0, 0.0),
        };
    }
    let split = if split.is_finite() {
        split.clamp(0.0, 1.0)
    } else {
        0.5
    };
    let size = |p: usize| {
        sizes
            .get(p)
            .copied()
            .filter(|s| ok(s.width) && ok(s.height))
            .unwrap_or(PageSize {
                width: 612.0,
                height: 792.0,
            })
    };
    let (bottom, overflow) = row(&view.bottom, &size, viewport, zoom);
    if !view.is_half() {
        let slots = bottom
            .into_iter()
            .map(|(page, dst, scale)| Slot {
                page,
                src: xywh(0.0, 0.0, 1.0, 1.0),
                dst,
                scale,
            })
            .collect();
        return Frame { slots, overflow };
    }
    let (top, _) = row(&view.top, &size, viewport, zoom);
    let part = |(page, dst, scale): (usize, Rect, f64), y0: f64, y1: f64| Slot {
        page,
        src: xywh(0.0, y0, 1.0, y1 - y0),
        dst: xywh(
            dst.x0,
            dst.y0 + dst.height() * y0,
            dst.width(),
            dst.height() * (y1 - y0),
        ),
        scale,
    };
    let mut slots: Vec<Slot> = top.into_iter().map(|s| part(s, 0.0, split)).collect();
    slots.extend(bottom.into_iter().map(|s| part(s, split, 1.0)));
    Frame { slots, overflow }
}

/// A page's rectangle and scale.
type Placed = (usize, Rect, f64);

/// Lays `pages` side by side at one height, fitted and zoomed: each page's rectangle and scale,
/// plus the overflow.
fn row(
    pages: &[usize],
    size: &dyn Fn(usize) -> PageSize,
    viewport: Viewport,
    zoom: Zoom,
) -> (Vec<Placed>, (f64, f64)) {
    // Sum of width / height: the row's width at height 1.
    let aspect: f64 = pages
        .iter()
        .map(|&p| {
            let s = size(p);
            s.width / s.height
        })
        .sum();
    if !(aspect.is_finite() && aspect > 0.0) {
        return (Vec::new(), (0.0, 0.0));
    }
    let fit_height = match zoom.fit {
        Fit::Page => viewport.height.min(viewport.width / aspect),
        Fit::Width => viewport.width / aspect,
    };
    let height = fit_height * zoom.factor;
    let width = height * aspect;
    let over_x = (width - viewport.width).max(0.0);
    let over_y = (height - viewport.height).max(0.0);
    let x0 = if over_x > 0.0 {
        -over_x * zoom.pan.0
    } else {
        (viewport.width - width) / 2.0
    };
    let y0 = if over_y > 0.0 {
        -over_y * zoom.pan.1
    } else {
        (viewport.height - height) / 2.0
    };
    let mut x = x0;
    let placed = pages
        .iter()
        .map(|&p| {
            let s = size(p);
            let w = height * s.width / s.height;
            let rect = xywh(x, y0, w, height);
            x += w;
            (p, rect, height / s.height)
        })
        .collect();
    (placed, (over_x, over_y))
}

#[cfg(test)]
mod tests {
    use super::*;

    const LETTER: PageSize = PageSize {
        width: 612.0,
        height: 792.0,
    };
    const SCREEN: Viewport = Viewport {
        width: 1200.0,
        height: 792.0,
    };

    fn full(pages: &[usize]) -> View {
        View {
            top: pages.to_vec(),
            bottom: pages.to_vec(),
        }
    }

    #[test]
    fn single_page_is_centred_at_fit() {
        let f = place(&full(&[0]), &[LETTER], SCREEN, 0.5, Zoom::default());
        let s = f.slots[0];
        assert_eq!(
            (s.dst.x0, s.dst.y0, s.dst.width(), s.dst.height()),
            (294.0, 0.0, 612.0, 792.0)
        );
        assert_eq!((s.scale, f.overflow), (1.0, (0.0, 0.0)));
    }

    #[test]
    fn two_pages_share_a_height_and_sit_side_by_side() {
        let a4ish = PageSize {
            width: 300.0,
            height: 600.0,
        };
        let f = place(
            &full(&[0, 1]),
            &[LETTER, a4ish],
            SCREEN,
            0.5,
            Zoom::default(),
        );
        let (a, b) = (f.slots[0].dst, f.slots[1].dst);
        assert_eq!(a.height(), b.height());
        assert!((a.x1 - b.x0).abs() < 1e-3);
        assert!(a.x0 + a.width() + b.width() <= SCREEN.width + 1e-3);
    }

    #[test]
    fn half_view_tiles_top_of_next_over_bottom_of_current() {
        let view = View {
            top: vec![1],
            bottom: vec![0],
        };
        let f = place(&view, &[LETTER, LETTER], SCREEN, 0.4, Zoom::default());
        let (t, b) = (f.slots[0], f.slots[1]);
        assert_eq!((t.page, b.page), (1, 0));
        assert_eq!((t.src.y0, t.src.height()), (0.0, 0.4));
        assert!((b.src.y0 - 0.4).abs() < 1e-6 && (b.src.height() - 0.6).abs() < 1e-6);
        assert!((t.dst.y0 + t.dst.height() - b.dst.y0).abs() < 1e-3);
        assert!((t.dst.height() + b.dst.height() - 792.0).abs() < 1e-3);
    }

    #[test]
    fn zoom_overflows_and_pans() {
        let mut z = Zoom {
            factor: 2.0,
            ..Zoom::default()
        };
        let f = place(&full(&[0]), &[LETTER], SCREEN, 0.5, z);
        assert_eq!(f.overflow, (24.0, 792.0));
        assert_eq!(f.slots[0].dst.y0, 0.0);
        z.pan_by(0.0, -396.0, f.overflow);
        assert_eq!(z.pan.1, 0.5);
        let f = place(&full(&[0]), &[LETTER], SCREEN, 0.5, z);
        assert_eq!(f.slots[0].dst.y0, -396.0);
        z.pan_by(0.0, -1e9, f.overflow);
        assert_eq!(z.pan.1, 1.0);
    }

    #[test]
    fn fit_width_fills_the_width() {
        let z = Zoom {
            fit: Fit::Width,
            ..Zoom::default()
        };
        let f = place(&full(&[0]), &[LETTER], SCREEN, 0.5, z);
        assert_eq!(f.slots[0].dst.width(), 1200.0);
        assert!(f.overflow.1 > 0.0);
    }

    #[test]
    fn hostile_numbers_never_panic() {
        let bad = PageSize {
            width: f64::NAN,
            height: -1.0,
        };
        for vp in [
            Viewport {
                width: 0.0,
                height: 10.0,
            },
            Viewport {
                width: f64::INFINITY,
                height: 10.0,
            },
            SCREEN,
        ] {
            for z in [
                Zoom {
                    factor: f64::NAN,
                    pan: (f64::INFINITY, -3.0),
                    fit: Fit::Width,
                },
                Zoom::default(),
            ] {
                let f = place(&full(&[0, 7]), &[bad], vp, f64::NAN, z);
                for s in f.slots {
                    assert!(s.dst.width().is_finite() && s.scale.is_finite());
                }
            }
        }
    }
}
