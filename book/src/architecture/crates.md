# Crates and dependency layers

```text
L6  ui-egui  automation
L5  engine
L4  io  sync  metadata  imslp  scan  input  devices
L3  render  gpu  format
L2  ops  ink  layout  text  search  algo  audio  practice
L1  doc
L0  geom  raster  pdf  codecs  midi  fsc
```

A crate depends only on lower layers, enforced by `cargo xtask layers`. `pdf`, `midi` and `fsc`
are standalone. `devices` is the only crate allowed FFI. Details:
[`docs/architecture.md` §2–3](https://github.com/Brinsleym/enscore-cleanroom/blob/main/docs/architecture.md#2-workspace-layout).
