//! Display filters (architecture §7). Dark and sepia remap text and line-art paints through the
//! PDF backend, so engraved music changes colour while photos and scans keep theirs.

use scorecraft_pdf::PaperColors;

/// How a page is coloured on screen. Display only; the score's file never changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Filter {
    /// The file's own colours.
    #[default]
    Normal,
    /// Light ink on a dark page, for dim stages.
    Dark,
    /// Dark brown ink on warm paper.
    Sepia,
}

impl Filter {
    pub const ALL: [Filter; 3] = [Filter::Normal, Filter::Dark, Filter::Sepia];

    /// The ink and paper colours the backend draws with, `None` for the file's own.
    pub fn colors(self) -> Option<PaperColors> {
        match self {
            Filter::Normal => None,
            Filter::Dark => Some(PaperColors {
                ink: [0xe6, 0xe6, 0xe6],
                paper: [0x1c, 0x1c, 0x1c],
            }),
            Filter::Sepia => Some(PaperColors {
                ink: [0x3b, 0x2a, 0x1a],
                paper: [0xf4, 0xec, 0xd8],
            }),
        }
    }

    /// The stable id used by settings, commands and the control channel.
    pub fn id(self) -> &'static str {
        match self {
            Filter::Normal => "normal",
            Filter::Dark => "dark",
            Filter::Sepia => "sepia",
        }
    }

    /// The filter with this [`Filter::id`].
    pub fn from_id(id: &str) -> Option<Filter> {
        Filter::ALL.into_iter().find(|f| f.id() == id)
    }
}

#[cfg(test)]
mod tests {
    use super::Filter;

    #[test]
    fn ids_round_trip() {
        for f in Filter::ALL {
            assert_eq!(Filter::from_id(f.id()), Some(f));
        }
        assert_eq!(Filter::from_id("neon"), None);
        assert_eq!(Filter::default().colors(), None);
    }
}
