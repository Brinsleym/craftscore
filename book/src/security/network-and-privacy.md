# Network, sync and privacy

- **Opt-in.** No network access until the user enables a feature that needs it (IMSLP, catalogue
  lookups, a sync backend). `SCORECRAFT_NO_NETWORK=1` refuses all of it.
- **No telemetry.** ScoreCraft sends no analytics or crash reports unless the user chooses to.
- **TLS** with platform or Mozilla roots; no certificate pinning bypass options.
- **Minimal data.** Catalogue lookups send only the text being looked up. Requests carry a generic
  User-Agent and no personal data.
- **Sync** credentials live in the OS keychain; optional end-to-end encryption keeps content
  unreadable to the backend; removing a backend deletes remote copies and keeps local ones.
