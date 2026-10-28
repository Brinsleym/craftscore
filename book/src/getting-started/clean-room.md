# Clean-room methodology

ScoreCraft rebuilds enScore's behaviour from public information and use as a user only, through a
three-step trail: **observe** (notes in `docs/observations/`), **specify** (behavioural specs in
`docs/spec/`), **implement** (code that cites the spec). Proprietary packages are never opened,
decompiled or traced, and formats without a public spec are learned black-box by one person and
implemented by another.

The binding rules: [`docs/clean-room.md`](https://github.com/Brinsleym/enscore-cleanroom/blob/main/docs/clean-room.md).
