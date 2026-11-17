# Prioritised task backlog

P0 = must precede everything; P1 = needed for Compressor MVP; P2 = later apps. Acceptance criteria are testable.

| ID | Pri | Task | Depends on | Acceptance criteria |
|---|---|---|---|---|
| SUITE-001 | P0 | Choose project license and name; legal check of name | — | License file in repo; name search documented |
| SUITE-002 | P0 | Create monorepo (Cargo workspace + pnpm), CI for Windows/Linux/macOS | 001 | CI builds an empty workspace on 3 OSes |
| SUITE-003 | P0 | Third-party license register + CI license check (cargo-deny) | 002 | CI fails on disallowed license |
| SUITE-004 | P1 | cs-store | 002 | See shared-engines.md |
| SUITE-005 | P1 | cs-settings | 004 | See shared-engines.md |
| SUITE-006 | P1 | cs-queue | 004 | See shared-engines.md |
| SUITE-007 | P1 | cs-media (probe + encode) | 006 | See shared-engines.md |
| SUITE-008 | P1 | cs-ui-kit v0 | 002 | Storybook with 10 components, a11y checks pass |
| CMP-001…CMP-020 | P1 | Compressor MVP | 004–008 | See compressor/tasks.md |
| SUITE-010 | P2 | cs-undo, cs-doc, cs-text, cs-render | 004 | per engine spec |
| PXL-* , KEY-*, PAG-*, NUM-*, FRM-* | P2 | App MVPs in roadmap order | 010 | per app tasks.md |
| SUITE-020 | P2 | cs-anim, cs-timeline | 010 | per engine spec |
| SUITE-030 | P2 | cs-audio, cs-plugin | 004 | per engine spec |
| MOT-*, FCP-*, LOG-*, MST-* | P2 | App MVPs | 020/030 | per app tasks.md |
| SUITE-040 | P3 | Launcher + cs-ipc | any 2 apps | Launcher lists installed apps; apps run without it |
| RES-001 | Research | Obtain lawfully owned Apple app bundles / Mac and run REA static analysis | user | evidence.md in each app updated with Evidence IDs |
| RES-002 | Research | Crawl each Apple user guide fully (menus, shortcuts, settings) | — | Each ui-ux.md has complete lists with URLs |
