# Logic Pro — evidence ledger

Evidence classes: **DOC** documented behavior (Apple guides/developer docs, via web search), **3P** third-party reverse-engineering/community material (search snippets only, not opened or verified), **INF** inference, **UNK** unknown, **BLK** blocked (needs binary/runtime unavailable here). No finding in this folder is *verified binary evidence*: no Apple binary was available.

### LOGIC-PRO-A01
- **Source / artifact:** Apple documentation / third-party pages listed in README
- **App version:** unknown
- **Method:** web search + reading result summaries
- **Observed evidence:** A project is a package (.logicx) or folder; package saving duplicates assets, folder saving references them.
- **Interpretation:** taken at face value; see limitations
- **Class:** DOC (documented behavior)
- **Confidence:** medium
- **Reproduction:** open the listed source URLs
- **Limitations:** search-result summaries, pages not individually opened


### LOGIC-PRO-A02
- **Source / artifact:** Apple documentation / third-party pages listed in README
- **App version:** unknown
- **Method:** web search + reading result summaries
- **Observed evidence:** Package contains Alternatives/NNN/ProjectData (undocumented little-endian binary), MetaData.plist, Resources/ProjectInformation.plist; backups under Alternatives/000. Source: community parsers, not verified here.
- **Interpretation:** taken at face value; see limitations
- **Class:** 3P (third-party material (not verified here))
- **Confidence:** low-medium
- **Reproduction:** open the listed source URLs
- **Limitations:** search-result summaries, pages not individually opened


### LOGIC-PRO-A03
- **Source / artifact:** analyst reasoning
- **App version:** unknown
- **Method:** reasoning from documented behaviour
- **Observed evidence:** Real-time audio graph with per-track channel strips, MIDI event layer, and plug-in hosting through Audio Units.
- **Interpretation:** plausible, unconfirmed
- **Class:** INF (inference)
- **Confidence:** low
- **Reproduction:** n/a
- **Limitations:** n/a


### LOGIC-PRO-A04
- **Source / artifact:** analyst reasoning
- **App version:** unknown
- **Method:** reasoning from documented behaviour
- **Observed evidence:** Audio engine threading, buffer scheduling and automation interpolation are undocumented.
- **Interpretation:** plausible, unconfirmed
- **Class:** UNK (unknown)
- **Confidence:** n/a
- **Reproduction:** n/a
- **Limitations:** n/a


### LOGIC-PRO-B01
- **Source / artifact:** n/a
- **App version:** unknown
- **Method:** file-system search on this PC
- **Observed evidence:** No Logic Pro binary/app bundle exists on this machine (Windows 11; searched Downloads, D:, user profile for .app/.dmg/.pkg/.ipa/package names). Static analysis with REA/Ghidra not performed.
- **Interpretation:** cannot be assessed
- **Class:** BLK (blocked)
- **Confidence:** n/a
- **Reproduction:** search the file system for the app bundle
- **Limitations:** n/a


### LOGIC-PRO-B02
- **Source / artifact:** n/a
- **App version:** unknown
- **Method:** file-system search on this PC
- **Observed evidence:** Runtime investigation of Logic Pro is impossible on Windows (macOS/iPadOS app).
- **Interpretation:** cannot be assessed
- **Class:** BLK (blocked)
- **Confidence:** n/a
- **Reproduction:** search the file system for the app bundle
- **Limitations:** n/a
