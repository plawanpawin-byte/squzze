# Final Cut Pro — evidence ledger

Evidence classes: **DOC** documented behavior (Apple guides/developer docs, via web search), **3P** third-party reverse-engineering/community material (search snippets only, not opened or verified), **INF** inference, **UNK** unknown, **BLK** blocked (needs binary/runtime unavailable here). No finding in this folder is *verified binary evidence*: no Apple binary was available.

### FINAL-CUT-PRO-A01
- **Source / artifact:** Apple documentation / third-party pages listed in README
- **App version:** unknown
- **Method:** web search + reading result summaries
- **Observed evidence:** A Library is a package containing Events and Projects; clips in Events reference media rather than being the media.
- **Interpretation:** taken at face value; see limitations
- **Class:** DOC (documented behavior)
- **Confidence:** medium
- **Reproduction:** open the listed source URLs
- **Limitations:** search-result summaries, pages not individually opened


### FINAL-CUT-PRO-A02
- **Source / artifact:** Apple documentation / third-party pages listed in README
- **App version:** unknown
- **Method:** web search + reading result summaries
- **Observed evidence:** Generated media (render, optimized, proxy, analysis files) are stored inside the library or in user-chosen external locations and can be deleted and regenerated.
- **Interpretation:** taken at face value; see limitations
- **Class:** DOC (documented behavior)
- **Confidence:** medium
- **Reproduction:** open the listed source URLs
- **Limitations:** search-result summaries, pages not individually opened


### FINAL-CUT-PRO-A03
- **Source / artifact:** analyst reasoning
- **App version:** unknown
- **Method:** reasoning from documented behaviour
- **Observed evidence:** Object model is likely a database/graph of media references, clips, compound clips and project timelines, with a render cache keyed by effect graph. Not confirmed.
- **Interpretation:** plausible, unconfirmed
- **Class:** INF (inference)
- **Confidence:** low
- **Reproduction:** n/a
- **Limitations:** n/a


### FINAL-CUT-PRO-A04
- **Source / artifact:** analyst reasoning
- **App version:** unknown
- **Method:** reasoning from documented behaviour
- **Observed evidence:** Internal engine split (decode, effect graph, GPU compositor, cache) is not publicly documented.
- **Interpretation:** plausible, unconfirmed
- **Class:** UNK (unknown)
- **Confidence:** n/a
- **Reproduction:** n/a
- **Limitations:** n/a


### FINAL-CUT-PRO-B01
- **Source / artifact:** n/a
- **App version:** unknown
- **Method:** file-system search on this PC
- **Observed evidence:** No Final Cut Pro binary/app bundle exists on this machine (Windows 11; searched Downloads, D:, user profile for .app/.dmg/.pkg/.ipa/package names). Static analysis with REA/Ghidra not performed.
- **Interpretation:** cannot be assessed
- **Class:** BLK (blocked)
- **Confidence:** n/a
- **Reproduction:** search the file system for the app bundle
- **Limitations:** n/a


### FINAL-CUT-PRO-B02
- **Source / artifact:** n/a
- **App version:** unknown
- **Method:** file-system search on this PC
- **Observed evidence:** Runtime investigation of Final Cut Pro is impossible on Windows (macOS/iPadOS app).
- **Interpretation:** cannot be assessed
- **Class:** BLK (blocked)
- **Confidence:** n/a
- **Reproduction:** search the file system for the app bundle
- **Limitations:** n/a
