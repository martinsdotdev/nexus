---
status: "accepted"
date: 2026-05-13
decision-makers: project owner
consulted: 
informed: future contributors
---

# Record architecture decisions

## Context and Problem Statement

Nexus is in active design. The brainstorming phase has produced ~30 architecturally significant decisions, hexagonal layering, the Decider pattern, the editor's aesthetic band, the theming model, the obs-websocket integration approach, the rename from "preset" to "theme", and more. Many of these decisions involved comparing real alternatives and rejecting some on specific grounds.

These rationales currently live in the brainstorming conversation and partially in the design spec. The spec captures the *result* (the design as it stands) but not the *rejected alternatives* or the *reasons we chose against them*. Six months from now, when a contributor wonders "why is the data layer event-sourced instead of just a Zustand store?", the spec answers "it is" but not "because we wanted free undo/redo, replay, time-travel debugging, and adapter-swappable persistence." The why is load-bearing for future decisions.

How do we capture decisions in a way that:
- Persists the reasoning, not just the conclusion?
- Stays append-only as decisions evolve, preserving history?
- Is greppable, version-controllable, and reviewable in PRs?
- Matches a known format so contributors can read and write it without learning a bespoke convention?

## Decision Drivers

* Durable record of *why*, not just *what*.
* Append-only history: when a decision is overturned, both records remain.
* Format familiar to engineers (low onboarding cost).
* Pure-text, version-controlled (no proprietary tool dependency).
* Compatible with code-review workflow (ADRs go through PRs).
* Compatible with the design spec, which has a different (descriptive, current-state) function.

## Considered Options

* MADR 4.0 (Markdown Architectural Decision Records)
* Nygard-style ADRs (the original 2011 format)
* Inline rationale in the design spec
* RFC-style design documents (one per major decision)
* No durable record (rationale lives in PRs and conversations)

## Decision Outcome

Chosen option: **"MADR 4.0"**, because it is the most-adopted ADR format in 2024–2026, has a typed front matter for status/date/decision-makers, supports several template variants (full, minimal, bare) calibrated to decision weight, ships an existing template we can copy verbatim, and integrates cleanly with PR review (ADRs are markdown files in `docs/decisions/`).

### Consequences

* **Good**, because every architecturally significant decision will have a durable, scannable rationale record.
* **Good**, because the format is conventional, engineers joining the project can read ADRs immediately without learning a custom format.
* **Good**, because the spec and ADRs can complement each other: the spec describes the current design, ADRs describe how we got here and why.
* **Good**, because superseding ADRs (rather than editing in place) preserves history, future readers can see what we changed our minds about and why.
* **Bad**, because writing an ADR is more effort than not writing one; some decisions will skip the process and rationale will be lost.
* **Bad**, because the ADR backlog (the ~30 decisions already made during brainstorming) needs to be backfilled to be useful. This is a meaningful one-time investment.
* **Neutral**, because the directory will accumulate files over the project's lifetime; we may need to introduce subdirectories (`product/`, `architecture/`, `ui/`, `integration/`) if the count grows past ~30.

### Confirmation

* The first contributor who joins the project should be able to navigate `docs/decisions/README.md` and understand what an ADR is, when to write one, and how to find a relevant past decision in under five minutes.
* Future PRs that introduce architecturally significant decisions (the bar is documented in the README) should include the relevant ADR; PR review checks for this.
* When the spec is updated to reflect a decision change, a new superseding ADR should accompany the spec edit.

## Pros and Cons of the Options

### MADR 4.0

