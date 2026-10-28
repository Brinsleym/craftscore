# Secure file handling

- Imported files are copied into the library's blob store and never opened for writing again.
- Saves write a temporary file in the same directory, fsync, then rename; the previous version
  survives any failure.
- Export never overwrites without confirmation (UI) or an explicit flag (CLI, automation).
- Paths from automation are resolved and checked against workspace roots after symlink resolution.
- Archive entries are extracted only by name inside a fresh directory; links and `..` are refused.
