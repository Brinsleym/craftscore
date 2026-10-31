use std::fmt;

/// Why a PDF could not be opened, rendered or read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfError {
    /// The bytes are not a PDF this backend can read.
    Invalid(String),
    /// The document is encrypted and needs a password.
    NeedsPassword,
    /// The password given does not open the document.
    WrongPassword,
    /// A PDF feature the backend does not support.
    Unsupported(String),
    /// A page index past the end of the document.
    PageOutOfRange { page: usize, count: usize },
    /// A scale that is zero, negative or not a number.
    BadScale,
    /// A tile outside the page or with no area.
    BadTile,
    /// The page could not be rendered or read (the message says why).
    Render(String),
}

impl PdfError {
    /// A stable machine-readable code for the engine, CLI and control channel.
    pub fn code(&self) -> &'static str {
        match self {
            PdfError::Invalid(_) => "invalid_pdf",
            PdfError::NeedsPassword => "needs_password",
            PdfError::WrongPassword => "wrong_password",
            PdfError::Unsupported(_) => "unsupported",
            PdfError::PageOutOfRange { .. } => "page_out_of_range",
            PdfError::BadScale => "bad_scale",
            PdfError::BadTile => "bad_tile",
            PdfError::Render(_) => "render_failed",
        }
    }
}

impl fmt::Display for PdfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PdfError::Invalid(why) => write!(f, "not a readable PDF: {why}"),
            PdfError::NeedsPassword => f.write_str("the score is protected by a password"),
            PdfError::WrongPassword => f.write_str("the password is incorrect"),
            PdfError::Unsupported(what) => write!(f, "unsupported PDF feature: {what}"),
            PdfError::PageOutOfRange { page, count } => {
                write!(
                    f,
                    "page {page} is out of range (the score has {count} pages)"
                )
            }
            PdfError::BadScale => f.write_str("the render scale must be a positive number"),
            PdfError::BadTile => f.write_str("the tile must have an area and lie on the page"),
            PdfError::Render(why) => write!(f, "the page could not be rendered: {why}"),
        }
    }
}

impl std::error::Error for PdfError {}