* **Good**, because most-adopted ADR format in 2024–2026; large corpus of examples; familiar to engineers.
* **Good**, because typed front matter (status, date, decision-makers) is machine-readable for indexing and tooling.
* **Good**, because three template variants (full, minimal, bare) calibrate effort to decision weight.
* **Good**, because [`adr-tools`](https://github.com/npryce/adr-tools) and similar CLIs exist for ADR creation and indexing.
* **Neutral**, because the template is opinionated about section order; minor learning curve.
* **Bad**, because the format is more verbose than Nygard's original, which can deter shorter ADRs.

### Nygard-style ADRs (original 2011 format)

* **Good**, because the original; shortest possible ADR format (Title, Status, Context, Decision, Consequences).
* **Good**, because lower writing friction than MADR.
* **Bad**, because lacks structured fields (no front matter, no decision-makers, no per-option pros/cons).
* **Bad**, because the format hasn't evolved to handle modern needs (superseding, status lifecycle, decision drivers).

### Inline rationale in the design spec

* **Good**, because zero new infrastructure; rationale lives next to the design.
* **Bad**, because the spec is current-state-only; rejected alternatives clutter it without serving its purpose.
* **Bad**, because superseding a decision means editing the spec in place, losing history.
* **Bad**, because PR reviewers have to read the whole spec to find the rationale for one decision.

### RFC-style design documents

* **Good**, because RFCs allow long-form rationale with full context.
* **Bad**, because RFCs are heavier than ADRs and typically reserved for cross-cutting concerns; we'd write fewer of them and lose smaller decisions.
* **Bad**, because RFC format isn't standardized; we'd invent our own.

### No durable record

* **Good**, because zero overhead.
* **Bad**, because rationale evaporates the moment the conversation closes; future maintainers reinvent the discussion.
* **Bad**, because no append-only history; we can't see what we changed our minds about.
* **Bad**, because PR archaeology is slow and incomplete; not every PR captures rationale.

## More Information

- MADR specification: [adr.github.io/madr](https://adr.github.io/madr/)
- MADR GitHub repository: [github.com/adr/madr](https://github.com/adr/madr)
- Original Nygard post (2011): "Documenting Architecture Decisions"
- The design spec this ADR practice complements: [`docs/superpowers/specs/2026-05-13-stream-overlay-editor-design.md`](../superpowers/specs/2026-05-13-stream-overlay-editor-design.md)

### Backlog of ADRs to backfill

The brainstorming phase produced decisions that should be captured retroactively. Suggested backlog, roughly in priority order:

1. Front-end-only scope for v1 (defer backend, multi-platform integrations)
2. Hexagonal architecture + functional core / imperative shell
3. Decider pattern for state mutation; event-sourced state
4. Result type at port boundaries (Railway-oriented programming, calibrated)
5. Discriminated-union typestate on `Workspace.mode`, `ObsConnection`, `Layout.status`
6. Modular monolith with calibrated cross-context strictness
7. Read-only types end-to-end, no DTOs
8. Editor aesthetic: Linear/Tldraw minimal-confident band
9. Three-pane editor layout (palette / canvas / inspector)
10. Smart guides + snap-to-edge over pixel grid
11. Live/Draft mode toggle (configurable per-session)
12. obs-websocket v5 integration with auto-detect + manual URL fallback
13. First-class scenes mapped to OBS scenes
14. Library of saved layouts (not single-layout)
15. Chat-command bindings + control-panel manual fire (interactivity v1)
16. CSS-custom-property theming over CSS-in-JS
17. OKLCH color space (not HSL/hex)
18. shadcn/ui token vocabulary + ReUI semantic state extensions
19. Two-token-universe architecture (editor vs overlay-runtime)
20. Per-theme pair philosophy (one accent + one neutral)
21. M3 + Fluent 2 token additions (`--border-subtle`, `--invert`, `--ease-emphasized`, stroke widths, `--radius-none/circular`)
22. Mock chat/event sources behind ports
23. localStorage + BroadcastChannel persistence (no backend in v1)
24. Editor controller pattern (Tldraw-inspired)
25. Theme as first-class concept (rename from "preset")
26. Adopt Feature-Sliced Design as macro organization (FSD layers + hexagonal segment discipline within)
27. v2 HTTP API design philosophy, when we add a backend, we ship "HTTP API," not "REST API"; no HATEOAS for single-frontend consumption; evaluate by Postel + ease-of-learning + difficulty-of-misuse (per [Florian Kraemer, 2025](https://florian-kraemer.net/software-architecture/2025/07/07/Most-RESTful-APIs-are-not-really-RESTful.html)); concrete decisions on versioning, deprecation, and schema documentation deferred until the backend is in scope.

Backfilling these is a multi-PR effort. Suggested approach: prioritize the architectural three (hexagonal, decider, typestate) first since they shape every other decision, then work down the list as time allows. Items 26–27 are recent additions; both are forward-looking and can wait until they're actively load-bearing.
