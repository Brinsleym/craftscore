# Sync and backup parity

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** major (first version, all rows missing) · **Target:** enScore 3.7 (2026)

enScore sells Sync as a subscription and requires an account. ScoreCraft's sync is free, works
without an account through storage the user already has, and offers a self-hostable server.
Design: [`architecture.md` §9](architecture.md#9-sync).

**Summary (estimated):** 0% ready.

| Id | Target | Rows | Status |
|---|---|---|---|
| SY-1 | Scores (blobs), markings, metadata, lists and bookmarks sync across devices | SYNC-1 | missing |
| SY-2 | Offline-first: every edit is local first; sync is background and resumable | SYNC-3 | missing |
| SY-3 | Concurrent offline markings on two devices merge without loss | SYNC-6 | missing |
| SY-4 | Folder backend (iCloud Drive, OneDrive, Dropbox, Syncthing) | SYNC-7 | missing |
| SY-5 | WebDAV backend (Nextcloud and others) | SYNC-7 | missing |
| SY-6 | Self-hostable `scorecraft-sync-server` with accounts the operator controls | SYNC-1 | missing |
| SY-7 | Optional end-to-end encryption | ours | missing |
| SY-8 | Fast first download on a new device: metadata first, blobs on demand | SYNC-5 | missing |
| SY-9 | Backup to a single archive and restore, local or remote | SYNC-2 | missing |
| SY-10 | Removing an account or backend removes remote copies, keeps local ones | SYNC-4 | missing |
| SY-11 | Sync never interrupts performance mode | ours | missing |

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-28 | major | First version |
