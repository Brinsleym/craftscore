# Fuzzing and regression testing

`cargo fuzz` targets for `pdf` (parse and render one page), `fsc`, `midi`, the library loader, the
setlist bundle reader and the control-channel request parser. CI runs each target for a short
budget on every PR touching it and longer on a schedule. Every crash found becomes a minimized
regression test.
