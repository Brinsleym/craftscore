# Implementation decisions

Dated records of choices that shape the code, and the open questions waiting for one. Decisions
that affect the process or several areas go through an [RFC](../rfcs/0000-decision-process.md);
this file records the outcome. Newest first.

## Open

| Id | Question | Options | Needed by |
|---|---|---|---|
| D1 | PDF rasteriser | (a) an existing permissively licensed pure-Rust PDF renderer, wrapped by `scorecraft-pdf`; (b) our own interpreter on `kurbo` + `vello_cpu`; (c) (a) now, (b) later behind the same API. Evaluate on the score corpus: correctness, speed, wasm size, licence, maintenance | M2 |
| D2 | Record encoding on disk | JSON (readable, diffable) vs CBOR (smaller, faster) for score records; ink op logs are binary either way | M1 |
| D3 | iOS shell | winit + wgpu + egui directly vs a thin Swift host around a Rust static library (no Swift UI logic) | M7 |
| D4 | OCR for title/composer detection | none (PDF metadata and text layer only) vs a permissively licensed on-device model | M9 |
| D5 | Sync server protocol | HTTP + JSON ops vs a compact binary protocol over WebSocket | M8 |
| D6 | Product name | "ScoreCraft" (working name, following the `{Function}Craft` convention) or another name; must not include "enScore" | before the first release |

## 2026-10-28: Stricter clean-room than PhotoCraft

PhotoCraft measured Photoshop partly from its installed bundle. ScoreCraft does not open any
installed enScore or forScore package at all, and formats without a public spec need a two-team
split. Reason: a score reader's value is mostly behaviour, which public listings and use as a user
describe well enough, and the extra distance removes any doubt about provenance. See
[`clean-room.md`](clean-room.md).

## 2026-10-28: No tiers, no required account

Every enScore Pro and Sync capability is free in ScoreCraft and works offline; sync uses storage the
user already has or a self-hosted server. Reason: the Crafting Apps are free and open source, and
an account requirement would contradict offline-first.

## 2026-10-28: Shared standards copied in

The Crafting Apps' shared standards live in a private repository. ScoreCraft keeps its own copies,
in its own words, in [`standards/`](standards/README.md), so outside contributors can read every
rule they are held to.
