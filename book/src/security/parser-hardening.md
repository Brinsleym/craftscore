# Parser hardening

Every parser (`pdf`, `codecs`, `audio`, `fsc`, `midi`, the library loader, setlist bundles):

- uses checked arithmetic for offsets, lengths and counts, and `get()` for indexing;
- caps allocations (object counts, image pixels, decompressed bytes, stroke points) with limits
  documented in the crate README;
- bounds recursion with depth limits and seen-sets;
- times out and supports cancellation for long work;
- has malformed-input unit tests and a `cargo fuzz` target;
- never panics ([never crash](https://github.com/Brinsleym/enscore-cleanroom/blob/main/docs/standards/never-crash.md)).
