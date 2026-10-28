# Interchange format descriptions

Descriptions of file formats that have **no public specification** (for example forScore's
`.4sc`), written by an analyst from black-box comparison of sample files and read by a different
implementer. The full rules are in [clean-room §5](../../clean-room.md#5-interchange-formats-without-a-public-spec).

Each description states:

- **Analyst** and **date**, and that the analyst did not read the other app's code or bundle.
- **Samples:** where they come from (files the analyst made, or published files with their
  licence), their sha256 and the corpus pin. Samples are never committed here.
- **Container:** the standard container (zip, plist, SQLite, JSON, PDF) and its spec.
- **Structure:** each field or entry, what it was observed to mean, and the experiment that showed
  it ("added one red stroke on page 2 → a new entry in …").
- **Unknowns:** everything seen but not understood. Importers report these and skip them.

| Format | Description | Analyst | Status |
|---|---|---|---|
| forScore `.4sc` (import) | _not started_ | | IMP-5 |
| ScoreCraft setlist bundle | ours, see [`../../file-format-parity.md`](../../file-format-parity.md) | | LIST-5 |
