//! Design tokens (`docs/ui-design.md`): every colour and size the shell draws with.

use craft_ui::buttons::{ButtonVisuals, IconButtonStyle};
use craft_ui::theme::WidgetPalette;
use egui::{Color32, CornerRadius, Stroke};

use crate::state::Theme;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tokens {
    /// Behind the pages.
    pub reader_bg: Color32,
    /// Drawn where a page's bitmap isn't ready yet.
    pub placeholder: Color32,
    pub toolbar_bg: Color32,
    pub text: Color32,
    pub icon: Color32,
    pub accent: Color32,
    pub hover: Color32,
    pub error: Color32,
    /// Toolbar height and touch-target side, in points (≥ 44 for fingers on a tablet).
    pub toolbar_height: f32,
    pub button: f32,
    pub radius: u8,
}

impl Tokens {
    pub fn for_theme(theme: Theme) -> Tokens {
        match theme {
            Theme::Light => Tokens {
                reader_bg: Color32::from_rgb(0x5f, 0x63, 0x68),
                placeholder: Color32::from_rgb(0xf4, 0xf4, 0xf2),
                toolbar_bg: Color32::from_rgb(0xf7, 0xf7, 0xf8),
                text: Color32::from_rgb(0x1f, 0x23, 0x28),
                icon: Color32::from_rgb(0x2b, 0x30, 0x36),
                accent: Color32::from_rgb(0x2a, 0x6f, 0xdb),
                hover: Color32::from_rgb(0xe3, 0xe6, 0xea),
                error: Color32::from_rgb(0xc0, 0x2b, 0x2b),
                toolbar_height: 52.0,
                button: 44.0,
                radius: 8,
            },
            Theme::Dark => Tokens {
                reader_bg: Color32::from_rgb(0x12, 0x13, 0x15),
                placeholder: Color32::from_rgb(0x2a, 0x2c, 0x30),
                toolbar_bg: Color32::from_rgb(0x1e, 0x20, 0x23),
                text: Color32::from_rgb(0xe8, 0xea, 0xed),
                icon: Color32::from_rgb(0xdc, 0xdf, 0xe3),
                accent: Color32::from_rgb(0x6c, 0xa0, 0xff),
                hover: Color32::from_rgb(0x34, 0x37, 0x3c),
                error: Color32::from_rgb(0xff, 0x7b, 0x72),
                toolbar_height: 52.0,
                button: 44.0,
                radius: 8,
            },
        }
    }

    pub fn icon_button(&self) -> IconButtonStyle {
        let v = |fill, icon| ButtonVisuals {
            fill,
            stroke: Stroke::NONE,
            icon,
        };
        IconButtonStyle {
            normal: v(Color32::TRANSPARENT, self.icon),
            hovered: v(self.hover, self.icon),
            pressed: Some(v(self.hover, self.accent)),
            selected: v(self.hover, self.accent),
            corner_radius: CornerRadius::same(self.radius),
            focus_stroke: Stroke::new(2.0, self.accent),
        }
    }

    /// Applies the tokens to egui's visuals.
    pub fn apply(&self, theme: Theme, ctx: &egui::Context) {
        let mut visuals = match theme {
            Theme::Light => egui::Visuals::light(),
            Theme::Dark => egui::Visuals::dark(),
        };
        visuals.panel_fill = self.toolbar_bg;
        visuals.override_text_color = Some(self.text);
        WidgetPalette {
            noninteractive: self.toolbar_bg,
            inactive: self.toolbar_bg,
            hovered: self.hover,
            active: self.hover,
            open: self.hover,
        }
        .apply_to(
            &mut visuals.widgets,
            CornerRadius::same(self.radius),
            Stroke::new(1.0, self.text),
        );
        ctx.set_visuals(visuals);
    }
}
