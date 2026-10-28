# Command engine

Every user-visible action is a command: `id`, `label`, menu path, shortcut, params doc,
`enabled` predicate and `run`. `run` validates its params, returns JSON and records exactly one
history step. Tap zones, pedals, MIDI, menus, the palette, the CLI, the control channel and MCP all
dispatch commands by id. Ids come from the [feature map](https://github.com/Brinsleym/enscore-cleanroom/blob/main/docs/feature-map.md).

`cargo xtask parity` compares the registry with the feature catalog and writes
[`docs/parity-checklist.md`](https://github.com/Brinsleym/enscore-cleanroom/blob/main/docs/parity-checklist.md). The `panic_hunt` test dispatches every
command with adversarial params.
