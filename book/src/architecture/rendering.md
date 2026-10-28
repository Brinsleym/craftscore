# Rendering and page turns

- `pdf` parses lazily and rasterises pages; `render` (CPU, the oracle) and `gpu` (wgpu) composite
  page tiles, display filters, snips and ink.
- Tiles are cached per page, zoom bucket and filter. The current, next and previous pages, and the
  next setlist piece's first page, are pre-rendered in background jobs, so a turn is a cache hit.
- `layout` produces placed page regions for single, two-page, half-page and alternating turns;
  animations interpolate two layouts and are skipped in e-ink mode.
- Budgets (turn ≤ 16 ms from cache, first vector page ≤ 120 ms) are in
  [`docs/scorecard.md`](https://github.com/Brinsleym/enscore-cleanroom/blob/main/docs/scorecard.md#performance).
