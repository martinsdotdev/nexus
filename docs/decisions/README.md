# Architecture Decision Records

This directory holds Nexus's **Architecture Decision Records (ADRs)**, written in [MADR 4.0](https://adr.github.io/madr/) format.

## What is an ADR?

> An Architectural Decision (AD) is a software design choice that addresses a functional or non-functional requirement that is architecturally significant.
>, MADR

ADRs capture the *why* behind decisions: the problem, the options considered, the chosen path, and the consequences. They complement the design spec (`docs/superpowers/specs/`), which captures the *what*, the resulting design as a whole.

Use ADRs for decisions that are:
- Architecturally significant, they shape structure, contracts, or trade-offs that future contributors must respect.
- Hard or expensive to reverse, if undoing it requires rewriting more than a single module, write an ADR.
- Multi-option, there were real alternatives we considered and rejected, and the rejection rationale is load-bearing for future readers.

Skip ADRs for:
- Implementation tactics that fit in a single file or PR description.
- Decisions trivially derivable from the spec or the codebase.
- Personal style preferences.

## Naming

`NNNN-title-with-dashes.md`

- `NNNN`, consecutive four-digit number, starting at `0000`. Reserved: `0000` is the meta-ADR establishing the ADR practice.
- `title-with-dashes`, short, lowercase, dash-separated, descriptive (≤ 8 words).
- Examples:
  - `0001-adopt-hexagonal-architecture.md`
  - `0014-rename-presets-to-themes.md`

## Status lifecycle

```
proposed  →  accepted  →  deprecated
                       →  superseded by ADR-NNNN
                       →  rejected (rare; usually deprecated)
```

A `proposed` ADR is a draft seeking review. `accepted` means it's in force. `deprecated` means we no longer apply it but the historical record stays. `superseded` links to the replacing ADR.

ADRs are append-only and date-stamped at the time of authoring. If a decision changes, **author a new ADR** that supersedes the old one, never edit accepted ADRs to reflect new thinking. The history is the point.

## Templates

Three templates ship with this directory; choose the one that matches the decision's weight:

| Template | When to use |
|---|---|
| [`adr-template.md`](adr-template.md) | **Default.** Full template with decision drivers, options, consequences, per-option pros/cons, confirmation. Use for any decision where the rationale is non-trivial. |
| [`adr-template-minimal.md`](adr-template-minimal.md) | Quick ADRs that still need rationale. Drops the per-option pros/cons table. |
| [`adr-template-bare.md`](adr-template-bare.md) | Empty skeleton with only headings, for power users who don't need prompt text. |

Copy a template, rename, fill in.

## Categorization

This directory is currently flat. If the ADR count grows past ~30 or natural clusters emerge, we'll move to subdirectories along these lines:

```
docs/decisions/
├── product/         scope, audience, wedge, monetization
├── architecture/    hexagonal layers, decider, ports & adapters, modular boundaries
├── ui/              editor UX, theming, design tokens, motion
└── integration/     OBS, platform sources, persistence
```

For now: flat directory, numerically ordered.

## Relationship to the spec

The spec at [`docs/superpowers/specs/`](../superpowers/specs/) is the *current* design, the present-tense source of truth for what we build. ADRs are the *history*, how we got here, what we chose against, what assumptions are load-bearing. Where they overlap:

- The spec describes *the chosen approach*. The ADR describes *why we chose it over alternatives*.
- An ADR should link to the spec section that realizes it.
- The spec doesn't repeat ADR rationale; it can reference an ADR for "why this and not X."

## Index

| # | Title | Status | Date |
|---|---|---|---|
| [0000](0000-record-architecture-decisions.md) | Record architecture decisions | accepted | 2026-05-13 |
| [0001](0001-use-rfc-9457-problem-details-for-http-errors.md) | Use RFC 9457 Problem Details for HTTP error responses | accepted | 2026-05-13 |
| [0002](0002-local-or-cloud-rust-core-with-qubit-rpc.md) | Local-or-cloud Rust core with qubit RPC and SvelteKit UI | superseded by [0005](0005-offline-collaborative-loro-crdt-trusted-relay.md) | 2026-05-13 |
| [0003](0003-dark-first-pro-creative-tool-editor-aesthetic.md) | Dark-first pro-creative-tool editor aesthetic | accepted | 2026-05-27 |
| [0004](0004-responsive-editor-shell.md) | Responsive editor shell via CSS-owned reflow, container-query panels, and off-canvas drawers | accepted | 2026-05-28 |
| [0005](0005-offline-collaborative-loro-crdt-trusted-relay.md) | Offline-collaborative local-first architecture on Loro CRDT with a trusted sync relay | accepted | 2026-05-28 |
| [0006](0006-data-driven-custom-themes.md) | Data-driven custom themes with a hybrid registry and token linking | superseded by [0007](0007-pure-data-driven-themes.md) | 2026-05-29 |
| [0007](0007-pure-data-driven-themes.md) | Pure data-driven themes (every theme is registry data; built-ins are seeded) | accepted | 2026-05-29 |
| [0008](0008-adopt-ark-ui.md) | Adopt Ark UI (headless) for the editor component layer | accepted | 2026-05-26 |

When adding an ADR, append a row to this table.

## License

The MADR template itself is dual-licensed MIT / CC0-1.0 (creator's choice). ADRs authored here adopt the project's overall license (TBD).
