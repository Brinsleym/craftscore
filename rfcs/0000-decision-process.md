# RFC-0000: How we make decisions

Adapted from the Crafting Apps' [RFC-0000](https://github.com/storytold/craft-contrib/blob/main/rfcs/0000-decision-process.md).

Changes to this process, to [`docs/clean-room.md`](../docs/clean-room.md), to
[`docs/standards/`](../docs/standards/README.md), to the parity weights, and other decisions that
affect the whole project are made through RFCs: pull requests that add `rfcs/NNNN-short-title.md`.
This process is changed the same way.

- **Maintainers** are the people with write access to this repository. **Owners** are the
  repository's admins, and are also maintainers.
- Anyone can open an RFC or comment on one.
- A maintainer starts the comment period by commenting its end date: **7 days** for changing the
  clean-room rules, the licence or this process; **3 days** for everything else.
- After the period, the RFC merges with approval from a maintainer other than the author, if no
  maintainer's objection is unresolved. An objection gives a reason and says what would resolve it.
- An owner can decide an objection, or merge or close an RFC early, and says why on the PR.
- While the project has a single maintainer, that maintainer may merge their own RFC after the
  comment period, and says so on the PR.

## RFC template

```markdown
# RFC-NNNN: Title

- **Author:** @username   **Started:** YYYY-MM-DD   **Comment period ends:** YYYY-MM-DD

## Summary
## Motivation
## Proposal
## Alternatives considered
## Clean-room impact
## Unresolved questions
```
