# Pages — evidence ledger

Evidence classes: **DOC** documented behavior (Apple guides/developer docs, via web search), **3P** third-party reverse-engineering/community material (search snippets only, not opened or verified), **INF** inference, **UNK** unknown, **BLK** blocked (needs binary/runtime unavailable here). No finding in this folder is *verified binary evidence*: no Apple binary was available.

### PAGES-A01
- **Source / artifact:** Apple documentation / third-party pages listed in README
- **App version:** unknown
- **Method:** web search + reading result summaries
- **Observed evidence:** Same IWA container family as Keynote/Numbers.
- **Interpretation:** taken at face value; see limitations
- **Class:** 3P (third-party material (not verified here))
- **Confidence:** low-medium
- **Reproduction:** open the listed source URLs
- **Limitations:** search-result summaries, pages not individually opened


### PAGES-A02
- **Source / artifact:** analyst reasoning
- **App version:** unknown
- **Method:** reasoning from documented behaviour
- **Observed evidence:** Document model: sections, paragraphs/styles, floating and inline objects, tables, with pagination engine.
- **Interpretation:** plausible, unconfirmed
- **Class:** INF (inference)
- **Confidence:** low
- **Reproduction:** n/a
- **Limitations:** n/a


### PAGES-A03
- **Source / artifact:** analyst reasoning
- **App version:** unknown
- **Method:** reasoning from documented behaviour
- **Observed evidence:** Text layout and pagination internals.
- **Interpretation:** plausible, unconfirmed
- **Class:** UNK (unknown)
- **Confidence:** n/a
- **Reproduction:** n/a
- **Limitations:** n/a


### PAGES-B01
- **Source / artifact:** n/a
- **App version:** unknown
- **Method:** file-system search on this PC
- **Observed evidence:** No Pages binary/app bundle exists on this machine (Windows 11; searched Downloads, D:, user profile for .app/.dmg/.pkg/.ipa/package names). Static analysis with REA/Ghidra not performed.
- **Interpretation:** cannot be assessed
- **Class:** BLK (blocked)
- **Confidence:** n/a
- **Reproduction:** search the file system for the app bundle
- **Limitations:** n/a


### PAGES-B02
- **Source / artifact:** n/a
- **App version:** unknown
- **Method:** file-system search on this PC
- **Observed evidence:** Runtime investigation of Pages is impossible on Windows (macOS/iPadOS app).
- **Interpretation:** cannot be assessed
- **Class:** BLK (blocked)
- **Confidence:** n/a
- **Reproduction:** search the file system for the app bundle
- **Limitations:** n/a
