# Contributors in the About window

**About ▸ Contributors** credits everyone who contributed to ScoreCraft, and **About ▸ Models**
credits the AI models named in `Co-Authored-By` trailers, as in the other Crafting Apps.

## Decision

- Two files, both **compiled into the binary** by the UI crate's `build.rs` (nothing is read from
  disk or the network at run time):
  - `contributors/contributors.json`: commit stats per GitHub username. **Generated** by
    `cargo xtask contributors`; never hand-edit.
  - [`contributors/people.toml`](../contributors/people.toml): the names people chose for
    themselves. **Each contributor adds only their own entry**, in their own PR.
- Each contributor gets one line: GitHub username (always), display name (opt-in, or their synced
  public GitHub profile name), real name (only if they told us), merged PRs, commits, lines added
  and deleted, first and last commit date.

## Credit yourself

Add your entry to `contributors/people.toml` in a PR you commit yourself (every field optional):

```toml
[people.your-github-username]
real_name = "Your Name"
display_name = "Nickname"
sync_github_name = true   # use the name on your public GitHub profile as display name
```

Delete your entry to remove your names. `cargo xtask contributors --check` verifies that each entry
was committed by its own user (or by a maintainer with a `consent_source`).

## Refreshing (maintainers)

`cargo xtask contributors` regenerates `contributors.json` from git history and the GitHub API.
