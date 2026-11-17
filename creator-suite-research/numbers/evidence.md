# Numbers — evidence ledger

Evidence classes: **DOC** documented behavior (Apple guides/developer docs, via web search), **3P** third-party reverse-engineering/community material (search snippets only, not opened or verified), **INF** inference, **UNK** unknown, **BLK** blocked (needs binary/runtime unavailable here). No finding in this folder is *verified binary evidence*: no Apple binary was available.

### NUMBERS-A01
- **Source / artifact:** Apple documentation / third-party pages listed in README
- **App version:** unknown
- **Method:** web search + reading result summaries
- **Observed evidence:** ZIP of images and Snappy-compressed protobuf IWA files holding metadata, text, cell definitions.
- **Interpretation:** taken at face value; see limitations
- **Class:** 3P (third-party material (not verified here))
- **Confidence:** low-medium
- **Reproduction:** open the listed source URLs
- **Limitations:** search-result summaries, pages not individually opened


### NUMBERS-A02
- **Source / artifact:** Apple documentation / third-party pages listed in README
- **App version:** unknown
- **Method:** web search + reading result summaries
- **Observed evidence:** numbers-parser ships protobuf definitions extracted from a copy of Numbers; newer Numbers can produce unreadable files.
- **Interpretation:** taken at face value; see limitations
- **Class:** 3P (third-party material (not verified here))
- **Confidence:** low-medium
- **Reproduction:** open the listed source URLs
- **Limitations:** search-result summaries, pages not individually opened


### NUMBERS-A03
- **Source / artifact:** Apple documentation / third-party pages listed in README
- **App version:** unknown
- **Method:** web search + reading result summaries
- **Observed evidence:** Formula text extraction is supported optionally; early versions relied on cached values.
- **Interpretation:** taken at face value; see limitations
- **Class:** 3P (third-party material (not verified here))
- **Confidence:** low-medium
- **Reproduction:** open the listed source URLs
- **Limitations:** search-result summaries, pages not individually opened


### NUMBERS-A04
- **Source / artifact:** analyst reasoning
- **App version:** unknown
- **Method:** reasoning from documented behaviour
- **Observed evidence:** Formula engine and dependency graph internals.
- **Interpretation:** plausible, unconfirmed
- **Class:** UNK (unknown)
- **Confidence:** n/a
- **Reproduction:** n/a
- **Limitations:** n/a


### NUMBERS-B01
- **Source / artifact:** n/a
- **App version:** unknown
- **Method:** file-system search on this PC
- **Observed evidence:** No Numbers binary/app bundle exists on this machine (Windows 11; searched Downloads, D:, user profile for .app/.dmg/.pkg/.ipa/package names). Static analysis with REA/Ghidra not performed.
- **Interpretation:** cannot be assessed
- **Class:** BLK (blocked)
- **Confidence:** n/a
- **Reproduction:** search the file system for the app bundle
- **Limitations:** n/a


### NUMBERS-B02
- **Source / artifact:** n/a
- **App version:** unknown
- **Method:** file-system search on this PC
- **Observed evidence:** Runtime investigation of Numbers is impossible on Windows (macOS/iPadOS app).
- **Interpretation:** cannot be assessed
- **Class:** BLK (blocked)
- **Confidence:** n/a
- **Reproduction:** search the file system for the app bundle
- **Limitations:** n/a
