# Architecture overview

ScoreCraft is engine-first: a pure-data library model (`doc`), a command engine (`engine`) that
every frontend talks to, and thin frontends (egui app, CLI, control channel, MCP). The imported PDF
is immutable; every edit and marking is data in source-page coordinates.

The full design plan: [`docs/architecture.md`](https://github.com/Brinsleym/enscore-cleanroom/blob/main/docs/architecture.md).
