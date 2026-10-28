## What

<!-- What this changes, for a reader who hasn't seen the issue. Feature-map ids: READ-5, … -->

## Sources (clean-room)

<!-- Public sources, observation notes (docs/observations/…) and specs (docs/spec/…) used.
     Write "none needed" for pure infrastructure. -->

## Checklist

- [ ] No proprietary code, assets, text or screenshots ([clean-room](../docs/clean-room.md#9-review-checklist-every-pr))
- [ ] New third-party code or assets are permissively licensed, with a licence file and an `ATTRIBUTION.md` row
- [ ] Tests added; `cargo test`, `cargo clippy -- -D warnings`, `cargo xtask layers` pass
- [ ] No panics or `unsafe` in non-test code ([never crash](../docs/standards/never-crash.md))
- [ ] `cargo xtask parity` / `cargo xtask scorecard` regenerated if commands or numbers changed
- [ ] Feature-map rows, parity docs and gaps updated with dates and revision history
- [ ] UI changes: before/after screenshots on public-domain scores

## Still open

<!-- What this does not do yet. -->
