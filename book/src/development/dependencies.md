# Dependencies

- Pure Rust by default; C/C++ only behind Cargo features in `scorecraft-devices`.
- Licences must be compatible with MIT OR Apache-2.0 (checked by `cargo deny`); no GPL, AGPL or
  LGPL crates.
- `Cargo.lock` is committed; git dependencies are pinned by revision.
- Dependabot proposes updates; CI checks them.
- PDFCraft's crates are git dependencies at one pinned revision, with its `[patch.crates-io]`
  entries for the patched hayro family repeated in our root `Cargo.toml`; bumping the revision is
  its own PR with a corpus run.
