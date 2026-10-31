//! The adapter over `pdfcraft-render`: everything PDFCraft-specific stays in this file.

use std::sync::Arc;

use pdfcraft_render as backend;

use crate::PdfError;

/// Hard cap on a rendered side in pixels; larger requests are scaled down to fit.
pub const MAX_SIDE: f32 = backend::MAX_SIDE;
/// Hard cap on rendered pixels per page; larger requests are scaled down to fit.
pub const MAX_PIXELS: f32 = backend::MAX_PIXELS;

/// One page's displayed geometry.
#[derive(Debug, Clone, PartialEq)]
pub struct PageGeometry {
    /// Displayed width in points, after `/Rotate` and `/UserUnit`.
    pub width: f32,
    /// Displayed height in points, after `/Rotate` and `/UserUnit`.
    pub height: f32,
    /// Clockwise rotation in degrees: 0, 90, 180 or 270.
    pub rotation: u16,
    /// The page label (`/PageLabels`), or the 1-based page number.
    pub label: String,
    /// The crop box in unrotated PDF user space: `[x0, y0, x1, y1]`. Annotations are stored in
    /// this space (architecture §1.1).
    pub crop: [f32; 4],
}

/// Document information that a library can show and search.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Metadata {
    pub title: Option<String>,
    /// `/Author`, where score PDFs usually keep the composer.
    pub author: Option<String>,
    pub subject: Option<String>,
    pub keywords: Option<String>,
    pub creator: Option<String>,
    pub producer: Option<String>,
    /// The header's PDF version, such as `1.7`.
    pub pdf_version: String,
    pub encrypted: bool,
    pub file_size: usize,
}

/// A bookmark from the document outline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutlineEntry {
    pub title: String,
    /// The 0-based page it goes to, if it resolves to a page of this document.
    pub page: Option<usize>,
    pub children: Vec<OutlineEntry>,
}

/// A rendered page or tile: premultiplied RGBA8, row-major, one `u32` word per pixel holding the
/// bytes R, G, B, A in memory order (so a GPU texture or egui image can take it without a copy).
#[derive(Clone, PartialEq, Eq)]
pub struct Bitmap {
    pub width: u32,
    pub height: u32,
    pub words: Vec<u32>,
}

impl Bitmap {
    /// The pixels as bytes, R, G, B, A per pixel.
    pub fn bytes(&self) -> &[u8] {
        bytemuck::cast_slice(&self.words)
    }

    /// The RGBA bytes of pixel (`x`, `y`), or `None` outside the bitmap.
    pub fn pixel(&self, x: u32, y: u32) -> Option<[u8; 4]> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let index = (y as usize)
            .checked_mul(self.width as usize)?
            .checked_add(x as usize)?;
        self.words.get(index).map(|w| w.to_ne_bytes())
    }

    /// Bytes held, for cache budgets.
    pub fn byte_len(&self) -> usize {
        self.words.len().saturating_mul(4)
    }
}

impl std::fmt::Debug for Bitmap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Bitmap({}×{})", self.width, self.height)
    }
}

/// A region of a page scaled to device pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Tile {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

/// One glyph of a page's text layer.
#[derive(Debug, Clone, PartialEq)]
pub struct TextGlyph {
    /// One character, or several for a ligature.
    pub text: String,
    /// Bounding box in page view space (points, y down, after rotation): `[x0, y0, x1, y1]`.
    pub rect: [f32; 4],
}

/// A page's text layer, for search and title detection.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PageText {
    pub glyphs: Vec<TextGlyph>,
    /// The page's text in reading order, with spaces and line breaks reconstructed.
    pub plain: String,
}

/// Device pixels that cover `points` at `scale` pixels per point (rounded up; at least 1).
pub fn device_pixels(points: f32, scale: f32) -> u32 {
    backend::device_pixels(points, scale)
}

/// The scale that fits a page into a `width` × `height` pixel viewport, keeping its aspect.
/// `None` for an empty viewport or a degenerate page.
pub fn fit_scale(page: &PageGeometry, width: f32, height: f32) -> Option<f32> {
    let s = (width / page.width).min(height / page.height);
    (s.is_finite() && s > 0.0).then_some(s)
}

