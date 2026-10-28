# Fonts

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** major (first version) · **Target:** enScore 3.7 (2026)

- Only fonts under the SIL Open Font License 1.1, Apache-2.0, MIT or the Ubuntu Font Licence.
- **Never commit font files to this repository.** Fonts are an optional build input,
  `SCORECRAFT_FONTS_DIR=<checkout of a fonts repository>`, read by `crates/text/build.rs` (the same
  pattern as PhotoCraft's `CRAFT_FONTS_DIR` and storytold/craft-fonts, which may be used directly
  where its licences fit). Without it, everything must still work with system fonts and a
  built-in fallback glyph set.
- **Music symbols** (stamps, manuscript paper clefs) come from a SMuFL font under the OFL, such as
  Bravura or Petaluma (Steinberg). The fonts repository keeps the licence text next to each family
  and an attribution row; ScoreCraft's `NOTICE` and `ATTRIBUTION.md` list what release builds embed.
- No font is ever extracted from enScore, forScore or an OS install.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-28 | major | First version |
