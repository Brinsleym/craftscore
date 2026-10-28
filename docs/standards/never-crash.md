# Never crash

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** major (first version) · **Target:** enScore 3.7 (2026)

A score reader runs on a music stand in front of an audience. A crash loses the performer's place,
and a crash during save can lose their markings. Not crashing outranks every feature.

## Rules

1. **No panics in non-test code.** No `unwrap`, `expect`, `panic!`, `unreachable!`, `todo!`,
   `unimplemented!`, and no indexing that can go out of bounds. Return a typed error and
   propagate it with `?`. A fallback (`unwrap_or`, `unwrap_or_default`) is fine only where it is
   truly correct and cannot silently corrupt a library.
2. **The one exception** is a provably infallible literal (a constant regex, a fixed-size array
   conversion): `#[allow(clippy::expect_used)]` plus `.expect("why it can't fail")`.
3. **No `unsafe`.** `unsafe_code = "forbid"` workspace-wide. `scorecraft-devices` is the single
   exception for platform FFI, feature-gated, `unsafe_code = "deny"` with per-block `#[allow]`,
   a `SAFETY:` comment on each block, and tests against real platform events.
4. **Hostile input.** Everything from a file, the network, a MIDI port, a pedal, the control
   channel, MCP or preferences is hostile:
   - `get()` instead of `[i]`; slice strings at char boundaries only;
   - `checked_*` / `saturating_*` for lengths, offsets, counts and page numbers;
   - guard division by zero and NaN/inf before casts;
   - cap allocations sized by input (object counts, image dimensions, decompressed sizes,
     stroke point counts) and stop decompression past the cap;
   - bound recursion with depth limits or seen-sets (PDF object graphs are cyclic);
   - time out network calls and background jobs, and make them cancellable.
5. **Don't cascade.** `lock().unwrap_or_else(PoisonError::into_inner)`; thread joins are `Result`s;
   one failed page render shows a placeholder for that page, not a dead reader.
6. **Durable saves.** Write to a temp file, fsync, rename. A failed save keeps the previous
   version and tells the user. A full disk is an error message, not a crash or a truncated file.
7. **Last-resort guard.** The app shell wraps command dispatch, import, export and rendering jobs
   in `catch_unwind`, reports an escaped panic as an error, and keeps the reader open on the
   current page. It is a safety net, not a licence to panic. Keep `panic = "unwind"` natively.
8. **Prove it.** Every crash fix ships with a minimal synthetic regression test that panicked
   before the fix.

## Enforcement

- `clippy.toml`: `allow-unwrap-in-tests`, `allow-expect-in-tests`, `allow-panic-in-tests`,
  `allow-indexing-slicing-in-tests`.
- Each production crate root: `#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]`.
- Each crate's `Cargo.toml`: `[lints] workspace = true`.
- `panic_hunt` (engine integration test) dispatches every command with adversarial params and
  document states.
- `cargo fuzz` targets for `pdf`, `fsc`, `midi`, the library loader and the setlist bundle reader.
- The scorecard's Reliability area counts crates carrying the lints.

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-28 | major | First version, from the Crafting Apps' never-crash rules |
