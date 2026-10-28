# Trust boundaries

```text
untrusted bytes        PDF / images / audio / .4sc / .scoreset / library from another device
untrusted network      IMSLP / catalogues / sync backends / WebDAV / eSCL scanners
untrusted devices      MIDI messages / HID pedal keys / stylus events
untrusted control      CLI args / paths / control-channel JSON / MCP requests / agent decisions
──────────────────── validation: parsers, caps, schemas, workspace roots ────────────────────
trusted core           doc model, ops, engine, format store
```

Everything crossing the line is validated at the crate that receives it; nothing inside trusts a
length, index or path that came from outside.