/// Display colours for a page: dark text and line art are drawn in `ink`, the page clears to
/// `paper`. Images keep their colours. Display only; the file never changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PaperColors {
    /// sRGB colour that dark paints are mapped towards.
    pub ink: [u8; 3],
    /// sRGB colour that light paints and the page background are mapped towards.
    pub paper: [u8; 3],
}

/// An open PDF score. `Send`, so a background render thread can own one; [`Document::fork`]
/// gives each thread its own.
pub struct Document {
    renderer: backend::PageRenderer,
    bytes: Arc<Vec<u8>>,
    password: Option<Arc<str>>,
    colors: Option<PaperColors>,
    pages: Vec<PageGeometry>,
    metadata: Metadata,
    outline: Vec<OutlineEntry>,
}

impl std::fmt::Debug for Document {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Document")
            .field("pages", &self.pages.len())
            .finish_non_exhaustive()
    }
}

impl Document {
    /// Opens a PDF from its bytes. The bytes are shared, never copied or modified.
    pub fn open(bytes: Arc<Vec<u8>>) -> Result<Self, PdfError> {
        Self::open_with_password(bytes, None)
    }

    /// Opens an encrypted PDF with its user or owner password.
    pub fn open_with_password(
        bytes: Arc<Vec<u8>>,
        password: Option<&str>,
    ) -> Result<Self, PdfError> {
        let info = backend::inspect(bytes.clone(), password).map_err(|e| match e {
            backend::OpenError::Invalid(why) => PdfError::Invalid(why),
            backend::OpenError::NeedsPassword => PdfError::NeedsPassword,
            backend::OpenError::WrongPassword => PdfError::WrongPassword,
            backend::OpenError::Unsupported(what) => PdfError::Unsupported(what),
        })?;
        let pages = info
            .pages
            .iter()
            .map(|p| PageGeometry {
                width: p.width,
                height: p.height,
                rotation: p.rotation,
                label: p.label.clone(),
                crop: p.crop,
            })
            .collect();
        let metadata = Metadata {
            title: non_empty(info.title),
            author: non_empty(info.author),
            subject: non_empty(info.subject),
            keywords: non_empty(info.keywords),
            creator: non_empty(info.creator),
            producer: non_empty(info.producer),
            pdf_version: info.pdf_version,
            encrypted: info.encrypted,
            file_size: info.file_size,
        };
        let outline = info.outline.iter().map(outline_entry).collect();
        let password: Option<Arc<str>> = password.map(Arc::from);
        let renderer = backend::PageRenderer::new(bytes.clone(), config(password.clone(), None));
        Ok(Document {
            renderer,
            bytes,
            password,
            colors: None,
            pages,
            metadata,
            outline,
        })
    }

    /// Another handle on the same file for another thread, drawing with `colors` (`None` for the
    /// file's own colours). Shares the bytes and page list; the new renderer parses the file
    /// again, so fork once per thread or colour change, not per render.
    pub fn fork(&self, colors: Option<PaperColors>) -> Document {
        let renderer =
            backend::PageRenderer::new(self.bytes.clone(), config(self.password.clone(), colors));
        Document {
            renderer,
            bytes: self.bytes.clone(),
            password: self.password.clone(),
            colors,
            pages: self.pages.clone(),
            metadata: self.metadata.clone(),
            outline: self.outline.clone(),
        }
    }

    /// The display colours this handle draws with.
    pub fn colors(&self) -> Option<PaperColors> {
        self.colors
    }

    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Geometry of page `index` (0-based).
    pub fn page(&self, index: usize) -> Result<&PageGeometry, PdfError> {
        self.pages.get(index).ok_or(PdfError::PageOutOfRange {
            page: index,
            count: self.pages.len(),
        })
    }

    pub fn pages(&self) -> &[PageGeometry] {
        &self.pages
    }

    pub fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    pub fn outline(&self) -> &[OutlineEntry] {
        &self.outline
    }

    /// The size in pixels a whole-page render at `scale` will have, after the size caps.
    pub fn render_size(&self, page: usize, scale: f32) -> Result<(u32, u32), PdfError> {
        let g = self.page(page)?;
        let s = checked_scale(g, scale)?;
        Ok((device_pixels(g.width, s), device_pixels(g.height, s)))
    }

