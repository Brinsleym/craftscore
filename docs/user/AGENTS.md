# User docs: rules for agents and contributors

Everything in this folder (except this file, CLAUDE.md and README.md) is end-user help for
musicians, to be published on the project website once it exists. The file path is the URL.
Renaming a file breaks links, so add the old path to `redirect_from` when you rename.

## Who reads this

Musicians using ScoreCraft, people answering their questions, and AI assistants looking things up
for either. Write for a musician, not a developer.

## Page rules

- One question or problem per page. Lead with the answer or the fix in the first two lines.
- Plain words, short sentences, numbered steps for anything the reader does.
- Start the body at `##`. The page title comes from frontmatter.
- Link to other pages instead of repeating them, with relative links to the `.md` file.
- No marketing copy, no "simply", no claims you have not checked in the code or a release.
- Call the product "ScoreCraft". Never describe enScore's or forScore's UI; describe ours.
- Name UI labels exactly as the app shows them, in **bold**. Menu paths read `Library › Import…`.
- Troubleshooting pages: `## Symptoms`, `## Fix`, `## Why this happens` (optional), `## Still stuck?`.

## Frontmatter

    ---
    title: Turn pages with a Bluetooth pedal
    description: One sentence for search results, link previews and AI summaries.
    order: 10                 # optional, sorts within the folder (lower first)
    verified: 0.1.0           # optional, app version you last checked this page against
    redirect_from: [old/path] # optional
    ---

Set `verified` only after checking the page against that release tag. Pages describe the latest
release, not unreleased code on main. Until the first release, pages are drafts and carry no
`verified` key.

## Folders

- `getting-started/` install and first run
- `guides/` how to do one thing
- `troubleshooting/` symptom → fix
- `faq/` short answers
- `reference/` lists and settings

A folder's `index.md` names the folder in the sidebar. Files or folders starting with `_` are
drafts and are not published.
