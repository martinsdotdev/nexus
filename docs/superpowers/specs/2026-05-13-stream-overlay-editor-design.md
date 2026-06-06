# Nexus, Stream Overlay Editor (v1 Design Spec)

**Date:** 2026-05-13
**Status:** Draft, awaiting user review
**Scope:** First implementation slice, front-end overlay engine + visual editor + obs-websocket integration. No backend, no real platform integrations.

> This spec is framework-agnostic. Contracts are expressed in TypeScript notation as a universal type vocabulary; any language with discriminated unions and structural typing (TS, F#, Kotlin, Rust, Scala, OCaml, Swift) can implement them. Specific libraries (React, Zustand, etc.) are deferred to a reference-implementation appendix.

---

## 1. Context

### 1.1 What this is

Nexus is a **local-first stream overlay engine and visual editor**. A streamer composes an overlay in a browser-based editor, copies a URL into OBS as a Browser Source, and the runtime renders the composition transparently over their game capture. The editor and overlay are two routes of the same statically-built single-page application; both read the same configuration from local storage.

### 1.2 Why this exists

The competitive landscape (StreamElements, Streamlabs, Nerd or Die, OWN3D) is dense but the existing tools share two limitations:
- Their default aesthetic vocabulary is generic; sharp design identity is reserved for paid premium packs.
- Viewer-side interactivity (chat commands triggering overlay reactions, channel-point integrations, polls) is bolted on through ad-hoc widgets rather than treated as a first-class primitive.

### 1.3 Audience

Non-technical streamers who currently assemble overlays from multiple services (alerts from StreamElements, chat box from Streamlabs, music widget from Pretzel, tip jar from Ko-fi, custom HTML from a friend who codes). They want it to look good, work reliably, and not require five accounts.

### 1.4 Wedge

Unification of fragmented widgets is the wedge: one overlay in place of the usual pile of services, with taste-grade design as its visible expression. Curated aesthetic identities (four themes in v1) ship at portfolio quality. Viewer interactivity is a first-class primitive baked into the runtime from day one.

### 1.5 v1 scope (this spec)

In scope:
- Two routes: a visual editor and a transparent overlay renderer.
- A typed widget contract; eight built-in widgets ported from the prototype.
- A four-layer composition model (Workspace → Layout → Scene → WidgetInstance).
- Four aesthetic themes (Cozy, Cyber, Editorial, Sticker), each a complete CSS-custom-property override.
- Mock chat and event sources behind ports designed for real-source swap-in.
- A chat-command-binding system: trigger pattern → named event published to the overlay event bus.
- obs-websocket v5 integration: auto-detect, password-challenge auth, browser-source creation, bidirectional scene mapping.
- Filesystem persistence (Rust server) with versioned schema and migration support; cloud-database persistence designed-for but deferred (see [ADR-0002](../../decisions/0002-local-or-cloud-rust-core-with-qubit-rpc.md)).
- Live sync between the editor and overlay routes. **Revised by [ADR-0005](../../decisions/0005-offline-collaborative-loro-crdt-trusted-relay.md):** sync is now an offline-collaborative Loro CRDT replicated to every client and merged by a trusted relay, not qubit subscriptions from a single-source-of-truth server.
- A Live/Draft mode toggle.

Out of scope (deferred):
- User accounts, cloud sync, multi-device.
- Real platform integrations (Twitch, YouTube, Kick), see v2 spec.
- Template gallery beyond a single curated default layout.
- Marketplace, community widget sharing, third-party widget loading.
- Polls, predictions, channel-point-redemption widgets.
- AI-driven theme generation or tuning.
- Mobile or tablet editor support.

---

## 2. Product principles

These nine rules are non-negotiable and apply to every decision in the codebase.

1. **The shell can call the core, but the reverse is not allowed.** (Bittencourt.) Enforced at lint level.
2. **The core has no time, no randomness, no I/O, no DOM, no storage.** Time, randomness, and I/O are injected through ports. The core consumes values, never produces them.
3. **State mutates only through Command → Decide → Events → Evolve.** No direct setter on workspace state. Undo is event reversal.
4. **`decide` is pure and total over (command, state). `evolve` only sets fields, adds to lists, increments, or toggles flags.** All business logic lives in `decide`; `evolve` is mechanical.
5. **Operations that can fail return `Result<T, E>` at port boundaries. No throws across ports.** Inside the pure core, return a result from the top-level decide; don't ceremony every internal helper.
6. **Aggregate lifecycle states are distinct types where the state machine has ≥3 states with state-gated operations.** Specifically: `Workspace.mode` (Live | Draft), `ObsConnection` (Disconnected | Connecting | Connected | Failed), `Layout` (Active | Archived).
7. **Bounded contexts publish integration events; they do not import each other's modules.** Strictly enforced on boundaries we expect to extract to a backend later (Persistence, Broadcast, Sources). Calibrated for in-process contexts (Composition, Theming), direct imports allowed within the editor.
8. **All types are `readonly` end-to-end. No DTOs.** The data shape leaving the core is the data shape arriving at the UI, the broadcast layer, the persistence layer.
9. **Editor animations are restrained (≤ 300 ms, default 180 ms, `ease-out`, never `ease-in-out`). Overlay-runtime animations earn their length by being rare and communicative.** Two budgets in one codebase.

---

## 3. System architecture

### 3.1 Layered organization (Feature-Sliced Design + hexagonal discipline)

The macro organization follows [Feature-Sliced Design](https://fsd.how/) (FSD), a layered methodology where each layer can only import from layers strictly below it. The micro discipline within each slice follows hexagonal architecture: `model/` segments are the pure functional core (Decider, types, projections); `api/` segments are the imperative shell (adapters, I/O); `ui/` segments are presentation.

Layers, top to bottom (each layer can import from any layer below):

```
src/
├── app/                       FSD layer 1, composition root, runtime entry points
│   ├── editor.tsx             wires CommandPort + adapters; mounts <EditorPage/>
│   ├── overlay.tsx            wires read-only state subscription; mounts <OverlayPage/>
│   └── landing.tsx            static landing route
│
├── pages/                     FSD layer 2, full page views (one per route)
│   ├── edit/
│   │   ├── ui/                page layout: composes editor widgets
│   │   └── index.ts           public API
│   ├── overlay/
│   │   ├── ui/                transparent renderer composition
│   │   └── index.ts
│   └── landing/
│       ├── ui/
│       └── index.ts
│
├── widgets/                   FSD layer 3, large self-contained UI chunks
│   │                          (NOTE: "widgets" here = FSD widgets, including
│   │                          both editor chrome AND overlay renderables)
│   ├── overlay/               our 8 overlay widgets, each rendered by overlay route
│   │   ├── webcam-frame/      { ui/, model/, index.ts }
│   │   ├── chat-box/
│   │   ├── alerts/
│   │   ├── goal-bar/
│   │   ├── follower-bubble/
│   │   ├── now-playing/
│   │   ├── stream-info/
│   │   └── socials/
│   └── editor/                editor chrome widgets
│       ├── canvas/            { ui/, model/, index.ts }, the dragging surface + selection layer
│       ├── inspector/         right panel
│       ├── palette/           left panel
│       ├── toolbar/           top bar (mode toggle, preset picker, etc.)
│       └── layers/            layers list (within inspector)
│
├── features/                  FSD layer 4, reused business actions
│   ├── compose-widget/        add, move, resize, delete, select, multi-select
│   │   ├── model/             decide + evolve for composition commands (PURE)
│   │   ├── ui/                drag handles, selection rectangles
│   │   └── index.ts
│   ├── snap-to-edge/          smart guides + snap algorithm (PURE)
│   ├── live-draft-mode/       Live/Draft toggle, draft branching
│   ├── undo-redo/             event-log reversal
│   ├── theming/               theme selection, accent override
│   ├── obs-control/           obs-websocket connection + scene mapping
│   │   ├── model/             ObsConnection typestate transitions
│   │   ├── api/               obs-websocket-js adapter (I/O)
│   │   └── index.ts
│   ├── obs-detect/            auto-probe localhost:4455
│   ├── persistence/           save/load/migrate
│   │   ├── model/             schema versioning, migrations (PURE)
│   │   ├── api/               LocalStorage adapter
│   │   └── index.ts
│   ├── broadcast/             BroadcastChannel mirror
│   │   ├── api/
│   │   └── index.ts
│   ├── chat-bindings/         pattern → event registry, matcher (PURE)
│   ├── test-fire/             editor toolbar's "fire test event" menu
│   └── layout-library/        save, rename, duplicate, archive layouts
│
├── entities/                  FSD layer 5, domain objects (types + deciders)
│   ├── workspace/
│   │   ├── model/             Workspace type, top-level decider, evolve, isTerminal
│   │   └── index.ts
│   ├── layout/
│   │   └── model/             Layout type, Layout.status typestate
│   ├── scene/
│   │   └── model/             Scene type
│   ├── widget-instance/
│   │   └── model/             WidgetInstance type
│   ├── theme/
│   │   ├── model/             Theme type, ThemeId brand, theme registry
│   │   ├── styles/            built-in theme CSS files
│   │   │   ├── cozy.css
│   │   │   ├── cyber.css
│   │   │   ├── editorial.css
│   │   │   └── sticker.css
│   │   └── index.ts
│   ├── chat-binding/
│   │   └── model/             ChatBinding type
│   └── obs-connection/
│       └── model/             ObsConnection typestate (4 variants)
│
└── shared/                    FSD layer 6, cross-cutting reusables, no business logic
    ├── ui/                    primitive components (Button, Input, Popover, Modal)
    ├── styles/
    │   ├── tokens.css         editor design tokens (typography, color, motion)
    │   └── reset.css
    ├── lib/
    │   ├── result.ts          Result<T, E> type + helpers
    │   ├── id.ts              branded ID type helpers
    │   ├── event-bus.ts       in-process event bus (cross-context communication)
    │   └── ports/             cross-cutting port interfaces consumed by ≥2 features
    │       ├── ClockPort.ts
    │       └── RandomPort.ts
    └── config/                feature flags, env constants
```

### 3.1.1 FSD's hard rule, applied

> *"A module in a slice can only import other slices when they are located on layers strictly below."*

This is FSD's single non-negotiable rule. It enforces our cross-context discipline lexically: `features/compose-widget/` cannot import from `features/persistence/` (same layer). If two features need to share a contract, the contract lives in `shared/lib/` or in the relevant `entities/<x>/model/`. Both features then import from that lower layer.

Practical consequence: hexagonal "ports" naturally live one layer below their consumers. `PersistencePort` lives in `shared/lib/ports/PersistencePort.ts` (or, since persistence is feature-shaped, in `features/persistence/index.ts` as the feature's public API, features expose their ports as part of their public surface).

### 3.1.2 Segment conventions (hexagonal discipline within slices)

Within each FSD slice (a folder in entities/features/widgets/pages), segments are conventionally named:

| Segment | Holds | Purity rule |
|---|---|---|
| `model/` | Types, decider, evolve, projections, schemas | **PURE.** No I/O, no DOM, no time, no randomness. Time and randomness arrive as injected ports. |
| `api/` | I/O adapters, localStorage, obs-websocket, BroadcastChannel | Impure. Returns `Result<T, E>` on failure paths. |
| `ui/` | Presentation components | May depend on `model/` and `shared/ui/`. Imports `api/` only via composition root. |
| `lib/` | Slice-internal utilities | Pure where possible. |
| `config/` | Slice-internal configuration | Static data. |

The nine project rules from §2 apply *within* these segments. The lint configuration enforces `model/` purity (no imports from `api/`, no imports of `Date`, `Math.random`, `localStorage`, `window`, etc.).

### 3.1.3 Public API per slice

Each slice declares its public surface in `index.ts`. Internal files within the slice are not importable from outside. Refactoring within a slice is safe as long as `index.ts` exports the same names.

```typescript
// features/compose-widget/index.ts
export type { ComposeCommand, ComposeEvent } from './model/types'
export { composeDecider } from './model/decider'
export { DragHandle, SelectionRect } from './ui'
// internal: model/snap-utils.ts, ui/internal-styles.module.css, etc., not exported
```

### 3.2 Bounded contexts → FSD slice mapping

The bounded contexts from DDD map onto FSD's `entities/` and `features/` layers. Entities hold domain *types* (data shapes + invariants); features hold *actions* (commands, deciders, the parts that actually do things). Some contexts are pure entity (just a type); most are entity + a feature that operates on it.

| Context | FSD entity (types) | FSD feature (actions) | Owns |
|---|---|---|---|
| **Workspace** | `entities/workspace/` | (top-level decider lives in the workspace entity itself; no separate feature in v1) | `Workspace` type; root decider, evolve, isTerminal |
| **Layout lifecycle** | `entities/layout/` | `features/layout-library/` | `Layout` type; commands for create/rename/duplicate/archive |
| **Composition** | `entities/scene/`, `entities/widget-instance/` | `features/compose-widget/`, `features/snap-to-edge/` | `Scene`, `WidgetInstance`; commands for add/move/resize/delete; snap algorithm |
| **Theming** | `entities/theme/` | `features/theming/` | `Theme`, `ThemeId`, `ThemeOverrides`; theme selection, accent override; built-in CSS files |
| **Bindings** | `entities/chat-binding/` | `features/chat-bindings/` | `ChatBinding` type; pattern → event registry, matcher |
| **OBS integration** | `entities/obs-connection/` | `features/obs-control/`, `features/obs-detect/` | `ObsConnection` typestate; connection lifecycle; obs-ws adapter; scene mapping |
| **Editor mode** | (lives in workspace entity) | `features/live-draft-mode/`, `features/undo-redo/` | Mode toggle, draft branching, event-log reversal |
| **Persistence** | (no domain type) | `features/persistence/` | Schema versioning, migrations (in `model/`), LocalStorage adapter (in `api/`) |
| **Broadcast** | (no domain type) | `features/broadcast/` | BroadcastChannel adapter |

Contexts communicate via integration events published to a shared in-process bus (`shared/lib/event-bus.ts`). FSD's import rule enforces this, features cannot import features at the same layer; the bus interface lives in `shared/lib/`, which both publishers and subscribers may import.

When *direct* communication is unavoidable (e.g., the `compose-widget` decider needs to know the active scene from `workspace`), the dependency goes through `entities/<x>/model/` (one layer below), which is allowed.

### 3.3 The two routes

`/edit` and `/overlay` are routes of the same SvelteKit application, both served by the Rust server. They communicate with the server via qubit RPC (queries, mutations) and qubit subscriptions (live state updates).

- **Editor route** (`/edit`): full editor chrome, three-pane layout, palette, canvas with selection affordances, inspector, layers panel, toolbar. Dispatches mutations to the Rust server; subscribes to workspace updates for live preview. Initiates obs-websocket connection through the server.
- **Overlay route** (`/overlay`): transparent renderer, no editor chrome. Subscribes to workspace state via qubit; never writes. Pure widget composition.
- **Landing route** (`/`): static-ish page with a live overlay demo and an "Open editor" CTA.

The Rust server is the authoritative state holder. Both routes are subscribers; mutations always flow through the server (no client-side workspace mutation outside what the server's Decider produces). This eliminates the editor↔overlay duplication problem and ensures a single source of truth.

---

## 4. Domain model

### 4.1 Core types

```typescript
type SchemaVersion = 1
type LayoutId = string & { readonly __layoutId: unique symbol }
type SceneId = string & { readonly __sceneId: unique symbol }
type WidgetInstanceId = string & { readonly __widgetInstanceId: unique symbol }
type BindingId = string & { readonly __bindingId: unique symbol }
type ThemeId = string & { readonly __themeId: unique symbol }
// Built-in theme IDs are well-known strings: 'cozy' | 'cyber' | 'editorial' | 'sticker'.
// v2+ may admit custom themes with arbitrary ThemeId values registered at runtime.

interface Workspace {
  readonly schemaVersion: SchemaVersion
  readonly mode: EditorMode             // discriminated below
  readonly preferences: Preferences
  readonly layouts: Readonly<Record<LayoutId, Layout>>
  readonly activeLayoutId: LayoutId
}

type EditorMode =
  | { readonly kind: 'live' }
  | { readonly kind: 'draft'; readonly baseline: WorkspaceSnapshot }

interface WorkspaceSnapshot {
  // Identical shape to Workspace but without `mode`, the baseline we reset to when discarding a draft.
  readonly layouts: Readonly<Record<LayoutId, Layout>>
  readonly activeLayoutId: LayoutId
}

interface Preferences {
  readonly appearance: 'light' | 'dark' | 'system'   // editor color appearance (controls .dark class)
  readonly obs: ObsConnection
}

type ObsConnection =
  | { readonly kind: 'disconnected' }
  | { readonly kind: 'connecting'; readonly url: string }
  | { readonly kind: 'connected'; readonly url: string; readonly sessionId: string }
  | { readonly kind: 'failed'; readonly url: string; readonly error: ObsConnectError }

interface Layout {
  readonly id: LayoutId
  readonly name: string
  readonly status: 'active' | 'archived'
  readonly aspect: '16:9' | '9:16'
  readonly scenes: Readonly<Record<SceneId, Scene>>
  readonly activeSceneId: SceneId
  readonly obsSceneMap: Readonly<Record<SceneId, string | null>>
  readonly bindings: ReadonlyArray<ChatBinding>
}

interface Scene {
  readonly id: SceneId
  readonly name: string
  readonly themeId: ThemeId
  readonly overrides: ThemeOverrides
  readonly widgets: ReadonlyArray<WidgetInstance>
}

interface ThemeOverrides {
  readonly accent: string | null
  readonly density: 'compact' | 'normal' | 'spacious' | null
}

interface WidgetInstance {
  readonly id: WidgetInstanceId
  readonly type: string                 // widget type identifier; matches a registered Widget meta
  readonly x: number; readonly y: number   // virtual canvas coordinates (1920×1080 or 1080×1920)
  readonly w: number; readonly h: number
  readonly z: number
  readonly visible: boolean
  readonly locked: boolean
  readonly props: Readonly<Record<string, unknown>>
}

interface ChatBinding {
  readonly id: BindingId
  readonly pattern: string              // literal '!hype' or regex source '^!\\w+$'
  readonly patternKind: 'literal' | 'regex'
  readonly role: 'any' | 'sub' | 'mod' | 'vip' | 'broadcaster'
  readonly event: string                // e.g. 'alert.fire:hype', 'scene.set:brb'
  readonly payload: Readonly<Record<string, unknown>>
}
```

### 4.2 The composition rule

Render-time composition layers, outer to inner:

1. **Layout** sets canvas aspect and which scene is active.
2. **Scene.themeId** loads the theme's CSS custom properties on the canvas root.
3. **Scene.overrides** apply on top, `accent` overrides the theme's accent variable; `density` toggles a class.
4. **WidgetInstance.props** provide content and per-widget configuration.

Switching `Scene.themeId` from `'cozy'` to `'cyber'` is one field assignment. The previous state is reachable by setting it back. Nothing is baked into widget instances at render time.

---

## 5. The Decider

> **Revised by [ADR-0005](../../decisions/0005-offline-collaborative-loro-crdt-trusted-relay.md):** the workspace document is a Loro CRDT, so there is no `evolve`/event log; `decide` becomes a pure validator/repair function (`validate(workspace) -> Vec<Repair>`) run authoritatively by the trusted relay. The Command/Event catalog below still describes the operations usefully, read `evolve` as Loro's merge and `decide` as the validator.

### 5.1 Type signature

Following Chassaing's four-element formulation:

```typescript
interface Decider<C, E, S> {
  readonly initialState: S
  readonly decide: (cmd: C, state: S) => Result<ReadonlyArray<E>, DecideError>
  readonly evolve: (state: S, evt: E) => S
  readonly isTerminal: (state: S) => boolean
}

type Result<T, E> =
  | { readonly ok: true; readonly value: T }
  | { readonly ok: false; readonly error: E }
```

Nexus has **one** top-level Decider: `Decider<Command, Event, Workspace>`. Bounded contexts contribute their own Command and Event variants to the shared union; the top-level `decide` dispatches to per-context deciders by command kind.

### 5.2 Commands and events (illustrative subset)

```typescript
type Command =
  // Workspace context
  | { readonly kind: 'workspace.mode.set'; readonly mode: 'live' | 'draft' }
  | { readonly kind: 'workspace.draft.publish' }
  | { readonly kind: 'workspace.draft.discard' }
  | { readonly kind: 'workspace.layout.create'; readonly name: string; readonly aspect: '16:9' | '9:16' }
  | { readonly kind: 'workspace.layout.rename'; readonly id: LayoutId; readonly name: string }
  | { readonly kind: 'workspace.layout.archive'; readonly id: LayoutId }
  | { readonly kind: 'workspace.layout.activate'; readonly id: LayoutId }

  // Composition context
  | { readonly kind: 'composition.scene.create'; readonly layoutId: LayoutId; readonly name: string; readonly themeId: ThemeId }
  | { readonly kind: 'composition.scene.activate'; readonly layoutId: LayoutId; readonly sceneId: SceneId }
  | { readonly kind: 'composition.widget.add'; readonly layoutId: LayoutId; readonly sceneId: SceneId; readonly widgetType: string; readonly x: number; readonly y: number }
  | { readonly kind: 'composition.widget.move'; readonly layoutId: LayoutId; readonly sceneId: SceneId; readonly widgetId: WidgetInstanceId; readonly x: number; readonly y: number }
  | { readonly kind: 'composition.widget.resize'; readonly layoutId: LayoutId; readonly sceneId: SceneId; readonly widgetId: WidgetInstanceId; readonly w: number; readonly h: number }
  | { readonly kind: 'composition.widget.update-props'; readonly layoutId: LayoutId; readonly sceneId: SceneId; readonly widgetId: WidgetInstanceId; readonly props: Record<string, unknown> }
  | { readonly kind: 'composition.widget.delete'; readonly layoutId: LayoutId; readonly sceneId: SceneId; readonly widgetId: WidgetInstanceId }

  // Theming context
  | { readonly kind: 'theming.theme.set'; readonly layoutId: LayoutId; readonly sceneId: SceneId; readonly themeId: ThemeId }
  | { readonly kind: 'theming.accent.set'; readonly layoutId: LayoutId; readonly sceneId: SceneId; readonly accent: string | null }

  // Bindings context
  | { readonly kind: 'bindings.create'; readonly layoutId: LayoutId; readonly binding: Omit<ChatBinding, 'id'> }
  | { readonly kind: 'bindings.delete'; readonly layoutId: LayoutId; readonly bindingId: BindingId }

  // OBS-sync context
  | { readonly kind: 'obs.connect'; readonly url: string; readonly password: string | null }
  | { readonly kind: 'obs.disconnect' }
  | { readonly kind: 'obs.scene-map.set'; readonly layoutId: LayoutId; readonly sceneId: SceneId; readonly obsSceneName: string | null }

type Event =
  // mirrors of commands, in past tense + timestamp + assigned IDs
  | { readonly kind: 'workspace.mode.changed'; readonly mode: 'live' | 'draft'; readonly at: number }
  | { readonly kind: 'workspace.layout.created'; readonly layout: Layout; readonly at: number }
  | { readonly kind: 'composition.widget.added'; readonly layoutId: LayoutId; readonly sceneId: SceneId; readonly widget: WidgetInstance; readonly at: number }
  | { readonly kind: 'composition.widget.moved'; readonly layoutId: LayoutId; readonly sceneId: SceneId; readonly widgetId: WidgetInstanceId; readonly from: Pos; readonly to: Pos; readonly at: number }
  // ... etc, one event variant per logical state change
```

### 5.3 Decide rules

- `decide` is pure: same `(cmd, state)` always produces the same result.
- `decide` is total: every `Command` variant is handled; unrecognized commands return a `DecideError.UnknownCommand`.
- `decide` produces *zero or more* events. A no-op command (e.g., setting a value to its current value) returns `Result.ok([])`.
- `decide` performs all business-rule validation: bounds checking on widget positions, uniqueness constraints, mode-gated operations, role checks on binding triggers.
- ID generation is *not* the responsibility of `decide`. IDs are generated upstream (in the imperative shell) using injected randomness, and passed into the command as needed. Alternatively, `decide` may accept an `idProvider` parameter passed at the boundary, but it must remain pure.

### 5.4 Evolve rules

> Verbatim from Chassaing: *"should probably not be more than setting a field, adding an element to a list, incrementing a value, or setting/resetting a flag."*

- `evolve` performs *one* state transition per event variant.
- `evolve` does not validate. Validation happened in `decide`; `evolve` trusts the event.
- `evolve` does not recompute derived data unless that derivation is trivially mechanical (e.g., incrementing a counter).
- `evolve` is total over `(state, event)`: every event variant has an explicit branch.

### 5.5 isTerminal

Nexus's overall Workspace is never terminal (it's a long-lived editing aggregate). Sub-aggregates have terminal-like states:

- `Layout.status === 'archived'` is terminal for composition commands targeting that layout; the decider rejects them.
- `ObsConnection.kind === 'failed'` is terminal for further send commands; the user must explicitly retry-connect to transition out.

The `isTerminal` function on the top-level Decider always returns `false` for the Workspace aggregate; per-sub-aggregate terminal checks live inside `decide` as guards.

---

## 6. Ports & adapters

### 6.1 Primary port

```typescript
interface CommandPort {
  dispatch(cmd: Command): Promise<Result<ReadonlyArray<Event>, DispatchError>>
  subscribe(handler: (e: Event) => void): Unsubscribe
  state(): Workspace                    // read-only snapshot
}
```

UI calls `dispatch`. Tests call `dispatch`. The CommandPort implementation is the imperative shell that wraps the Decider, it loads the current state, calls `decide`, applies events through `evolve`, persists, broadcasts, and returns the events to the caller.

### 6.2 Secondary ports

```typescript
interface PersistencePort {
  load(): Promise<Result<Workspace | null, LoadError>>
  save(workspace: Workspace): Promise<Result<void, SaveError>>
  clear(): Promise<Result<void, SaveError>>
}

interface BroadcastPort {
  publish(event: Event | OverlayEvent): void
  subscribe(handler: (event: Event | OverlayEvent) => void): Unsubscribe
}

interface ChatSourcePort {
  start(): Promise<Result<void, ChatSourceError>>
  stop(): Promise<void>
  onMessage(handler: (msg: ChatMessage) => void): Unsubscribe
  status(): ChatSourceStatus
}

interface EventSourcePort {
  start(): Promise<Result<void, EventSourceError>>
  stop(): Promise<void>
  onEvent(handler: (evt: PlatformEvent) => void): Unsubscribe
  status(): EventSourceStatus
}

interface ObsControlPort {
  connect(url: string, password: string | null): Promise<Result<ObsSession, ObsConnectError>>
  disconnect(): Promise<void>
  listScenes(): Promise<Result<ReadonlyArray<string>, ObsError>>
  createBrowserSource(args: {
    sceneName: string
    sourceName: string
    url: string
    width: number
    height: number
  }): Promise<Result<void, ObsError>>
  setActiveScene(name: string): Promise<Result<void, ObsError>>
  onSceneChanged(handler: (sceneName: string) => void): Unsubscribe
  onDisconnected(handler: (reason: string) => void): Unsubscribe
}

interface ClockPort { now(): number }
interface RandomPort { uuid(): string; pick<T>(xs: ReadonlyArray<T>): T }
```

### 6.3 Adapters provided in v1 and where they live

Under FSD organization, each port and its v1 adapter live in the feature that owns that I/O concern. The port interface is exported from the feature's `index.ts` (public API); the adapter implementation lives in the feature's `api/` segment.

| Port | FSD location (interface + v1 adapter) | v1 adapter | v2 adapter (planned) |
|---|---|---|---|
All ports live in the Rust `nexus-core` crate (`crates/nexus-core/src/ports/`). v1 adapters live in `nexus-server` (`crates/nexus-server/src/adapters/`). The SvelteKit UI consumes the server's qubit-exposed handlers, it does not implement ports.

| Port | Rust location | v1 adapter (local mode) | v2 adapter (cloud mode) |
|---|---|---|---|
| `Persistence` trait | `nexus-core::ports::persistence` | `FilePersistence`, JSON files in OS app-data dir (via `dirs` crate) | `DatabasePersistence`, Postgres via `sqlx` (deferred) |
| `Auth` trait | `nexus-core::ports::auth` | `NoOpAuth` (single tenant, local binary owner) | `OAuthAuth` or session-based (deferred) |
| `ObsControl` trait | `nexus-core::ports::obs_control` | `ObwsAdapter`, Rust `obws` crate, native obs-ws v5 client |, (same in cloud, talks to streamer's OBS via tunnel, design deferred) |
| `ChatSource` trait | `nexus-core::ports::chat_source` | `MockChatSource` (scripted demo content) | `TwitchChatSource`, `YouTubeChatSource`, `KickChatSource` (v2) |
| `EventSource` trait | `nexus-core::ports::event_source` | `MockEventSource` (scripted alerts at jittered intervals) | Real platform adapters (v2) |
| `Clock` trait | `nexus-core::ports::clock` | `SystemClock`, `std::time::SystemTime` |, |
| `Random` trait | `nexus-core::ports::random` | `OsRandom`, `uuid::Uuid::new_v4()` |, |

Two architectural shifts from the previous design:

1. **`CommandPort` and `BroadcastPort` are subsumed by qubit.** The qubit-generated TypeScript client *is* the command port for the UI (typed query/mutation methods). qubit subscriptions *are* the broadcast mechanism. No separate port interfaces exist on the UI side.
2. **Adapters are Rust, not JS.** `ObwsAdapter` replaces `ObsWebsocketAdapter` (Rust `obws` crate instead of `obs-websocket-js`). `FilePersistence` replaces `LocalStoragePersistence`. The UI never sees the adapter implementations, it sees the qubit API surface.

### 6.4 Mock source contracts

The mock sources are not throwaway scaffolds, they are first-class adapters that drive demos, drive automated UI tests, and validate that the editor and overlay work without any external service. They implement the same ports as the real adapters and are toggleable at the `app/` composition root (e.g., via env flag or query parameter).

`MockChatSource` emits chat messages on a configurable schedule from a curated message pool with simulated user metadata (subscriber tier, mod flag, vip flag).

`MockEventSource` emits platform events (alert.follow, alert.subscribe, alert.cheer, alert.raid) at jittered intervals so the overlay always shows live activity even when no real stream is happening.

---

## 7. Widget contract

### 7.1 Widget meta descriptor

Each widget type registers a `WidgetMeta` describing how it's added, how its props are edited, and how it renders. The runtime never knows about specific widget types, only the meta descriptors.

```typescript
interface WidgetMeta<Props extends Record<string, unknown> = Record<string, unknown>> {
  readonly type: string                              // unique identifier; matches WidgetInstance.type
  readonly displayName: string                       // shown in palette and inspector header
  readonly category: WidgetCategory                  // 'core' | 'chat' | 'alerts' | 'goals' | 'media' | 'info'
  readonly icon: string                              // Lucide icon name
  readonly defaultProps: Props
  readonly defaultSize: { readonly w: number; readonly h: number }
  readonly minSize: { readonly w: number; readonly h: number }
  readonly maxSize: { readonly w: number; readonly h: number } | null
  readonly propSchema: PropSchema<Props>             // drives inspector form
  readonly inlineEdit: InlineEditMap<Props>          // declares which props are inline-editable
  readonly subscriptions: ReadonlyArray<string>      // event-bus event kinds this widget cares about (e.g., 'chat.message', 'alert.*')
}
```

### 7.2 Prop schema

The prop schema is a typed, declarative description of each prop. The editor's inspector reads it and generates appropriate form fields without per-widget custom code.

```typescript
type PropSchema<Props> = {
  readonly [K in keyof Props]: PropField<Props[K]>
}

type PropField<T> =
  | { readonly kind: 'text'; readonly label: string; readonly placeholder?: string; readonly maxLength?: number }
  | { readonly kind: 'multiline'; readonly label: string; readonly rows?: number }
  | { readonly kind: 'number'; readonly label: string; readonly min?: number; readonly max?: number; readonly step?: number; readonly unit?: string }
  | { readonly kind: 'boolean'; readonly label: string }
  | { readonly kind: 'color'; readonly label: string }
  | { readonly kind: 'select'; readonly label: string; readonly options: ReadonlyArray<{ readonly value: T; readonly label: string }> }
  | { readonly kind: 'group'; readonly label: string; readonly fields: PropSchema<T extends Record<string, unknown> ? T : never> }
```

### 7.3 Inline edit declarations

A widget that wants double-click-to-edit-in-place on certain props declares them:

```typescript
type InlineEditMap<Props> = {
  readonly [K in keyof Props]?: InlineEditKind
}

type InlineEditKind = 'text' | 'color'
```

The selection layer in the editor reads this map and renders an inline-edit affordance over the widget for matching prop names. Inline edit dispatches the same `composition.widget.update-props` command as the inspector form.

### 7.4 Widget component contract

A widget component (in any rendering framework) receives:

```typescript
interface WidgetRenderArgs<Props> {
  readonly props: Props                        // typed by the widget's prop schema
  readonly size: { readonly w: number; readonly h: number }
  readonly bus: EventBus                       // subscribe to platform events
  readonly clock: ClockPort                    // for time-aware widgets (now-playing progress, etc.)
  readonly mode: 'edit' | 'overlay'            // some widgets render differently in editor (e.g., placeholders)
}
```

The widget is responsible for its own internal layout within its declared `size`. Widgets MUST NOT reach outside their bounding box. Widgets MUST NOT block the main thread for more than 1 frame (16 ms). Widgets MUST cleanup subscriptions on unmount.

### 7.5 v1 widgets

| Type | Category | What it does |
|---|---|---|
| `webcam-frame` | core | Decorative frame for the streamer's webcam capture. Six shape variants (squircle, circle, hex, pill, pixel, blob). Presented as a placeholder in editor; transparent cutout in overlay so OBS composites the real webcam under it. |
| `chat-box` | chat | Auto-scrolling chat with badges, emote support, user-color coloring. Subscribes to `chat.message`. |
| `alerts` | alerts | Animated alert presenter, follow, subscribe, cheer, raid, donation. Subscribes to `alert.*`. Queue-aware (one at a time, FIFO, configurable durations). |
| `goal-bar` | goals | Progress bar with current/target labels. Subscribes to `goal.increment`. Configurable goal type (sub, follower, donation). |
| `follower-bubble` | info | Rotating recent-follower display. Subscribes to `alert.follow`. |
| `now-playing` | media | Current track display with animated equalizer. Subscribes to `media.track-changed`. Manual fallback (streamer types track via inspector). |
| `stream-info` | info | Title + game + uptime + viewer count. Subscribes to `stream.info-changed`. |
| `socials` | info | Strip of social handles. Static configuration; no event subscriptions. |

---

## 8. Theming system

> **Mechanism superseded by [ADR-0007](../../decisions/0007-pure-data-driven-themes.md)** (which supersedes [ADR-0006](../../decisions/0006-data-driven-custom-themes.md)). Themes are no longer CSS files selected by `[data-theme]`. Every theme, the four built-ins included, is **data** in the workspace doc's `themes` registry; the relay seeds the built-ins as protected entries, and `resolveThemeStyle` applies the resolved tokens as **inline CSS variables** (no `[data-theme]` cascade, no per-theme CSS file, no `evolve`). The token vocabulary in §8.1 below remains accurate and canonical; the *mechanism* described in §8.1–§8.3 (CSS files, `data-theme` swap, evolve) is historical. See the ADR for the current model.

### 8.1 Themes as CSS files

Each theme is a CSS file that overrides the shared token vocabulary (defined in §12.2) on `[data-theme="<name>"]`. Variable names match the shared schema; only values differ.

```css
/* themes/cozy.css, applied on the overlay-runtime root */
[data-theme="cozy"] {
  /* Surfaces */
  --background:            oklch(95% 0.02 80);
  --foreground:            oklch(15% 0.02 80);
  --card:                  oklch(98% 0.02 80);
  --card-foreground:       oklch(15% 0.02 80);
  --popover:               oklch(98% 0.02 80);
  --popover-foreground:    oklch(15% 0.02 80);
  --muted:                 oklch(92% 0.02 80);
  --muted-foreground:      oklch(45% 0.02 80);

  /* Emphasis */
  --primary:               oklch(70% 0.15 50);
  --primary-foreground:    oklch(15% 0 0);
  --secondary:             oklch(90% 0.02 80);
  --secondary-foreground:  oklch(15% 0.02 80);
  --accent:                oklch(88% 0.04 50);
  --accent-foreground:     oklch(15% 0.02 80);

  /* State */
  --info:                  oklch(65% 0.14 230);
  --info-foreground:       oklch(98% 0 0);
  --success:               oklch(65% 0.16 145);
  --success-foreground:    oklch(98% 0 0);
  --warning:               oklch(78% 0.16 90);
  --warning-foreground:    oklch(15% 0 0);
  --destructive:           oklch(58% 0.20 30);
  --destructive-foreground: oklch(98% 0 0);

  /* Affordances */
  --border:                oklch(85% 0.02 80);
  --border-subtle:         oklch(90% 0.02 80);
  --input:                 oklch(94% 0.02 80);
  --ring:                  oklch(70% 0.15 50);

  /* Inverse */
  --invert:                oklch(20% 0.02 80);
  --invert-foreground:     oklch(95% 0.02 80);

  /* Theme-specific shape & motion */
  --radius:                10px;                      /* rounder for Cozy */
  --shadow-widget:         0 4px 12px oklch(40% 0.05 80 / 0.1);

  --font-display:          'Bricolage Grotesque', sans-serif;
  --font-body:             'Geist', sans-serif;
  --motion-feel:           'gentle';                  /* widgets can branch on this */
}
```

A theme is fully self-contained. To add a fifth theme, a contributor writes one CSS file populating the same variable schema and registers a new `ThemeId` value in the theme registry (a static `Record<ThemeId, ThemeMeta>` in code for v1; a workspace-backed registry in v2+ for custom themes). No JavaScript changes required for the visual identity.

### 8.2 Composition at render time

The overlay root element carries data attributes that select the active theme and density:

```html
<div data-theme="cozy" data-density="normal" style="--accent: <override or empty>;">
  <!-- widgets -->
</div>
```

Widgets style themselves using only the variables declared by the theme. Hard-coded colors in widget CSS are prohibited; lint catches them.

### 8.3 Switching themes

The editor dispatches `theming.theme.set` → event → evolve sets `Scene.themeId`. The overlay's DOM `data-theme` updates → CSS variables re-cascade → all widgets repaint within one frame. No JS re-render of widgets is required for a theme swap; this is the architectural payoff of pure-CSS theming.

---

## 9. Event bus & interactivity

### 9.1 Event taxonomy

A bounded vocabulary of event kinds, namespaced by source category:

```
chat.message              { user, text, badges, color, timestamp }
chat.command              { user, command, args, role }          // emitted after pattern match
chat.cleared              { ... }
alert.follow              { user, timestamp }
alert.subscribe           { user, tier, months, message? }
alert.cheer               { user, bits, message? }
alert.raid                { fromChannel, viewers }
alert.donation            { user, amount, currency, message? }
goal.increment            { goalKind, by, total }
media.track-changed       { title, artist, artUrl?, source }
stream.info-changed       { title, game, viewerCount, uptimeSec }
scene.changed             { sceneId }
system.connected          { source }                              // chat source / event source / obs / etc
system.disconnected       { source, reason }
overlay.test-fire         { eventKind, payload }                 // editor-fired test event
```

This is the v1 vocabulary. New event kinds require a documented justification and matching widget subscription.

### 9.2 Bindings: chat → events

A chat command binding is a tuple `{ pattern, role, event, payload }`:

- The Bindings context maintains the active binding registry.
- When a `chat.message` event flows through the bus, the bindings layer matches against each binding's pattern and role gate.
- A match publishes the bound `event` with merged `payload` (binding payload + match captures).
- Widgets subscribed to that event react.

Example:

```typescript
{
  pattern: '!hype',
  patternKind: 'literal',
  role: 'any',
  event: 'overlay.effect-fire',
  payload: { effect: 'confetti', duration: 1500 }
}
```

Any chat message starting with `!hype` from any user publishes the effect-fire event with a confetti payload. The alerts widget (or a future effects widget) reacts.

### 9.3 Test-fire from editor

The editor exposes a "Fire test event" menu in the toolbar. Selecting an event kind dispatches `overlay.test-fire` through the bus with a synthetic payload. This drives end-to-end verification without depending on chat sources.

---

## 10. OBS integration

### 10.1 obs-websocket v5 summary

obs-websocket is a WebSocket protocol implemented by OBS Studio 28+ natively (no plugin install required, though the user must enable it in OBS settings). The server defaults to `ws://localhost:4455` with optional password authentication.

Authentication uses a SHA-256 challenge-response: the server sends a salt and challenge string; the client computes `base64(sha256(base64(sha256(password + salt)) + challenge))` and sends it as the `authentication` field in an `Identify` opcode.

### 10.2 Connection lifecycle (typestate)

```
Disconnected --[connect command]--> Connecting --[hello]--> Connecting
              [retry-connect command from Failed]                            |
                                                                              v
Failed <--[any error]------------- (anywhere)                                Connected
                                                                              |
                                                                              v
                                  (user disconnect or socket close)         Disconnected
```

Operations gated by state:

| Operation | Required state |
|---|---|
| `connect(url, password)` | Disconnected | Failed |
| `disconnect()` | Connecting | Connected |
| `listScenes()` | Connected |
| `createBrowserSource(...)` | Connected |
| `setActiveScene(name)` | Connected |
| `onSceneChanged(handler)` | Connected (subscription removed when disconnected) |

The TypeScript representation uses a discriminated union; helper functions extract the typed subset (e.g., `function whenConnected<T>(c: ObsConnection, f: (s: Extract<ObsConnection, { kind: 'connected' }>) => T): Result<T, NotConnected>`).

### 10.3 Browser source creation

When the streamer clicks "Use in OBS" with a connected obs-websocket:

1. Editor calls `ObsControlPort.listScenes()` to fetch current OBS scenes.
2. User selects target OBS scene (defaulting to active one).
3. Editor calls `createBrowserSource({ sceneName, sourceName: 'Nexus Overlay', url: <built overlay URL>, width: <from layout aspect>, height: <from layout aspect> })`.
4. URL format: `https://<host>/overlay?layout=<layoutId>` (or `/overlay#layout=<id>` for hash-routed deploys without server config).
5. On success, the editor records that the OBS browser source exists (in `Preferences.obs.browserSourceCreated: { sceneName, sourceName }`) to avoid re-creating on subsequent clicks.

### 10.4 Bidirectional scene mapping

When `Layout.obsSceneMap` is populated:

- **OBS → Nexus:** when `ObsControlPort.onSceneChanged` fires, find the Nexus scene whose `obsSceneMap` entry matches and dispatch `composition.scene.activate`.
- **Nexus → OBS:** when `composition.scene.activate` fires, look up the OBS scene name in the map and call `ObsControlPort.setActiveScene`. Suppress the bounce-back by tracking a "last set from us" marker.

### 10.5 Fallback flow (no obs-ws)

When `ObsConnection.kind === 'disconnected'` after auto-probe failure, the "Use in OBS" button opens a modal with:
- The overlay URL with a copy button.
- A 15-second animated GIF showing the OBS sequence (Sources → + → Browser → paste URL → set dimensions).
- The canvas dimensions in a second copy button.
- A "Try connecting to OBS again" link that re-runs the probe.

---

## 11. Editor UX

### 11.1 Layout

The editor chrome is a dark-first professional-creative-tool shell (see [ADR-0003](../../decisions/0003-dark-first-pro-creative-tool-editor-aesthetic.md)), drawing on Zed, Graphite, Affinity, and DaVinci Resolve. Regions:

- **Title bar (top, full width, ~40px):** app mark, Live/Draft chip, undo/redo, "Use in OBS".
- **Left tool rail (~48px):** vertical icon rail of tools/modes (Affinity/Graphite convention), distinct from the widget palette.
- **Dockable studio panels (left and right, default ~280px, collapsible):** Affinity-style "studios". The widget palette is the default left studio; the inspector (selected-widget props from `propSchema`, or scene-level controls when nothing is selected) is the default right studio; the layers list is a studio. Panels collapse to their header and present a drag handle. Full drag-docking and saved custom layouts are a later iteration; the frames are dockable-shaped from the start.
- **Center canvas:** virtual-pixel surface at `Layout.aspect` (1920×1080 or 1080×1920), CSS-scaled to fit available space with letterboxing. Pannable but the default zoom is "fit to viewport."
- **Bottom scene strip (~88px):** Resolve-style page cards switching the active scene (Live / Starting Soon / BRB / Ending), the visual home of `composition.scene.activate`. Replaces the earlier top-toolbar scene tabs.
- **Command palette (overlay):** opens on Cmd/Ctrl-K (the Zed pattern), a keyboard-first fuzzy launcher for editor actions.

The four signature patterns map to their inspirations: command palette (Zed), tool rail + dockable studios (Affinity/Graphite), scene strip (DaVinci Resolve pages).

**Implementation note:** §3.1's FSD tree is written in illustrative React `.tsx` vocabulary. The real project is SvelteKit: editor chrome lives under `packages/app/src/routes/edit/` (the page composition) plus `packages/app/src/lib/shared/ui/` (generic primitives: `IconButton`, `StudioPanel`, `CommandPalette`). The `.tsx` paths in §3.1 indicate layer placement, not literal filenames.

### 11.2 Mode toggle behavior

- **Live mode (default):** every dispatch immediately persists (server-side, via the `Persistence` adapter) and pushes to all subscribed qubit clients (the overlay route receives the update). The toolbar shows a small `LIVE` badge with a connection-status dot indicating whether the overlay is currently subscribed (the server tracks subscriber count per workspace).
- **Draft mode:** dispatches mutate a draft branch; persistence still saves (so refresh doesn't lose work) but broadcast does not push to the overlay. A persistent banner shows `DRAFT, changes paused`. Two actions: "Publish" (commits the draft as the new baseline, fires events on the broadcast bus) and "Discard" (rolls back to the baseline).
- The toggle is a single switch; clicking it dispatches `workspace.mode.set`. The Decider handles the branch creation when entering draft and the merge or revert when leaving.

### 11.3 Snapping and guides

While dragging a widget on the canvas:

- Distance to canvas edges and to every other widget's bounding box is computed.
- When within 8 virtual-pixels of an alignment (edge, center, midpoint), the widget snaps and a faint guide line appears extending across the canvas.
- Holding `Alt` (or `Option`) during drag disables snapping for fine positioning.

The snapping algorithm runs on each pointer move; it is O(n) over widgets, fine for typical scene widget counts (n ≤ 20).

### 11.4 Inline editing

When a widget is selected and has `inlineEdit: { propName: 'text' }`, double-clicking the widget enters edit mode on that prop:

- A contenteditable surface appears in place.
- ESC cancels; Enter / blur commits via `composition.widget.update-props`.

For `inlineEdit: { propName: 'color' }`, double-clicking opens a popover color picker anchored to the widget.

Multi-prop widgets define inline edit per-prop; only the matching prop edits inline. Everything else uses the inspector.

### 11.5 Layout library

Layouts list at the toolbar's left edge: dropdown showing layout name + thumbnail. Each entry has rename, duplicate, archive, set-as-active actions. Up to ~50 layouts before performance considerations; v1 has no hard cap.

Archived layouts hide from the dropdown but persist in storage. A "Show archived" toggle exposes them.

### 11.6 Keyboard shortcuts (v1)

| Action | Shortcut |
|---|---|
| Undo | `⌘Z` / `Ctrl+Z` |
| Redo | `⌘⇧Z` / `Ctrl+Y` |
| Save (no-op, just visible reassurance) | `⌘S` / `Ctrl+S` |
| Delete selected widget | `Delete` or `Backspace` |
| Duplicate selected widget | `⌘D` / `Ctrl+D` |
| Nudge selected (1px) | Arrow keys |
| Nudge selected (10px) | `Shift` + Arrow keys |
| Deselect, or close an open drawer or the command palette | `Esc` |
| Open command palette | `⌘K` / `Ctrl+K` |
| Switch scene N (N in 1–9) | Number key `1`–`9` (binds to the Nth scene in display order; no-op past the layout's scene count) |
| Toggle Live/Draft | `⌘L` / `Ctrl+L` |
| Open Layout switcher | `⌘O` / `Ctrl+O` |
| Open Use in OBS modal | `⌘E` / `Ctrl+E` |

All shortcut-driven actions are instant (no animation), per the editor motion budget.

### 11.7 Responsive behavior

The editor is desktop-first but degrades gracefully to a ~640px floor (best-effort to ~360px), per [ADR-0004](../../decisions/0004-responsive-editor-shell.md). Layout reflow is owned entirely by CSS (viewport `@media` for the shell, `@container` for panel internals), so a prerendered page is correct at any width before JavaScript runs; JavaScript adds only the interactive off-canvas drawer open and close.

Four viewport zones:

| Zone | Width | Left widgets panel | Right inspector |
|---|---|---|---|
| Wide | >= 1280px | docked | docked |
| Medium | 960 to 1279px | docked | off-canvas drawer |
| Narrow | 640 to 959px | off-canvas drawer | off-canvas drawer |
| Best-effort | < 640px | drawer | drawer (tool rail horizontalizes, canvas full-bleed, scene strip thins) |

The inspector sheds first (it is reference; the widget palette is the working surface). A panel that has gone off-canvas is reached from a title-bar toggle that opens it as a Drawer: slide-in over a scrim, focus-trapped, dismissed by Escape, the scrim, or a close button, with the background made `inert` while open, and only one drawer open at a time. When the viewport widens back to a zone where a panel re-docks, its drawer closes so nothing is orphaned.

Panel internals adapt to the panel's own width via `@container`, independent of the viewport: below 260px the body padding tightens; below 220px the inspector's property rows stack label-over-value. Touch input (coarse pointer) expands tap targets to 44px via transparent overlays so the dense pro-tool visuals are unchanged, and hover affordances are gated behind `@media (hover: hover)`. All motion honors `prefers-reduced-motion`.

---

## 12. Design tokens

### 12.0 Two token universes

> **Overlay-token mechanism superseded by [ADR-0007](../../decisions/0007-pure-data-driven-themes.md)** (see also the §8 banner). The two-universe model and the shared vocabulary below remain canonical, but the overlay theme tokens no longer live in `themes/<name>.css` files hot-swapped via `[data-theme]` (the "Overlay-runtime theme tokens" row, the alias-tier `themes/*.css` source, and the "Editor universe vs theme universe" note further down). Every theme is registry **data** the relay seeds (built-ins) or the user authors, applied as **inline CSS variables** by `resolveThemeStyle`. The resolver always emits all overlay tokens, which is what keeps the overlay from inheriting the editor universe's identically-named `:root` tokens (there are no per-route stylesheet boundaries between them at runtime).

Nexus has **two distinct token universes**, both CSS-custom-property-driven but with different scope and audience:

| Universe | File location | Used by | Lifecycle |
|---|---|---|---|
| **Editor tokens** | `shared/styles/tokens.css` | `ui/editor/`, `ui/landing/` | Dark-first, light secondary. Pro-creative-tool aesthetic (Zed/Graphite/Affinity/Resolve) per [ADR-0003](../../decisions/0003-dark-first-pro-creative-tool-editor-aesthetic.md). Never changes per theme. |
| **Overlay-runtime theme tokens** | `themes/<name>.css` (one per theme) | `ui/overlay/` only | One complete token override per theme (Cozy, Cyber, Editorial, Sticker). Hot-swapped via `[data-theme]` attribute. |

The two universes use the same naming vocabulary (so a widget consuming `--foreground` doesn't care which universe it's running in) but maintain separate values. The editor never adopts a theme's aesthetic; the overlay never falls back to editor tokens. This separation is enforced by directory boundaries and lint rules, `ui/overlay/**` may not import `shared/styles/tokens.css` and vice versa.

The token vocabulary below describes the **shared schema** (the variables both universes define). Each theme file populates the same variable names with its own values.

#### Three-tier token model (Fluent 2 framework)

Within each universe, tokens are organized in three tiers (matching Microsoft Fluent 2's terminology):

| Tier | What it is | Where it lives in Nexus |
|---|---|---|
| **Global** | Raw values (specific OKLCH colors, specific px sizes, specific cubic-beziers) | Embedded directly in the CSS files below as the right-hand side of `--token-name: VALUE;` declarations |
| **Alias** | Semantic role names that point to global values (`--card-foreground`, `--primary`, `--ease-emphasized`) | `shared/styles/tokens.css` (editor) and `themes/*.css` (overlay) |
| **Component** | Per-component refinements derived from alias tokens (e.g., a tooltip's specific arrow-shadow) | Per-component CSS Modules (`ui/**/X.module.css`), referencing alias tokens via `var(--...)` |

The Alias tier is what this spec documents. Component-tier overrides are an implementation detail; they MUST reference alias tokens, never raw global values. (Lint rule: `var\(--[\w-]+\)` allowed; literal `oklch(`, `#`, or `rgb(` in component CSS is a lint error.)

### 12.1 Typography (Butterick-grounded)

```css
:root {
  --font-sans:  "Geist", system-ui, sans-serif;
  --font-serif: "Instrument Serif", Georgia, serif;
  --font-mono:  "Geist Mono", "JetBrains Mono", ui-monospace, monospace;

  --text-xs: 11px;
  --text-sm: 13px;
  --text-base: 15px;
  --text-lg: 18px;
  --text-xl: 22px;
  --text-2xl: 28px;

  --leading-tight:   1.20;
  --leading-base:    1.40;
  --leading-relaxed: 1.55;

  --measure-prose: 64ch;
  --measure-ui:    36ch;
}
```

Rules: body 15px / line-height 1.4, max line-length 64ch for prose, curly quotes and proper ellipsis at source level, tabular numerals on coordinate readouts, no bold-italic combos.

**Role mapping** (which size token to use for which purpose). Inspired by M3's display/headline/title/body/label taxonomy and Fluent 2's caption/body/subtitle/title ramp, expressed as a usage table for our size-named tokens:

| Token | M3 / Fluent role equivalent | Editor uses |
|---|---|---|
| `--text-xs` (11px) | Caption 2 / Label Small | Inspector field labels, layer-row names, hotkey hints, footer chrome |
| `--text-sm` (13px) | Caption 1 / Label Medium / Body Small | Inspector helper text, tooltip body, palette item names |
| `--text-base` (15px) | Body 1 / Body Medium | Default body text everywhere, panel content, modal copy, dialog prose |
| `--text-lg` (18px) | Subtitle 1 / Title Small | Section headings (panel titles, modal titles) |
| `--text-xl` (22px) | Title 2 / Title Medium | Page headings (the editor's only top-level title) |
| `--text-2xl` (28px) | Title 1 / Title Large / Headline Medium | Landing page hero only, never in editor chrome |

If a future contributor wants role-named tokens (e.g., `--text-body-medium` aliased to `--text-base`), nothing prevents adding them, but the size names remain canonical to match the Tailwind/shadcn ecosystem we inherit from.

### 12.2 Color (dark-first, shadcn/ui vocabulary + ReUI state extensions, OKLCH)

**Dark-first** per [ADR-0003](../../decisions/0003-dark-first-pro-creative-tool-editor-aesthetic.md): dark values are the baseline in `:root` (re-asserted under `.dark` via the selector group `:root, .dark`), `app.html` ships `class="dark"` so the page paints dark with zero JavaScript, and `.light` is the secondary opt-out. The naming still follows shadcn/ui; ReUI's state additions (`info`, `success`, `warning`, `invert`) fill the gaps; every background has a paired `-foreground`.

The **canonical values** live in `packages/app/src/lib/shared/styles/tokens.css`. The defining change from the prior flat palette is an explicit **elevation ladder**: app background is darkest, each surface lifts above it. The palette is grounded in Zed's One Dark, translated to OKLCH at hue 264 with a faint cool chroma (0.008 to 0.015) so the near-blacks read as graphite rather than flat grey; structural depth draws from Graphite, Affinity, and DaVinci Resolve.

**Surface roles** (dark baseline, the elevation ladder):

```css
:root,
.dark {
  --background:         oklch(20% 0.012 264);   /* app shell, darkest */
  --foreground:         oklch(92% 0.01 264);
  --card:               oklch(23.5% 0.012 264); /* studio panel body, lifts above bg */
  --card-foreground:    oklch(92% 0.01 264);
  --popover:            oklch(27% 0.013 264);   /* command palette, floats highest */
  --popover-foreground: oklch(93% 0.01 264);
  --muted:              oklch(26% 0.012 264);
  --muted-foreground:   oklch(68% 0.012 264);
}
```

The emphasis, state, affordance, and inverse roles below document the shadcn/ReUI **vocabulary and pairing convention**. Their literal values and inline comments are the pre-dark-first light originals, kept for reference only; the **canonical dark-first values live in `tokens.css`** (e.g. `--primary: oklch(62% 0.16 255)`, `--ring: oklch(72% 0.13 245)` deliberately brighter than primary for dark-surface contrast, `--invert` is a light chip on dark). Read those blocks for the variable names, not the numbers.

**Emphasis roles** (interactive emphasis levels):

```css
:root {
  /* Primary, high-emphasis actions, brand */
  --primary:                     oklch(55% 0.18 250);
  --primary-foreground:          oklch(98% 0 0);

  /* Secondary, lower-emphasis actions */
  --secondary:                   oklch(94% 0 0);
  --secondary-foreground:        oklch(18% 0 0);

  /* Accent, hover/focus highlights, selection */
  --accent:                      oklch(94% 0 0);
  --accent-foreground:           oklch(18% 0 0);
}
```

**Semantic state roles** (per ReUI; shadcn ships only `destructive`):

```css
:root {
  --info:                        oklch(60% 0.15 230);
  --info-foreground:             oklch(98% 0 0);

  --success:                     oklch(60% 0.18 145);
  --success-foreground:          oklch(98% 0 0);

  --warning:                     oklch(75% 0.18 90);
  --warning-foreground:          oklch(18% 0 0);

  --destructive:                 oklch(55% 0.22 30);
  --destructive-foreground:      oklch(98% 0 0);
}
```

**Affordance roles** (borders, inputs, focus):

```css
:root {
  --border:                      oklch(90% 0 0);   /* strong dividers, panel boundaries */
  --border-subtle:               oklch(94% 0 0);   /* M3 outline-variant, inspector field separators, list item dividers */
  --input:                       oklch(96% 0 0);
  --ring:                        oklch(55% 0.18 250);   /* focus ring; usually matches primary */
}
```

**Inverse roles** (per M3 inverse-surface and ReUI invert), surfaces that contrast against the main app shell:

```css
:root {
  --invert:                      oklch(18% 0 0);   /* dark in light theme; flips in .dark */
  --invert-foreground:           oklch(95% 0 0);
}
```

Used by toasts, the command palette overlay (if added later), the "LIVE" badge in the toolbar, and any other surface that needs to contrast with `--background`.

**Light secondary** (`.light`): ported from the original light palette plus a faint cool chroma at hue 264 so toggling dark <-> light is hue-stable. The `.light` class on `<html>` is the opt-out from the dark default. Full values in `tokens.css`; representative surfaces: `--background: oklch(99% 0.004 264)`, `--card: oklch(97% 0.004 264)`, `--popover: oklch(99.5% 0.003 264)`, with `--foreground: oklch(20% 0.01 264)`. Chrome surfaces and shadows get light variants (lighter surfaces, much softer shadows).

#### Chrome structure tokens (pro-tool shell, additive)

The pro-tool shell needs structural tokens the flat shadcn set lacks. Additive, no renames:

- **Layout metrics:** `--titlebar-height` (40px), `--toolrail-width` (48px), `--scenestrip-height` (88px), `--panel-header-height` (32px), `--panel-min-width` (200px), `--panel-default-width` (280px).
- **Chrome surfaces:** `--titlebar`, `--panel-header`, `--toolrail`, `--divider` (the inset 1px seams between chrome regions, darker than `--border`).
- **Elevation shadows:** `--shadow-panel` (docked panel lift), `--shadow-popover` (command palette, dropdowns), `--shadow-dragging` (a panel mid-drag, visual only this cycle).
- **Shared focus recipe:** `--focus-ring` (a 2px background gap + 2px `--ring` outline) so every focusable shares one recipe.

**Editor universe vs theme universe.** The editor tokens above live in `shared/styles/tokens.css` and define the editor's single dark-first pro-tool design (per [ADR-0003](../../decisions/0003-dark-first-pro-creative-tool-editor-aesthetic.md)). Each overlay theme in `themes/<name>.css` defines the **same variable names** with its own values, Cozy uses warm peach hues across `--background`/`--card`/`--primary`, Cyber uses cold cyan-neon, Editorial uses cool greyscale with one strong accent, Sticker uses high-saturation primary-school colors. A widget written against `--card-foreground` works identically in either universe.

**Pair philosophy** (per Park UI). Each theme commits to **one accent hue + one neutral scale**. Documented pairings:

| Theme | Accent | Neutral |
|---|---|---|
| Cozy | warm peach `oklch(70% 0.15 50)` | warm grey `oklch(L 0.01 80)` |
| Cyber | cyan-neon `oklch(75% 0.20 200)` | cool grey `oklch(L 0.02 250)` |
| Editorial | electric blue `oklch(55% 0.22 260)` | pure grey `oklch(L 0 0)` |
| Sticker | hot pink `oklch(70% 0.25 0)` | warm cream `oklch(L 0.03 90)` |

Per-theme accent + neutral choice is the entire visual identity. No theme mixes multiple accent hues.

### 12.3 Motion (Emil-grounded + M3 emphasized curve)

```css
:root {
  --dur-instant: 0ms;
  --dur-fast:    120ms;
  --dur-normal:  180ms;
  --dur-slow:    240ms;

  --ease-out:         cubic-bezier(0.16, 1, 0.3, 1);   /* default, sharp start, gentle settle */
  --ease-in:          cubic-bezier(0.7, 0, 0.84, 0);
  --ease-emphasized:  cubic-bezier(0.2, 0, 0, 1);      /* M3's emphasized, important transitions only */
  --ease-spring:      cubic-bezier(0.5, 1.5, 0.5, 1);  /* decorative; overlay-runtime only */

  --press-scale: 0.97;
  --enter-scale-from: 0.95;
}
```

**`--ease-emphasized`** is reserved for transitions that should feel *important*, modal opens, "Connect to OBS" success animation, mode-toggle confirmation. Per M3, the emphasized curve emphasizes important transitions while standard `--ease-out` handles routine motion. Use sparingly; over-applying defeats the purpose.

Editor: ≤ 300 ms, `ease-out` default, never `ease-in-out`, button press scale 0.97, modal enter from scale 0.95 never 0, drag and keyboard actions instant. Overlay-runtime budget is separate, up to 800 ms for alerts and scene transitions, with the spring easing available for decorative motion.

### 12.4 Spacing, radii, stroke

Spacing follows an 8px base rhythm; radii derive from a single base via `calc()` (per shadcn) so a theme can re-tune all radii by changing one value; stroke widths are tokenized (per Fluent 2) so borders, snapping guides, focus rings, and icons share a coordinated scale.

```css
:root {
  --space-1: 4px;
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  --space-5: 24px;
  --space-6: 32px;
  --space-8: 48px;

  /* Radii, single base; sm/md/lg/xl derived. none and circular are explicit endpoints (per Fluent 2) */
  --radius-none:     0;
  --radius:          6px;
  --radius-sm:       calc(var(--radius) * 0.6);    /* 3.6px @ default */
  --radius-md:       calc(var(--radius) * 0.8);    /* 4.8px @ default */
  --radius-lg:       var(--radius);                /* 6px @ default */
  --radius-xl:       calc(var(--radius) * 1.6);    /* 9.6px @ default */
  --radius-canvas:   14px;                          /* signature shape, explicit, doesn't scale */
  --radius-circular: 9999px;                        /* for avatars, badges, pill shapes */

  /* Stroke widths (per Fluent 2 strokeWidthThin/Thick) */
  --stroke-thin:     1px;     /* panel borders, dividers */
  --stroke-thick:    1.5px;   /* icon strokes (matches Lucide default), focus rings */
  --stroke-thicker:  2px;     /* selection outlines on canvas, snap guides */
}
```

A theme tunes the radius identity by setting `--radius` to a new value (Sticker theme could use `12px` for chunkier rounding; Editorial theme could use `2px` for sharp publication shapes). The derived sm/md/lg/xl scale proportionally; `--radius-none` and `--radius-circular` stay constant.

### 12.5 Iconography

Lucide. Stroke width references `--stroke-thick` (1.5px at default). Sizes: 16px in inspector, 18px in toolbar, 20px in canvas overlays. No icon-set mixing. No emoji in editor chrome. Icons inherit color via `currentColor`; never hard-coded fills.

### 12.6 Responsive, touch, and breakpoint tokens

Added with the responsive shell ([ADR-0004](../../decisions/0004-responsive-editor-shell.md)):

- **Breakpoints** (documentation-only custom properties `--bp-wide: 1280px`, `--bp-medium: 960px`, `--bp-floor: 640px`). CSS `@media`/`@container` cannot read `var()`, so these literals are mirrored in `routes/edit/shell-state.svelte.ts` (`BREAKPOINTS`, for `matchMedia`) and hard-coded in each query condition. The token block in `tokens.css` is the canonical human reference; the three copies are kept in sync by hand.
- **Drawer + touch:** `--drawer-width: min(320px, 86vw)` (an off-canvas drawer never exceeds the viewport at 360px) and `--touch-target-min: 44px` (reached on coarse pointers via a transparent `::after` overlay, so the dense 28 to 32px visuals are unchanged).
- **Fluid display type:** `--text-xl` and `--text-2xl` are `clamp()`-bounded (scaling between the 360px and 1280px viewports); the UI sizes (`--text-xs/sm/base/lg`) stay fixed so chrome text does not drift.
- **Viewport height:** the shell uses `100dvh` (dynamic viewport height), not `100vh`.
- **Reduced motion:** a global `@media (prefers-reduced-motion: reduce)` block in `reset.css` zeroes transitions and animations (0.01ms, so `transitionend` still fires), covering the drawer slide and all chrome motion. The editor budget (<= 300ms, `ease-out`) is unchanged; the drawer slide uses `--dur-slow` (240ms).

---

## 13. Persistence

> **Revised by [ADR-0005](../../decisions/0005-offline-collaborative-loro-crdt-trusted-relay.md):** persistence is a Loro binary snapshot (`doc.export(snapshot)`) written atomically to the OS app-data dir, not a hand-rolled workspace JSON file. JSON export/import is retained as a user-facing feature. Live sync is Loro's update protocol over WebSocket, not qubit subscriptions from a single authority.

### 13.1 Storage shape

Workspace state lives in the Rust server process. Persistence is abstracted behind the `Persistence` trait (see §6.3); v1 uses `FilePersistence`, v2 cloud mode uses `DatabasePersistence`. Both implementations serialize the same `Workspace` shape.

**Local mode (v1):** One JSON file per workspace, stored under the OS-conventional app-data directory:
- Linux: `$XDG_DATA_HOME/nexus/` or `~/.local/share/nexus/`
- macOS: `~/Library/Application Support/nexus/`
- Windows: `%APPDATA%\nexus\`

File naming: `workspace-<workspaceId>.json`. A small `index.json` at the data-dir root lists known workspaces with their IDs, names, last-modified timestamps, and thumbnail paths. The streamer can locate, back up, or hand-edit these files directly, a feature, not a vulnerability.

**Cloud mode (v2):** Workspaces live in Postgres tables, keyed by `(ownerId, workspaceId)`. Same JSON-serialized payload as the file format. Migration path: `nexus migrate-up` (export local workspaces, upload to cloud) is a one-time per-user import.

On load: deserialize → validate against schema → if valid, hydrate; if invalid, snapshot the corrupt payload to a recovery directory (`recovery/<timestamp>/`), log the error with full context, and fall back to the curated default workspace.

### 13.2 Migration

When a workspace file's `schemaVersion` differs from the current code's version, the Rust server applies migrations forward on load:

```rust
fn migrate(workspace: WorkspaceAnyVersion) -> Result<WorkspaceCurrent, MigrationError>
```

Each migration is a pure function from version N to version N+1. They chain. A failing migration writes the corrupt payload to the recovery directory and returns the curated default with a non-blocking toast emitted via qubit to the connected editor.

Schema versions ship with the binary; we never read an unknown version. If the workspace file's version is *higher* than the server's known version (e.g., file was edited by a newer Nexus build that downgraded), the server refuses to load it and surfaces an actionable error.

### 13.3 Export / import

The workspace JSON is the export format. "Export Layout" serializes one `Layout` (not the whole workspace) into a JSON file. "Import Layout" reads a JSON file, validates it, generates new IDs to avoid collisions, and dispatches a `workspace.layout.create` command with the imported layout's contents.

Both operations are qubit mutations on the Rust server; the SvelteKit UI triggers them via the generated client. The server handles file I/O (read from path / write to download).

### 13.4 Live sync (qubit subscriptions)

Mutations are processed by the server's Decider; produced events are applied via `evolve` to the authoritative workspace state, persisted, and *then* pushed to all subscribed qubit clients.

The editor and overlay routes both call `api.workspace.subscribe()` on mount and receive a stream of state snapshots (or event deltas; design detail TBD in implementation). When the editor mutates, the overlay's subscription receives the new state within milliseconds (WebSocket round-trip on localhost is < 10 ms).

Multiple editor tabs are no longer a sync problem, they're just multiple subscribers to the same authoritative server state. The "edit from two tabs simultaneously" case is handled by the Decider: each mutation is a discrete command, applied serially, with the resulting state pushed to all subscribers.

If a client disconnects (browser tab closed, network blip), it reconnects and re-subscribes; the server sends the current state as the first event of the new subscription. No data is lost.

---

## 14. Verification

### 14.1 Test strategy

The architecture's payoff: most logic tests do not need a browser.

| Test category | Where it runs | What it covers |
|---|---|---|
| Decider unit tests | Pure Node | Every `decide` branch; every `evolve` branch; replays from event logs |
| Port contract tests | Pure Node | Each port has a "contract test suite"; both mock and real adapters pass it |
| Adapter integration tests | `cargo test` + tokio runtime + tempdir for filesystem | `FilePersistence` exercises real filesystem in a temp directory; `ObwsAdapter` runs against a stubbed obs-ws server; mock chat/event sources need no external deps |
| UI component tests | Browser-like env | Per-widget rendering smoke tests; inspector form generation against prop schemas |
| End-to-end (E2E) | Real browser via Playwright | Editor open → drag widget → switch theme → open `/overlay` in second tab → verify identical rendering → fire test event → verify widget reacts |

### 14.2 Acceptance scenarios

Manual verification checklist for sign-off on v1:

1. **First-run**: Visit `/edit` in a fresh browser. Editor opens with curated default layout already populated. No setup dialog. Test events fire on cadence from `MockEventSource`. Verified.
2. **Edit and preview**: Drag the webcam widget; chat widget; alert area. Each move animates instantly in `/edit`. Open `/overlay` in a second tab. Both tabs show identical compositions. Make another edit in `/edit`; `/overlay` updates within 1–2 frames via the qubit subscription. Confirm via dev-tools that the WebSocket round-trip is < 20 ms.
3. **Theme swap**: Click each of Cozy/Cyber/Editorial/Sticker. Canvas updates immediately; overlay updates in lockstep.
4. **Inline edit**: Double-click a widget exposing inline-text edit (e.g., now-playing if it surfaces a title); type new text; press Enter. Updates persist.
5. **Live/Draft toggle**: Enter Draft mode. Make a change. Overlay does NOT update. Click Publish. Overlay updates. Make another change in Draft. Click Discard. Overlay state matches pre-draft baseline.
6. **OBS auto-detect**: With OBS running and obs-websocket enabled (no password), refresh the editor. The "Use in OBS" button shows a "connected" indicator. Click it. Select a target scene. Confirm. Verify the Browser Source appears in OBS with the correct URL and dimensions.
7. **OBS password flow**: Configure obs-websocket with a password. Repeat #6. Editor prompts for password. Connection succeeds. Source created.
8. **OBS fallback**: Close OBS or disable obs-websocket. Refresh editor. "Use in OBS" opens the manual modal with URL, GIF, dimensions copy button.
9. **Bidirectional scene mapping**: Configure each Nexus scene to map to an OBS scene by name. Switch OBS scene via OBS interface. Nexus overlay switches to the mapped scene. Switch in editor. OBS switches.
10. **Chat-command binding**: Create a binding `!hype` → `overlay.effect-fire { effect: 'confetti' }`. With `MockChatSource` running, inject a `!hype` message. Confetti fires in the overlay.
11. **Persistence**: Close the editor tab. Reopen. State restored exactly.
12. **Layout library**: Create a second layout. Switch between them. Each has its own scenes, themes, widgets. Each maps to a distinct overlay URL.
13. **Aspect toggle**: Toggle 16:9 ↔ 9:16 on a layout. Canvas reflows. Widget positions clamp to new bounds where needed.
14. **Undo/redo**: Make ten edits. Undo all the way back. Redo to the front. State should match exactly at each step.
15. **Schema migration**: Manually edit a workspace JSON file in the app-data directory to set `"schemaVersion": 0` (a pretend old version). Restart the Rust server. Server either runs the migration successfully or writes the corrupt payload to the recovery directory and emits a toast via qubit, no crash.

### 14.3 Performance budgets

| Surface | Budget |
|---|---|
| Editor initial paint (cold cache) | < 1.5 s on a fast laptop |
| Overlay initial paint | < 800 ms |
| Drag → render latency | < 16 ms (1 frame) |
| qubit mutation → server persist → subscriber update | < 32 ms localhost (2 frames) |
| Theme swap | < 16 ms (single CSS-cascade frame) |
| Workspace load+validate from filesystem | < 50 ms |
| Rust server cold start (binary launch → listening on port) | < 500 ms |

---

## 15. Project rules (cheatsheet)

Pinned where every contributor will see them, README, CONTRIBUTING, and as ESLint/dependency-cruiser rules where mechanically enforceable.

1. **Shell calls core. Core does not call shell.**
2. **Core has no clock, no randomness, no I/O, no DOM, no storage.**
3. **State mutates only through Command → Decide → Events → Evolve.**
4. **decide is pure and total. evolve is mechanical.**
5. **Fallible operations at port boundaries return `Result<T, E>`. No throws across ports.**
6. **Aggregate lifecycles with 3+ gated states are discriminated unions: `Workspace.mode`, `ObsConnection`, `Layout`.**
7. **Bounded contexts publish events; they do not import each other.**
8. **All types are `readonly`. No DTOs.**
9. **Editor animations ≤ 300 ms, ease-out default. Overlay-runtime gets a separate, more expressive budget.**

---

## 16. Implementation stack (locked)

> **Revised by [ADR-0005](../../decisions/0005-offline-collaborative-loro-crdt-trusted-relay.md):** add `loro` (Rust) / `loro-crdt` (JS) as the CRDT document substrate; qubit is retained for the control plane only (not the document data-plane); `@tanstack/svelte-query` is dropped for document state (the local Loro replica is the reactive store). `eslint-plugin-boundaries` is referenced below but is not yet installed.

The architectural decisions in §1–15 are framework-agnostic. This section names the concrete technology stack the v1 implementation is built on. The stack is locked per [ADR-0002](../../decisions/0002-local-or-cloud-rust-core-with-qubit-rpc.md) (local-or-cloud Rust core with qubit RPC) and operates inside the FSD organization from §3.1.

### 16.1 Core (Rust)

| Concern | Choice | Notes |
|---|---|---|
| Language | Rust 1.84+ (2024 edition) | Idiomatic sum types (`enum`), `Result<T, E>` native, exhaustive `match`, ownership enforces immutability discipline by default. |
| Workspace structure | Cargo workspace with two crates: `nexus-core` (pure) + `nexus-server` (adapters + HTTP/RPC) | `nexus-core` has zero dependencies on `nexus-server`; verified via `cargo tree`. Matches functional-core / imperative-shell. |
| RPC framework | **qubit** | Rust↔TS RPC over JSON-RPC 2.0. TypeScript clients generated from Rust handlers via `ts-rs`. Built on `jsonrpsee`. Supports queries, mutations, subscriptions. |
| HTTP server | `axum` (under qubit) | Tokio-based async, mature, composable. |
| Async runtime | `tokio` | Standard choice; required by `axum` and `jsonrpsee`. |
| Serialization | `serde` + `serde_json` | Used by qubit for the wire format and for filesystem persistence. |
| Type sharing | `ts-rs` (via qubit) + `tsify` if needed | Rust types compiled to TypeScript declarations at build time. |
| Schema validation | `validator` crate (Rust side) | Schema invariants enforced in `nexus-core::decide`. |
| Error types (core, typed) | `thiserror`, enums with `#[derive(thiserror::Error)]` | Used in `nexus-core` where callers may need to match on error variants. Serialize to RFC 9457 Problem Details at the HTTP boundary (per [ADR-0001](../../decisions/0001-use-rfc-9457-problem-details-for-http-errors.md)). |
| Error types (glue, untyped) | `anyhow`, `anyhow::Result<T>` with `.context()` | Used in `nexus-server` for config loading, server startup, request-handler glue. `?` propagates through diverse error sources; `.context("loading workspace from disk")` annotates without forcing typed enums. Converts to typed errors at module boundaries. |
| obs-websocket | `obws` crate (Rust v5 client) | Replaces JS bridge; Rust talks to OBS directly. |
| Persistence (local) | `FilePersistence` adapter, JSON files in OS app-data directory (via `dirs` crate) | Workspace serialized as JSON; one file per workspace. |
| Persistence (cloud, v2) | `DatabasePersistence` adapter, Postgres via `sqlx` (deferred) | Same `Persistence` trait, swapped at composition root. |
| Auth (local) | `NoOpAuth` (single tenant) | Local binary belongs to its launcher. |
| Auth (cloud, v2) | `OAuthAuth` or session-based (deferred) | New port + adapter; handlers already accept `Auth` context. |
| ID generation | `uuid` crate (UUIDv4) | Generated server-side; injected via `RandomPort` for tests. |
| Date/time | `chrono` | Timestamps on events; durations for "uptime"; serialization via serde. |
| Configuration | `figment` or `envy` for env-var-driven config | 12-factor: `NEXUS_MODE`, `NEXUS_BIND_ADDR`, `NEXUS_DATA_DIR`, `NEXUS_DATABASE_URL` (cloud only). |
| CLI | `clap` for the binary's CLI flags | `nexus serve`, `nexus serve --port 9876`, `nexus serve --mode cloud`. |
| Static asset serving | `tower-http::services::ServeDir` | Embeds the SvelteKit build output into the binary (via `include_dir`) and serves it under `/` and `/edit` and `/overlay`. |
| Logging | `tracing` + `tracing-subscriber` | Structured logs; ready for OpenTelemetry export in cloud mode. |
| Testing (unit) | `cargo test` for pure-core tests; `tokio-test` for async | Pure-core test suite has zero adapter dependencies. |
| Linter | `clippy` (with project ruleset) | Enforces idiomatic Rust + project-specific lints. |
| Formatter | `rustfmt` (default config) | Auto-applied on save. |
| Build optimization | LTO + `wasm-opt` not applicable; `cargo build --release` with `lto = "fat"`, `codegen-units = 1`, `strip = true` for distribution binaries. | Smaller binaries; ~5–10 MB compressed expected. |

### 16.2 UI (SvelteKit)

| Concern | Choice | Notes |
|---|---|---|
| UI framework | **SvelteKit 2.x + Svelte 5 (runes)** | Modern Svelte with `$state`, `$derived`, `$effect`. |
| Deploy adapter | `@sveltejs/adapter-static` | Builds the UI as static HTML/CSS/JS; the Rust server serves the output via `tower-http`. |
| Routing | SvelteKit filesystem routing (`src/routes/`) | `/+page.svelte` (landing), `/edit/+page.svelte`, `/overlay/+page.svelte`. |
| RPC client | `@qubit-rs/client` | Generated TypeScript client from Rust handlers; provides typed query/mutation/subscription methods. The transport layer. |
| Server-state cache | **`@tanstack/svelte-query` (TanStack Query v5)** | Cache, dedup, invalidation, retry, optimistic updates, devtools. Wraps qubit calls: queries via `createQuery`, mutations via `createMutation` with optimistic-update + rollback, cache invalidation via `QueryClient.invalidateQueries`. |
| Live data delivery | qubit subscriptions → write into TanStack Query cache | Single subscription handler per workspace pushes server-state updates into the cache via `qc.setQueryData(['workspace', id], newState)`. UI components read from the cache, never from the raw subscription. |
| Reactive state (UI) | Svelte 5 runes reading from TanStack Query cache | `$derived` over query results; UI re-renders when the cache changes. |
| Optimistic update pattern | TanStack Query `onMutate` snapshots cache → applies optimistic state → `onError` rolls back; `onSettled` invalidates | Drag-move feels instant: pointermove updates cache locally → mutation sent in background → on failure, cache rolls back, widget snaps to prior position. |
| Drag-drop | Hand-rolled pointer-event handlers in the editor canvas | Free-position canvas; ~150 lines for the snap-to-edge + smart-guides algorithm. Reconsider `pragmatic-drag-and-drop` if accessibility or touch edges appear. |
| Animation | Svelte built-ins (`svelte/transition`, `svelte/motion`) + GSAP if a specific overlay-runtime moment needs it | Editor stays restrained; overlay-runtime gets expressive motion. |
| Iconography | `lucide-svelte` | Direct port of Lucide for Svelte. |
| Headless UI components | `@ark-ui/svelte` (Zag.js machines) behind `shared/ui` wrappers, + `culori` for oklch conversion | Accessible Select / Dialog / NumberInput / Switch / Collapsible / Color Picker / Tooltip / ToggleGroup, styled with the token system. Editor-only; the `/overlay` bundle stays Ark-free. Per [ADR-0008](../../decisions/0008-adopt-ark-ui.md). |
| Styling | Svelte scoped `<style>` blocks + CSS custom properties (from `tokens.css`) | No CSS Modules needed, Svelte's scoping is built-in. |
| CSS reset | `modern-normalize` | ~2 KB; focused on real cross-browser issues. |
| Schema mirror | Generated from Rust via `ts-rs`; no separate TS validation library needed | Server is the validation authority; client trusts the contract. |
| Testing (unit/integration) | Vitest + `@testing-library/svelte` | For Svelte components and TS helpers. |
| Testing (E2E) | Playwright | Tests against the live Rust server binary in CI. |
| Linter (TS/Svelte) | ESLint + `eslint-plugin-svelte` + `eslint-plugin-boundaries` | The boundaries plugin enforces FSD layer rules. |
| Formatter | Prettier + `prettier-plugin-svelte` | Auto-applied on save. |
| Type checker | `svelte-check` + `tsc --noEmit` | Run in CI. |

### 16.3 Build + toolchain

| Concern | Choice | Notes |
|---|---|---|
| Node version | 22 LTS | Minimum; `package.json` engines field. |
| Package manager (UI) | **pnpm** | Lockfile: `pnpm-lock.yaml`. |
| Build orchestration | `pnpm build` chains: `pnpm svelte-kit build` → `cargo build --release` (with build script copying static assets into the binary) | Single command builds everything; outputs one binary. |
| Dev loop | `pnpm dev` runs Vite (UI) + `cargo watch -x run` (Rust server) concurrently | UI hot-reloads; Rust restarts on code change. |
| Type-gen step | `cargo run --bin generate-types` outputs `src/lib/api/generated.d.ts` | Run on Rust change; checked into git for deterministic builds. |
| CI | GitHub Actions (or equivalent) | Matrix: Rust stable + nightly; Node 22 LTS. Steps: `cargo test`, `cargo clippy`, `cargo fmt --check`, `pnpm test`, `pnpm lint`, `pnpm test:e2e`. |
| Cross-platform binaries | `cargo dist` (or manual `cross` builds) | Targets: `x86_64-pc-windows-msvc`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-unknown-linux-gnu`. |

### 16.4 Project layout (locked)

```
nexus/
├── Cargo.toml                                workspace manifest
├── package.json                              SvelteKit dependencies
├── pnpm-lock.yaml
├── svelte.config.js
├── vite.config.ts
├── tsconfig.json
│
├── crates/
│   ├── nexus-core/                           hexagonal pure core
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── workspace/                    Workspace entity + decider + evolve
│   │       ├── layout/
│   │       ├── scene/
│   │       ├── widget_instance/
│   │       ├── theme/
│   │       ├── chat_binding/
│   │       ├── obs_connection/
│   │       └── ports/                        Persistence, Auth, ObsControl, ChatSource, ...
│   │
│   └── nexus-server/                         qubit handlers + adapters + HTTP server
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs                       binary entrypoint
│           ├── handlers/                     qubit query/mutation/subscription handlers
│           ├── adapters/
│           │   ├── persistence/
│           │   │   ├── file.rs               local FilePersistence
│           │   │   └── database.rs           (v2) cloud DatabasePersistence
│           │   ├── auth/
│           │   │   ├── noop.rs               local NoOpAuth
│           │   │   └── oauth.rs              (v2) cloud OAuthAuth
│           │   ├── obs/                      obws-based ObsControl adapter
│           │   └── chat/                     mock chat source + event source
│           ├── config.rs                     env-var-driven mode selection
│           └── server.rs                     axum app, qubit router wiring
│
└── src/                                      SvelteKit application
    ├── app.html
    ├── routes/                               FSD pages (filesystem routing)
    │   ├── +layout.svelte
    │   ├── +page.svelte                      landing
    │   ├── edit/+page.svelte
    │   └── overlay/+page.svelte
    └── lib/
        ├── widgets/                          FSD widgets (overlay/ + editor/)
        ├── features/                         FSD features (drag, theming, obs-detect, ...)
        ├── entities/                         FSD entities (thin TS wrappers around qubit-generated types)
        ├── shared/
        │   ├── ui/                           Svelte primitive components
        │   ├── styles/
        │   │   ├── tokens.css                editor design tokens
        │   │   ├── normalize.css             modern-normalize
        │   │   └── themes/                   overlay theme CSS files (cozy, cyber, editorial, sticker)
        │   ├── lib/
        │   │   ├── api.ts                    qubit client setup + TanStack QueryClient instance
        │   │   ├── query-keys.ts             centralized query-key factory (shared across createQuery sites)
        │   │   └── subscription-bridge.ts    bridges qubit subscriptions → qc.setQueryData()
        │   └── config/
        └── api/
            └── generated.d.ts                qubit-generated TS types (checked into git)
```

### 16.5 What this stack does not include

- No client-side *application* state library (Zustand, Redux, MobX) for UI-only state. **TanStack Query *is* the client-side state layer** for server-state, it owns cache, mutation, optimistic updates, invalidation. Pure UI ephemera (drag in-progress, hover state, modal-open flags) lives in component-local `$state` runes.
- No client-side schema validation library (Zod, Valibot). The server validates; the generated TypeScript types are trusted by the client.
- No CSS-in-JS runtime. Svelte scoped styles + CSS custom properties cover all needs.
- No animation library in v1. Svelte built-ins are sufficient; GSAP added only if a specific overlay-runtime moment requires imperative timeline control.
- No WASM. Rust runs as a native process, not in the browser.
- No service workers, no offline-first browser caching. The Rust binary serves the SvelteKit assets; offline-without-binary doesn't apply.

### 16.6 What this stack admits later

- **Cloud mode** activates by setting `NEXUS_MODE=cloud` + database connection string + auth provider. Same binary, environment-configured.
- **Tauri packaging** wraps the Rust server + bundled assets into platform installers (auto-update, system tray integration) without architectural change.
- **Real platform integrations** (Twitch, YouTube, Kick) become additional `ChatSource` and `EventSource` adapter implementations in `nexus-server`.
- **Custom themes** (v2+) can be either filesystem-loaded (drop a CSS file into the data directory) or registry-loaded (downloaded from a future themes marketplace).

---

## 17. Glossary

- **Adapter**: a concrete implementation of a secondary port; lives in the owning feature's `api/` segment (FSD).
- **FSD (Feature-Sliced Design)**: the layered organizing methodology used by Nexus. Layers (top to bottom: `app` → `pages` → `widgets` → `features` → `entities` → `shared`) can only import from layers strictly below. Slices within a layer cannot import each other; cross-slice contracts live one layer down.
- **Slice**: a domain-based partition within an FSD layer (e.g., `features/compose-widget/`). Each slice declares its public API via `index.ts`; internal files are not importable from outside.
- **Segment**: a technical-purpose grouping within a slice (`ui/`, `api/`, `model/`, `lib/`, `config/`). `model/` segments are the pure functional core; `api/` segments are the imperative shell.
- **Bounded context**: a coherent slice of the domain with its own vocabulary; communicates with other contexts via events.
- **Command**: a request to change state, expressed as a discriminated-union variant.
- **Decider**: the four-element pure structure `{ initialState, decide, evolve, isTerminal }` per Chassaing.
- **Event**: a fact about a state change that has happened, in past tense.
- **Imperative shell**: the layer that owns I/O, time, randomness, and wraps the pure core.
- **Layout**: a complete overlay product owned by one streamer; contains scenes.
- **Port (primary / driving)**: an interface the application exposes to drivers (UI, tests, CLI).
- **Port (secondary / driven)**: an interface the application needs from infrastructure.
- **Theme**: a complete token override expressed as a CSS file declaring values for the shared token schema. v1 ships four built-in themes (Cozy, Cyber, Editorial, Sticker). v2+ may admit custom themes (community-contributed, streamer-authored) under the same schema.
- **ThemeId**: a string identifier for a theme. Built-in themes use well-known IDs; custom themes (v2+) register their own. Not a closed type union.
- **Appearance**: the editor's light/dark/system color preference, applied via the `.dark` class on `<html>`. Independent of theme, the editor never adopts an overlay theme; the overlay's color identity comes entirely from its active theme.
- **Scene**: a composition within a layout (Live / Starting Soon / BRB / Ending or user-defined).
- **Typestate**: an aggregate lifecycle expressed as distinct types with state-gated operations.
- **Widget**: a self-contained overlay element conforming to the WidgetMeta contract.
- **Widget instance**: one placement of a widget on a scene's canvas.
- **Workspace**: the root aggregate containing all of a streamer's layouts and preferences.

---

## 18. Open questions deferred to implementation

These don't block the spec; they need decisions during build-out but won't change the design.

- Exact event-log retention strategy in memory (cap, snapshot interval).
- Specific Lucide icon mapping per widget type.
- Default values for `MockEventSource` cadence (probably 12–25 s between events with randomization).
- Whether to ship Geist self-hosted or via Google Fonts (self-hosted is more reliable for OBS Browser Source which may run with limited network access).
- Whether keyboard shortcuts respect `prefers-reduced-motion` to skip even the small amount of animation present in the editor.

---

*End of spec.*
