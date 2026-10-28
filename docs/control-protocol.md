# Control protocol

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** major (first version, design ahead of M3) · **Target:** enScore 3.7 (2026)

How agents and scripts drive the running ScoreCraft app. Same design as PhotoCraft's control
channel: JSON lines over an authenticated loopback TCP socket, with the engine's commands as the
main vocabulary.

## Starting it

```sh
scorecraft --control            # prints  control: 127.0.0.1:<port> token=<64 hex chars>
scorecraft-cli mcp --bridge     # MCP server that forwards to the running app
scorecraft-cli serve            # headless JSON server (no window)
```

- Binds `127.0.0.1` only. The first line of every connection must be
  `{"auth":"<token>"}`; the 256-bit token is random per launch unless `SCORECRAFT_CONTROL_TOKEN`
  is set.
- One request per line, one reply per line: `{"id":1,"method":"…","params":{…}}` →
  `{"id":1,"result":…}` or `{"id":1,"error":{"code":"…","message":"…"}}`.

## Methods

| Method | Params | Result |
|---|---|---|
| `command` | `{"id":"view.turn.next","params":{}}` | the command's JSON result |
| `command_batch` | `{"steps":[{"id":…,"params":…}],"stop_on_error":true}` | per-step results |
| `command_list` | `{"filter":"annotate."}` | ids, labels, params docs, enabled state |
| `ui.state` | `{}` | the serde UI state (`ui-egui/src/state.rs`): open reader, page, layout, tool, panels |
| `ui.set` | `{"path":"reader.layout","value":"twoPage"}` | new state |
| `ui.pointer` | `{"kind":"pen","phase":"down|move|up","x":…,"y":…,"pressure":0.6,"tilt":[0,0]}` | ok |
| `ui.key` | `{"key":"ArrowRight","modifiers":[]}` | ok |
| `ui.pedal` | `{"profile":"default","button":"right"}` | ok (simulates a mapped pedal) |
| `ui.midi` | `{"bytes":[176,64,127]}` | ok (simulates incoming MIDI) |
| `ui.screenshot` | `{"scale":1}` | base64 PNG, rendered offscreen |
| `ui.dialog.set` / `ui.dialog.confirm` | `{"field":…,"value":…}` | dialog state |
| `jobs.list` / `jobs.cancel` | `{}` / `{"job":id}` | background jobs (render, import, OCR, sync) |
| `prefs.get` / `prefs.set` | `{"key":"turn.halfPage.split"}` | one preference at a time |

## Engine commands

All feature-map commands are available through `command`. Their params docs follow the same
mini-schema as PhotoCraft's (`{"name":type=default}`), for example:

```text
view.layout.set        {"mode":"single|twoPage|scroll"="single"}
view.turn.next         {"half":bool?}
annotate.stroke        {"page":int,"layer":id?,"tool":"pen|highlighter|whiteout"="pen","colour":"#rrggbbaa"?,"width":pt=1.5,"points":[[x,y,pressure?,t?]]}
list.entry.add         {"list":id,"score":id,"pageRange":[int,int]?,"tempo":bpm?}
score.export.pdf       {"score":id,"layers":[id]?,"flatten":bool=true,"out":path}
```

Commands that touch the filesystem take paths only inside the configured workspace roots
(below); anything else is refused at the automation boundary.

## Transport limits

- Request line ≤ 8 MiB, batch ≤ 1,000 steps, ≤ 8 concurrent connections, per-command timeout.
- Screenshots and exports larger than the limit return an error with the size.

## Security

- Read and write workspace roots are configured separately (`--control-read-root`,
  `--control-write-root`); defaults are the library directory only.
- Account, sync-credential and network-enabling preferences cannot be changed over the channel.
- See [the automation security model](../book/src/automation/security-model.md).

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-28 | major | First version, adapted from PhotoCraft's control protocol |
