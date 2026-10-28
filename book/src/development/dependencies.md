# Dependencies

- Pure Rust by default; C/C++ only behind Cargo features in `scorecraft-devices`.
- Licences must be compatible with MIT OR Apache-2.0 (checked by `cargo deny`); no GPL, AGPL or
  LGPL crates.
- `Cargo.lock` is committed; git dependencies are pinned by revision.
- Dependabot proposes updates; CI checks them.
