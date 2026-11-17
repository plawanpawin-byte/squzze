# Master roadmap

**Constraint from the user:** research only for now. No implementation starts until the user approves. The phases below are a plan.

## Implementation order and rationale
| Order | App | Why here | Effort (solo, rough) | Main risk |
|---|---|---|---|---|
| 1 | **Compressor-alt** | Smallest scope, FFmpeg does the heavy work, validates queue/store/settings engines, immediately useful | 2–3 months MVP | Codec patents / FFmpeg license variants |
| 2 | Pixelmator-alt | Establishes cs-render, cs-doc, cs-undo | 6–9 months | Brush engine, color management |
| 3 | Keynote-alt | Reuses doc/render, adds text/animation | 6–9 months | Text layout quality |
| 4 | Pages-alt | Reuses cs-text/cs-doc; hardest part is pagination | 6–9 months | DOCX fidelity |
| 5 | Numbers-alt | cs-calc is a contained project | 4–6 months | Function coverage/XLSX fidelity |
| 6 | Freeform-alt | Scene graph + ink; light | 3–4 months | Real-time collab (optional, later) |
| 7 | Motion-alt | Needs mature cs-render/cs-anim | 9–12 months | GPU effect breadth |
| 8 | Final Cut-alt | Needs timeline, render cache, proxy pipeline | 12+ months | Scale and performance |
| 9 | Logic-alt | Real-time audio, plug-in hosting | 12+ months | Real-time robustness; consider integrating existing DAW code |
| 10 | MainStage-alt | Builds on cs-audio/cs-plugin | 4–6 months after Logic | ASIO/low-latency on Windows |
| — | Launcher | Optional, trivial once cs-ipc exists | 2–4 weeks | None |

Estimates are planning guesses, not measurements.

## Milestones
- **M0 (now):** research complete, specs reviewed by the user.
- **M1:** Repo + CI + shared crates skeleton (cs-store, cs-settings, cs-queue, cs-media).
- **M2:** Compressor MVP (see compressor/implementation-spec.md).
- **M3:** Compressor v1.0 (watch folders, presets, signed Windows installer, Linux/macOS builds).
- **M4+:** subsequent apps in the order above.

## Decision gates
1. Before M1: user approves tech stack and project/license choice (GPL-3.0-or-later vs Apache-2.0/MIT; see licensing-register.md).
2. Before app 8–10: re-evaluate Qt vs web UI for dense timeline/mixer UIs.
3. Before any Apple-format importer: legal review.
