# Command-line interface

`scorecraft-cli` subcommands: `import`, `info`, `export`, `run --cmd <id> --params <json>`,
`batch`, `commands [--json]`, `serve`, `mcp [--bridge]`. Unknown flags are rejected, every
subcommand answers `--help`, and batch runs never overwrite an existing output silently.
Examples: [`docs/development.md`](https://github.com/Brinsleym/enscore-cleanroom/blob/main/docs/development.md#driving-the-app-programmatically).
