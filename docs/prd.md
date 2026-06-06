# Nexus, Product Requirements

**Status:** Scaffold (sections defined; content TBD section-by-section as we fill in).
**Last updated:** 2026-05-13
**Related:** [`docs/architecture.md`](architecture.md), [`docs/milestones.md`](milestones.md), [`docs/superpowers/specs/2026-05-13-stream-overlay-editor-design.md`](superpowers/specs/2026-05-13-stream-overlay-editor-design.md)

This PRD captures *what* Nexus is, *for whom*, and *why*. It complements the engineering spec (which captures *how*) and the milestones doc (which captures *when*).

## 1. Problem

TBD, working framing in spec §1.1 and §1.2 (the gap in the streaming-overlay landscape Nexus addresses; the assembly-of-services pain that streamers tolerate today).

## 2. Vision

TBD, extension of the wedge (a single overlay replacing the usual pile of services) into a multi-version, multi-persona vision.

## 3. Personas

Pointer to spec §1.6 (pending, the persona-ladder edits surfaced earlier are not yet committed to the spec).

When filled in, this section will carry, per persona:
- Identity and motivations
- Acquisition channel
- Friction tolerance
- Willingness to pay (for the relevant tiers)
- Anti-personas (who Nexus is explicitly not for)

## 4. Use cases / user stories

Grouped by persona. User-story format: *"As a [persona], I want to [action] so that [outcome]."*

- Developer-streamer: TBD
- Designer-streamer: TBD
- Mass-market streamer (v2): TBD
- Agency / multi-streamer manager (v2+): TBD
- Partnered / megastar (v3+): TBD
- Esports broadcast production (v3+): TBD

## 5. Functional requirements

To be derived from spec §3 (architecture), §6 (ports & adapters), §7 (widget contract), §11 (editor UX), §10 (OBS integration). The PRD restates these as user-facing capabilities, not engineering interfaces.

Top-level groupings:
- Editor, what the streamer can do in `/edit`
- Overlay, what the runtime renders in `/overlay`
- Server, what the daemon does behind the scenes
- Sources, chat / event / OBS integrations

## 6. Non-functional requirements

- Performance, pointer to spec §14.3 budgets (TBD: per-persona reliability targets that may exceed those numbers)
- Reliability, TBD; different per persona tier (developer-streamer tolerates restarts; esports broadcast does not)
- Accessibility, WCAG AA minimum at the editor surface; TBD which features get AAA
- Privacy / data residency, local-first by default; cloud-mode data handling TBD per region
- Security, TBD; covers auth, secrets, sandboxing of custom themes (v2+), webhook signature verification (v2+ platform integrations)

## 7. Success metrics

TBD, per-persona adoption, engagement, retention, NPS.

Open question worth surfacing now: what defines "success" for v1 specifically? Adoption count? GitHub stars? Usage telemetry (which we haven't decided whether to ship)? Number of streams using Nexus on-air? "Used in production by N developer-streamers" is the most honest v1 metric.

## 8. Out of scope (v1)

Pointer to spec §1.5 cuts. Brief restatement here for product-team readability.

## 9. Roadmap

See [`docs/milestones.md`](milestones.md).

## 10. Open questions

- DESIGN.md adoption strategy for themes (hybrid / full / inspiration-only)
- Appearance × Theme coupling (A keep / B orthogonal / C per-theme contrast knob)
- Density semantics, cut, keep, or relocate
- Whether v1 ships with usage telemetry (privacy ↔ product-improvement tradeoff)
- Pricing model for v2 hosted offering
- Whether to ship a v1.5 with a Tauri installer before v2 cloud
- Long-term: marketplace for community themes, widgets, layouts
