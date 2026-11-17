# Motion — evidence ledger

Evidence classes: **DOC** documented behavior (Apple guides/developer docs, via web search), **3P** third-party reverse-engineering/community material (search snippets only, not opened or verified), **INF** inference, **UNK** unknown, **BLK** blocked (needs binary/runtime unavailable here). No finding in this folder is *verified binary evidence*: no Apple binary was available.

### MOTION-A01
- **Source / artifact:** Apple documentation / third-party pages listed in README
- **App version:** unknown
- **Method:** web search + reading result summaries
- **Observed evidence:** Replicator builds static copies in a pattern (not a simulation); particle systems comprise an emitter and life cycle.
- **Interpretation:** taken at face value; see limitations
- **Class:** DOC (documented behavior)
- **Confidence:** medium
- **Reproduction:** open the listed source URLs
- **Limitations:** search-result summaries, pages not individually opened


### MOTION-A02
- **Source / artifact:** Apple documentation / third-party pages listed in README
- **App version:** unknown
- **Method:** web search + reading result summaries
- **Observed evidence:** Behaviors (e.g. Scale Over Life) animate objects procedurally.
- **Interpretation:** taken at face value; see limitations
- **Class:** DOC (documented behavior)
- **Confidence:** medium
- **Reproduction:** open the listed source URLs
- **Limitations:** search-result summaries, pages not individually opened


### MOTION-A03
- **Source / artifact:** analyst reasoning
- **App version:** unknown
- **Method:** reasoning from documented behaviour
- **Observed evidence:** Project file structure was not confirmed by search (.motn XML claim UNVERIFIED).
- **Interpretation:** plausible, unconfirmed
- **Class:** UNK (unknown)
- **Confidence:** n/a
- **Reproduction:** n/a
- **Limitations:** n/a


### MOTION-A04
- **Source / artifact:** analyst reasoning
- **App version:** unknown
- **Method:** reasoning from documented behaviour
- **Observed evidence:** Layer/object tree with keyframe curves and behavior evaluators, GPU compositing.
- **Interpretation:** plausible, unconfirmed
- **Class:** INF (inference)
- **Confidence:** low
- **Reproduction:** n/a
- **Limitations:** n/a


### MOTION-B01
- **Source / artifact:** n/a
- **App version:** unknown
- **Method:** file-system search on this PC
- **Observed evidence:** No Motion binary/app bundle exists on this machine (Windows 11; searched Downloads, D:, user profile for .app/.dmg/.pkg/.ipa/package names). Static analysis with REA/Ghidra not performed.
- **Interpretation:** cannot be assessed
- **Class:** BLK (blocked)
- **Confidence:** n/a
- **Reproduction:** search the file system for the app bundle
- **Limitations:** n/a


### MOTION-B02
- **Source / artifact:** n/a
- **App version:** unknown
- **Method:** file-system search on this PC
- **Observed evidence:** Runtime investigation of Motion is impossible on Windows (macOS/iPadOS app).
- **Interpretation:** cannot be assessed
- **Class:** BLK (blocked)
- **Confidence:** n/a
- **Reproduction:** search the file system for the app bundle
- **Limitations:** n/a
