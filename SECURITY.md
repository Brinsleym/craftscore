# Security Policy

ScoreCraft parses complex documents (PDF, images, audio, `.4sc` archives, setlist bundles), talks to
the network (IMSLP, public catalogues, sync backends, eSCL scanners) and exposes CLI, JSON control
and MCP automation. Treat files, paths, metadata, network responses, MIDI and pedal input,
automation requests and command parameters as potentially malicious.

## Reporting a vulnerability

Please give the maintainers a reasonable opportunity to investigate and coordinate a fix before
publishing exploitable details, proof-of-concept code or weaponized files.

Use GitHub's **private vulnerability reporting** for this repository (Security ▸ Report a
vulnerability) when it is enabled. If it is not, contact the repository owner through their GitHub
profile, give only a minimal non-exploitable summary in public, and ask for a private transfer
method before sharing details. Do not open a public issue containing a working exploit, a secret,
a sensitive crash dump or a malicious attachment.

## What to include

- affected ScoreCraft version, commit or release artefact;
- operating system, architecture, build profile and feature flags;
- the affected component or path;
- security impact and required attacker access;
- exact reproduction steps, expected and actual behaviour;
- crash logs, backtraces, sanitizer output or resource measurements;
- whether it reproduces from a clean checkout, and any workaround.

Remove credentials, personal paths, private scores and unrelated system information from logs.

## Malicious test files

Do not attach a potentially harmful or non-redistributable file to a public issue. First give its
format, size, sha256, observed effect, provenance (synthetic, fuzz-generated) and the command that
reproduces the problem. Test files accepted into the corpus must be minimized, free of personal or
proprietary data (no copyrighted scores), legally redistributable, and documented with the expected
failure behaviour. A fixed parser defect gets a permanent regression test.

## Scope

- PDF, image, audio, `.4sc`, setlist bundle and library parsing;
- decompression bombs, oversized dimensions, integer overflow, memory exhaustion, recursion, hangs;
- import/export and filesystem path handling (traversal, symlinks, zip slip, unsafe overwrite);
- engine command validation and (de)serialization;
- the control server, headless JSON server, MCP server and MCP-to-GUI bridge;
- network clients (IMSLP, catalogues, eSCL) and sync backends and server, including
  authentication, TLS, end-to-end encryption and data deletion;
- MIDI and HID input handling;
- dependencies, CI, packaging, signing, checksums and release integrity.

General bugs without a security impact go through the normal issue process.

## Current status

No code has been released. The planned controls are described in the
[security documentation](book/src/security/overview.md); this section will list what is
implemented and what is not, release by release.
