# Sync

Offline-first: each device's library is the source of truth and sync exchanges ops and blobs.
Annotations merge as an add-wins set with tombstones, metadata as last-writer-wins registers on
hybrid logical clocks, list order as a sequence CRDT. Backends: a folder the OS already syncs,
WebDAV, or the self-hostable `scorecraft-sync-server`; no account is required. Optional end-to-end
encryption. See [`docs/architecture.md` §9](https://github.com/Brinsleym/enscore-cleanroom/blob/main/docs/architecture.md#9-sync) and
[`docs/sync-parity.md`](https://github.com/Brinsleym/enscore-cleanroom/blob/main/docs/sync-parity.md).
