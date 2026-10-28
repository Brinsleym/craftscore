# Localization parity

> **Last reviewed:** 2026-10-28 · **Last updated:** 2026-10-28 · **Change:** major (first version, all rows missing) · **Target:** enScore 3.7 (2026)

enScore's App Store listing names 15 locales; the listings call it "twelve languages". ScoreCraft's
UI strings go through `tl!` and `crates/ui-egui/src/i18n/<lang>.tsv`; coverage will be measured by
`cargo xtask i18n-coverage`. Translations are clean-room: written from the meaning of our English
text ([`ui-design.md`](ui-design.md#localisation)).

| Language | Code | enScore | UI strings | Status |
|---|---|---|---:|---|
| English | `en` | yes | 0% | none |
| Dutch | `nl` | yes | 0% | none |
| French | `fr` | yes | 0% | none |
| German | `de` | yes | 0% | none |
| Greek | `el` | yes | 0% | none |
| Italian | `it` | yes | 0% | none |
| Japanese | `ja` | yes | 0% | none |
| Korean | `ko` | yes | 0% | none |
| Polish | `pl` | yes | 0% | none |
| Portuguese (Brazil) | `pt-br` | yes | 0% | none |
| Portuguese (Portugal) | `pt` | yes | 0% | none |
| Russian | `ru` | yes | 0% | none |
| Simplified Chinese | `zh-hans` | yes | 0% | none |
| Spanish | `es` | yes | 0% | none |
| Swedish | `sv` | yes | 0% | none |
| Traditional Chinese | `zh-hant` | yes | 0% | none |

Also needed beyond enScore: right-to-left layout (Arabic, Hebrew) and music terms (key names,
"D.S. al Coda") localized with each language's conventions (H for B in German, solfège in French
and Italian key names).

## Revision history

| Date | Change | Summary |
|---|---|---|
| 2026-10-28 | major | First version |
