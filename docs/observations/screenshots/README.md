# Reference screenshots

Screenshots of enScore taken by Brinsley as a user (enScore on Windows, 2026-10-28), one
subdirectory per section of the app. They are observation material under
[clean-room §2.4](../../clean-room.md#2-allowed-sources).

**The image files are not committed.** [Clean-room §3](../../clean-room.md#3-forbidden-sources-and-actions)
keeps proprietary screenshots out of the repository, so `*.png` in this folder is gitignored and
the images live on the observer's machine (and in the project's shared files, folder
`enscore-screenshots/`). This index is committed so notes and specs can cite a photo by number,
for example "screenshot 13". Turn what a photo shows into an observation note in your own words
before it feeds a spec.

To use the index, put the images at the paths below (the folder layout is the same in the shared
files).

## Library

| # | File | What it shows | Feature-map ids |
|---|---|---|---|
| 1 | [`library/01-library-default-view.png`](library/01-library-default-view.png) | Default screen when opening the app: the user's library with all the scores they have imported | LIB-3, LIB-14 |
| 2 | [`library/02-library-inside-folder.png`](library/02-library-inside-folder.png) | Inside a folder within the library | LIB-3 |
| 3 | [`library/03-library-more-menu.png`](library/03-library-more-menu.png) | Library toolbar, overflow ("…") menu expanded: title/composer detection, multi-select, switch to list view | LIB-7, LIB-14 |
| 4 | [`library/04-library-sort-menu.png`](library/04-library-sort-menu.png) | Library toolbar, sort menu expanded: alphabetical, recent, by composer | LIB-9 |
| 5 | [`library/05-library-import-menu.png`](library/05-library-import-menu.png) | Library toolbar, import menu expanded: PDFs, folder, scanner, IMSLP, blank manuscript, forScore `.4sc` | LIB-1, LIB-2, IMP-2, IMP-4, IMP-5, PAGE-6 |
| 6 | [`library/06-library-filter-sidebar-bach.png`](library/06-library-filter-sidebar-bach.png) | Filter: a right sidebar for filtering the library. The search field holds "Bach" and the view shows all the user's Bach scores | LIB-5 |

## Lists

| # | File | What it shows | Feature-map ids |
|---|---|---|---|
| 7 | [`lists/07-lists-empty.png`](lists/07-lists-empty.png) | Empty Lists view, no user-created lists | LIST-1 |
| 8 | [`lists/08-lists-header-zoom.png`](lists/08-lists-header-zoom.png) | Zoom of the Lists header toolbar | LIST-1, LIST-5 |
| 9 | [`lists/09-lists-header-more-menu.png`](lists/09-lists-header-more-menu.png) | Lists header, overflow ("…") menu expanded: multi-select, switch to list view | LIST-1 |
| 10 | [`lists/10-lists-header-sort-menu.png`](lists/10-lists-header-sort-menu.png) | Lists header, sort menu expanded: alphabetical, recent, custom | LIST-1 |
| 11 | [`lists/11-lists-one-empty-list-recital.png`](lists/11-lists-one-empty-list-recital.png) | One empty list created, called "Recital" | LIST-1 |
| 12 | [`lists/12-lists-recital-selected-expanded.png`](lists/12-lists-recital-selected-expanded.png) | "Recital" list selected and its view expanded | LIST-1 |
| 13 | [`lists/13-lists-add-scores-dialog.png`](lists/13-lists-add-scores-dialog.png) | The list's add-scores dialog: library folders (each with a count of the items it holds), library items with image previews, and a search field | LIST-1, LIST-6 |
| 14 | [`lists/14-lists-add-scores-two-selected.png`](lists/14-lists-add-scores-two-selected.png) | Two scores selected to add to the list; the confirm button's label changes to count them ("Add 2 Scores") | LIST-1 |
| 15 | [`lists/15-lists-add-scores-nested-folders.png`](lists/15-lists-add-scores-nested-folders.png) | The add-scores dialog with folders expanded, showing nested folders | LIST-1, LIB-3 |
| 16 | [`lists/16-lists-recital-two-scores.png`](lists/16-lists-recital-two-scores.png) | "Recital" list with two scores: cover image, score count, open-list button, entries with reorder handles | LIST-1, LIST-4 |
| 17 | [`lists/17-lists-more-menu-zoom.png`](lists/17-lists-more-menu-zoom.png) | Zoom of the list's "…" menu expanded (visible in photo 16): rename, change image, delete list | LIST-1 |

## Adding screenshots

- One subdirectory per app section, in lower case (`library/`, `lists/`, later `recordings/`,
  `practice/`, `search/`, `settings/`, …).
- Name files `<photo number, two digits>-<section>-<short description>.png`, and keep numbering
  global across sections so a photo number is unique.
- Add a row to the table for the section, with the feature-map ids the photo informs.