    /// Renders page `page` at `scale` pixels per point (`dpi / 72`) on a white background.
    /// Requests over [`MAX_SIDE`] or [`MAX_PIXELS`] are scaled down to fit.
    pub fn render(&mut self, page: usize, scale: f32) -> Result<Bitmap, PdfError> {
        let g = self.page(page)?;
        let s = checked_scale(g, scale)?;
        self.rasterise(page, s, None)
    }

    /// Renders one tile of page `page` at `scale`, in that render's device pixels.
    pub fn render_tile(&mut self, page: usize, scale: f32, tile: Tile) -> Result<Bitmap, PdfError> {
        let g = self.page(page)?;
        let s = checked_scale(g, scale)?;
        let (w, h) = (device_pixels(g.width, s), device_pixels(g.height, s));
        let inside = tile.w > 0
            && tile.h > 0
            && tile.x.checked_add(tile.w).is_some_and(|r| r <= w)
            && tile.y.checked_add(tile.h).is_some_and(|b| b <= h);
        if !inside {
            return Err(PdfError::BadTile);
        }
        let t = backend::Tile {
            x: tile.x,
            y: tile.y,
            w: tile.w,
            h: tile.h,
        };
        self.rasterise(page, s, Some(t))
    }

    /// The text layer of page `page`.
    pub fn text(&mut self, page: usize) -> Result<PageText, PdfError> {
        self.page(page)?;
        let req = backend::RenderRequest {
            page,
            kind: backend::RequestKind::Text,
            scale: 1.0,
            ..backend::RenderRequest::default()
        };
        let out = self.renderer.render(req);
        if let Some(why) = out.error {
            return Err(PdfError::Render(why));
        }
        let Some(text) = out.text else {
            return Ok(PageText::default());
        };
        Ok(PageText {
            glyphs: text
                .glyphs
                .iter()
                .map(|g| TextGlyph {
                    text: g.text.clone(),
                    rect: g.rect,
                })
                .collect(),
            plain: text.plain_text(),
        })
    }

    fn rasterise(
        &mut self,
        page: usize,
        scale: f32,
        tile: Option<backend::Tile>,
    ) -> Result<Bitmap, PdfError> {
        let req = backend::RenderRequest {
            page,
            kind: backend::RequestKind::Pixels,
            tile,
            scale,
            ..backend::RenderRequest::default()
        };
        let out = self.renderer.render(req);
        if let Some(why) = out.error {
            return Err(PdfError::Render(why));
        }
        let words = out.rgba.into_words();
        let expected = (out.width as usize).checked_mul(out.height as usize);
        if out.width == 0 || out.height == 0 || expected != Some(words.len()) {
            return Err(PdfError::Render(format!(
                "the renderer returned {} pixels for {}×{}",
                words.len(),
                out.width,
                out.height
            )));
        }
        Ok(Bitmap {
            width: out.width,
            height: out.height,
            words,
        })
    }
}

/// Validates `scale` and applies the renderer's size caps, so sizes reported by
/// [`Document::render_size`] match what [`Document::render`] returns.
fn checked_scale(page: &PageGeometry, scale: f32) -> Result<f32, PdfError> {
    if !scale.is_finite() || scale <= 0.0 {
        return Err(PdfError::BadScale);
    }
    Ok(backend::effective_scale(page.width, page.height, scale))
}

fn config(password: Option<Arc<str>>, colors: Option<PaperColors>) -> backend::RenderConfig {
    let reading_colors = backend::ReadingColors {
        params: colors.map(|c| backend::ReadingParams {
            text: c.ink,
            background: c.paper,
            only_black: false,
            line_art: true,
        }),
    };
    backend::RenderConfig {
        password,
        reading_colors,
        ..backend::RenderConfig::default()
    }
}

fn non_empty(s: Option<String>) -> Option<String> {
    s.map(|t| t.trim().to_owned()).filter(|t| !t.is_empty())
}

fn outline_entry(item: &backend::OutlineItem) -> OutlineEntry {
    OutlineEntry {
        title: item.title.clone(),
        page: item.page,
        children: item.children.iter().map(outline_entry).collect(),
    }
}
