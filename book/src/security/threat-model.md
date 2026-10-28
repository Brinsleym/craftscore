# Threat model

## Assets

- confidentiality of the user's scores, markings, practice log and unrelated local files;
- integrity of the library (no lost or corrupted markings, ever) and of the original PDFs;
- availability of the reader during a performance;
- confidentiality of sync credentials and, with end-to-end encryption, of synced content;
- integrity and provenance of release artefacts and dependencies.

## Threat actors

- the author of a malicious PDF, image, audio file, `.4sc` or setlist bundle (shared by a
  bandmate, downloaded, or returned by IMSLP);
- a malicious or compromised network service (catalogue, sync server, WebDAV host, scanner);
- a malicious local process connecting to the control server;
- a malicious or over-privileged MCP client or agent;
- a hostile MIDI or HID device;
- a compromised dependency or build environment.

## Primary scenarios

- **Hostile document parsing.** Oversized dimensions, deep or cyclic object graphs, decompression
  bombs, corrupt offsets. Reject before unreasonable work, show an error or a placeholder page,
  keep a minimized regression test.
- **Archive extraction.** Zip slip and symlinks in `.scoreset` or `.4sc` archives. Entries are
  resolved inside a fresh directory; absolute paths, `..` and links are refused.
- **Sync poisoning.** A backend returns crafted ops. Ops are schema-validated, size-capped and
  applied as normal transactions; blobs are verified by hash; nothing from a backend is executed.
- **Data loss.** A crash or full disk during save. Atomic writes, a journal, and verify/repair.
- **Performance denial.** Anything that could freeze the reader on stage (a huge page, a flood of
  MIDI) is bounded and runs off the UI thread.
