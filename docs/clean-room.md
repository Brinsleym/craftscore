# Clean-room methodology

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** major (first version) · **Target:** enScore 3.7 (2026)

ScoreCraft is a clean-room reimplementation of **enScore**, the sheet music reader by Jonathan
Vardouniotis. We rebuild what enScore *does* for a musician, in our own code, from public and
user-facing information only. This page is the rule book. Every contributor, human or agent,
follows it, and every PR is reviewed against it.

The process is the one the storytold Crafting Apps use for their reimplementations (PhotoCraft for
Adobe Photoshop, see [storytold/photocraft](https://github.com/storytold/photocraft)), with one
deliberate difference: **we are stricter about the target app's installed files.** PhotoCraft
inspected Photoshop's installed bundle (plists, shortcut files, strings in the binary) to measure
parity. ScoreCraft does not open, unpack or read any part of an installed enScore package. We
measure enScore from what a user can see and what the developer has published.

## 1. What we copy and what we never copy

| We reimplement | We never copy |
|---|---|
| Behaviour a user can observe: what a feature does, which gestures and inputs drive it, what it produces | enScore's source code, binaries, bytecode, shaders, models or build artefacts, in any language |
| Capabilities and workflows described publicly (store listings, release notes, the developer's site, reviews, videos) | Anything obtained by decompiling, disassembling, debugging, decrypting, hooking or tracing enScore |
| File *interchange* where the user owns the file (for example PDFs with ink, a `.4sc` export the user made) | enScore's or forScore's icons, artwork, stamps, fonts, sounds (including metronome sounds), UI copy or screenshots in the shipped product |
| Public standards (PDF / ISO 32000, MIDI 1.0, Bluetooth HID, eSCL, Unicode, SMuFL) | Network APIs, sync protocols or server endpoints of enScore, by sniffing traffic or reading tokens |
| Open-licensed code and assets compatible with MIT OR Apache-2.0, attributed in [`ATTRIBUTION.md`](../ATTRIBUTION.md) | GPL, AGPL, LGPL or non-commercial code, even "just to look", for anything we ship |

Matching the *look* is limited to conventions a musician expects (where a page-turn zone sits, that
a highlighter is translucent). We do not reproduce enScore's visual identity: its name, logo,
colours, icon set, layout of marketing screens or exact wording. ScoreCraft has its own design
system ([`ui-design.md`](ui-design.md)).

## 2. Allowed sources

In order of preference:

1. **Public specifications.** ISO 32000-2 (PDF 2.0) and ISO 32000-1, the Adobe PDF 1.7 reference,
   MIDI 1.0 and MIDI-CI from the MIDI Association, Bluetooth HID and the USB HID usage tables,
   Mopria/Apple eSCL (Wi-Fi scanning), SMuFL (Standard Music Font Layout), Unicode, ICU locale data,
   the IMSLP / MediaWiki public API documentation.
2. **The developer's public material.** App Store, Google Play, Microsoft Store and Amazon Appstore
   listings; release notes ("What's New"); enscore.app; support pages; published videos.
3. **Third-party public descriptions.** Reviews, partner pages (for example the AirTurn app page),
   forum posts by users, tutorials.
4. **Observation as a user.** A contributor who has a legal copy of enScore may use it *as a user*
   on their own device and write down what they see: which control does what, what the result
   looks like, how a gesture feels. Observation notes go in [`observations/`](observations/README.md).
   Observation never includes looking inside the app (files in its bundle or sandbox, logs,
   network traffic, memory).
5. **Academic papers and open-licensed implementations** for algorithms (stroke smoothing, page
   deskewing, pitch detection, full-text indexing), under permissive licences only.

## 3. Forbidden sources and actions

- Downloading, extracting, unzipping, mounting or reading an enScore `.ipa`, `.app`, `.apk`, `.aab`,
  `.msix`/`.appx` or any other package, or files from its install or data directories.
- Decompiling, disassembling, running `strings`/`class-dump`/`jadx`/`ILSpy` or similar on enScore,
  forScore or any other proprietary score reader.
- Intercepting, proxying or replaying enScore network traffic; calling its servers; using its
  account tokens.
- Reading leaked, pirated or "found" source code or internal documents of any score reader.
- Reading GPL/AGPL/LGPL implementations of a format or algorithm we are writing (read only the
  public spec or a permissive description).
- Pasting proprietary UI text, screenshots or assets into this repository. Screenshots used for
  observation stay on the observer's machine and are described in words. Their index (photo
  number, file name, description) is committed in
  [`observations/screenshots/`](observations/screenshots/README.md); the image files are gitignored.
- Using AI tools to "recall" proprietary code. Agents must not be asked to reproduce enScore's or
  forScore's internals, and an agent that produces suspicious, unexplained detail about a
  proprietary format's internals must cite a public source or the change is rejected.

When in doubt, stop and ask in the PR. A feature we cannot build clean-room is recorded as
**not clean-room feasible** in [`gaps.md`](gaps.md) rather than built by cutting corners.

## 4. The two-step process: observe, specify, implement

PhotoCraft's rule is "behaviour and look only". For ScoreCraft we make the paper trail explicit,
because so much of a score reader is behaviour (gestures, timing, layout) rather than published
formats.

1. **Observe.** Someone records what enScore does in an observation note
   (`docs/observations/<area>-<topic>.md`, template in [`observations/README.md`](observations/README.md)):
   source (a link, or "observed in enScore <version> on <device>"), date, and a plain-language
   description. No code, no assets, no copied text beyond short quotes of public marketing copy.
2. **Specify.** The behaviour becomes a short behavioural spec in `docs/spec/<area>.md`
   ([`spec/README.md`](spec/README.md)): inputs, outputs, edge cases, acceptance tests. The spec
   is written in our own words, links its observation notes and public sources, and gets a
   feature-map id ([`feature-map.md`](feature-map.md)).
3. **Implement.** The implementer works from the spec, public standards and our own judgement.
   The PR links the spec, and the commit or PR text states the sources used.

The same person may observe and implement: this is a documentation discipline, not a two-team
wall. A strict two-team split (the person who observed must not write the code) is **required**
for one area only: reverse-engineering a file format that has no public specification (see §5).

## 5. Interchange formats without a public spec

Some workflows need files from proprietary apps, most importantly forScore's `.4sc` (enScore
imports it, carrying pages, ink, stamps and layers). Rules:

- **Black-box only.** The format is learned only from files the analyst created themselves (or
  that were published under a licence that permits it), by comparing inputs and outputs: make a
  score, add one stroke, export, compare. Never from the app's code or bundle.
- **Two-team split.** The analyst writes a format description in `docs/spec/formats/<name>.md`.
  The implementer reads only that description and the sample files. They are different people (or
  different agent sessions that share nothing but the spec).
- **Containers first.** Most such files are standard containers (zip, plist, SQLite, JSON, PDF).
  Parse the standard container with our own or a permissively licensed parser, then map the
  observed structure.
- **Samples are not committed.** Like PhotoCraft's corpora, sample files live in a separate corpus
  repository, are fetched at pinned commits and are sha256-verified ([`development.md`](development.md#test-corpora)).
  Only files the author can licence permissively are pinned.
- **Fail loudly.** Unknown structure produces an import report listing what was skipped, never
  silent loss or a crash.
- **Read first, write later.** We do not write a proprietary format until someone can check our
  output in the other app.

## 6. Names and trademarks

- The product is **ScoreCraft**. "enScore" and "forScore" are their owners' marks. We use them only
  nominatively, to say what we interoperate with or what we measured ("imports forScore `.4sc`
  files", "target: enScore 3.7"), never in the product name, icon, store listing title or
  marketing in a way that suggests endorsement.
- The README and docs say plainly that ScoreCraft is an independent project, not affiliated with or
  endorsed by enScore, forScore, IMSLP or any hardware maker.

## 7. Online services

enScore uses online services for title and composer detection, IMSLP search and Sync. We
reimplement the *capability*, never the service:

- **Title and composer detection** uses our own on-device heuristics (PDF metadata, text layer,
  OCR of the first page) and, optionally, open public catalogues (IMSLP, MusicBrainz, Wikidata)
  through their documented APIs and terms.
- **IMSLP** is accessed through its public MediaWiki API, honouring its terms of use, rate limits
  and the copyright status shown per file (public domain in the user's country).
- **Sync** is our own protocol with a self-hostable server and file-based backends
  ([`architecture.md`](architecture.md#9-sync)). We never talk to enScore's servers.

## 8. AI models

No model trained on enScore's or forScore's assets or output is used. OCR and layout models must be
permissively licensed with a documented training-data source, listed in `ATTRIBUTION.md`, and run
on-device by default.

## 9. Review checklist (every PR)

Reviewers (and `.coderabbit.yaml` instructions) check:

- [ ] Sources are public or observation notes, and are named in the PR.
- [ ] No proprietary code, assets, text or screenshots are added.
- [ ] New third-party code or assets are permissively licensed, keep their licence file next to
      them and have an `ATTRIBUTION.md` row in the same change.
- [ ] Format work without a public spec followed §5 (two-team split, black-box samples only).
- [ ] The feature-map row and the parity doc for the area are updated.

## 10. If something slips through

Anyone who believes proprietary material was used opens an issue (or contacts the maintainers
privately, see [`SECURITY.md`](../SECURITY.md) for how). Maintainers revert the change first and
investigate second. The affected code is rewritten from the spec by someone who did not see the
tainted version, and the incident is recorded in [`decisions.md`](decisions.md).

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-28 | major | First version, adapted from PhotoCraft's clean-room rules with stricter bundle and format rules |
