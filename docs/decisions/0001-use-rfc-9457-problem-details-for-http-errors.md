---
status: "accepted"
date: 2026-05-13
decision-makers: project owner
consulted: 
informed: future contributors
---

# Use RFC 9457 Problem Details for HTTP error responses

## Context and Problem Statement

Nexus v1 has no HTTP API, it is a front-end-only application persisting locally and integrating with OBS via WebSocket. v2, when it lands, introduces a backend: persistence-as-a-service, multi-platform integrations (Twitch / YouTube / Kick), webhook receivers, possibly billing. Every backend operation can fail in multiple ways: bad request, missing resource, auth failure, rate-limited, internal error, upstream service down, schema migration in progress.

If we don't decide the error-response format now, we will:
- Invent ad-hoc shapes per endpoint as we go, ending up with inconsistent error structures across the API.
- Lose the opportunity to use battle-tested tooling (clients, validators, observability integrations) that parse a standard format.
- Spend design time on a problem the IETF already solved.

The decision is purely about *the on-the-wire shape* of HTTP error responses. It does not commit us to a specific HTTP API style (REST, RPC, GraphQL); the format is compatible with any HTTP API and is the de facto standard for backends that return structured errors. It is also forward-looking, v2 hasn't started, but the cost of choosing it now is zero, and the cost of *not* choosing it (debating it under deadline pressure later) is non-zero.

## Decision Drivers

- Consistency across every error response in the API.
- Compatibility with existing client tooling and HTTP middleware libraries.
- IETF-standardized; durable; not tied to a specific framework or company.
- Extensible for domain-specific error data (e.g., `correlationId`, validation error lists).
- Reasonable mental model for non-technical streamers' clients in v2+ (developer-streamers consuming our API for custom integrations).
- Cheap to adopt; cheap to discard if we ever change our mind (it's just a JSON shape).

## Considered Options

- **RFC 9457 Problem Details for HTTP APIs** (the IETF standard; obsoletes RFC 7807)
- **JSON:API error format** (the JSON:API spec's error envelope)
- **GraphQL-style errors** (errors in response body, HTTP 200 with `errors[]`)
- **Ad-hoc shape** (define our own `{ error, code, ... }` and version it ourselves)
- **Plain text error messages** (HTTP status + a string body)

## Decision Outcome

Chosen option: **"RFC 9457 Problem Details for HTTP APIs"**, because it is the IETF standard for structured HTTP error responses, has a stable specification that won't change underneath us, ships with a registered Content-Type (`application/problem+json`) that frees us from inventing one, supports first-class extension fields for domain-specific data, and is implementation-cheap (~50 lines for a middleware emitting it; client parsers are commodity). It also obsoletes RFC 7807, so we get the freshest version of the standard and avoid the version-confusion cost of an older spec.

The format we'll emit:

```json
{
  "type": "https://nexus.app/problems/scene-not-found",
  "title": "Scene not found",
  "status": 404,
  "detail": "No scene with ID 'scn_8H2K' exists in layout 'lay_a1b2'.",
  "instance": "/api/v1/layouts/lay_a1b2/scenes/scn_8H2K",
  "correlationId": "req_01jvz7q8m2a3b4c5"
}
```

- `type`: a stable, dereferenceable URI under the `nexus.app/problems/` namespace. Each problem type's URI MUST resolve to human-readable documentation explaining the error and how to fix it. URIs are slug-shaped, kebab-case, never include user data.
- `title`: a fixed short title per problem type, same for every occurrence of that type.
- `status`: matches the actual HTTP status code in the response. Advisory only; the response's status code is the authoritative signal.
- `detail`: occurrence-specific, may include user-facing details. Clients MUST NOT parse this field for programmatic information; it's for humans.
- `instance`: a URI identifying the specific occurrence, typically the request path, optionally with a fragment or query.
- Extension fields (e.g., `correlationId`, `validationErrors`, `retryAfter`) are added per problem type as documented in `type`'s referenced documentation.

Response always includes header: `Content-Type: application/problem+json`.

### Consequences

- **Good**, because every error endpoint follows the same JSON shape; clients write one parser, not N.
- **Good**, because we get a registered MIME type (`application/problem+json`) for free; no invention required for content negotiation.
- **Good**, because the IETF process means the spec won't pivot under us; RFC 9457 will outlive Nexus.
- **Good**, because extension fields let us add domain-specific information (correlation IDs, validation error lists, retry hints) without breaking the schema.
- **Good**, because most HTTP middleware ecosystems (Express, Fastify, Hono, Axum, Spring, ASP.NET, etc.) have RFC 9457 / 7807 helpers; minimal implementation friction.
- **Good**, because observability tools (Sentry, Datadog, OpenTelemetry semantic conventions) increasingly understand RFC 9457 fields out of the box.
- **Bad**, because we commit to maintaining stable `type` URIs that resolve to documentation; an off-by-rename or URI churn is a breaking change.
- **Bad**, because the spec is HTTP-specific; internal `Result<T, E>` types in `core/` and `features/*/model/` use their own discriminated-union error shapes, not Problem Details. Mapping happens at the HTTP boundary (in the backend middleware). Slight cognitive overhead.
- **Neutral**, because we have no HTTP API yet; the cost of adopting this *today* is just the cost of writing this ADR. The benefit is realized starting from v2.

### Confirmation

When v2 backend implementation begins:
- HTTP error middleware emits `application/problem+json` for every non-2xx response.
- Every `type` URI in the codebase has corresponding documentation accessible from the URI (404 on the docs path is a release blocker).
- Schema validation in CI ensures every problem type emitted by any handler matches its declared shape (TypeBox / Zod / JSON Schema).
- An automated contract test fetches every documented `type` URI in CI to catch broken documentation links before release.

For the meantime (v1, no backend): no confirmation needed. This ADR is dormant until v2 implementation starts.

## Pros and Cons of the Options

### RFC 9457 Problem Details

- **Good**, because IETF-standardized; will not change under us.
- **Good**, because extensibility built into the spec (extension fields are first-class).
- **Good**, because content-type is registered (`application/problem+json`); no invention.
- **Good**, because middleware libraries exist across most server stacks.
- **Good**, because supersedes the older RFC 7807; latest standard.
- **Good**, because XML variant exists for the rare contingent where JSON isn't viable (probably never for us).
- **Neutral**, because `type` URIs need to resolve to documentation, a discipline, not a difficulty.

### JSON:API error format

- **Good**, because well-defined and widely used in the JSON:API ecosystem.
- **Bad**, because JSON:API is more than an error format, it's a full API style; adopting JSON:API errors without the rest of JSON:API is an unusual choice.
- **Bad**, because the JSON:API error envelope (`{ errors: [{ id, status, code, title, detail, source, meta, links }] }`) is verbose and assumes errors come in arrays even when a single error is returned.
- **Bad**, because content-type (`application/vnd.api+json`) signals the whole JSON:API contract, not just errors.

### GraphQL-style errors

- **Bad**, because we're not committing to GraphQL; the error shape's semantics depend on the GraphQL request model.
- **Bad**, because HTTP 200 with errors in body is a category mismatch for non-GraphQL HTTP APIs, defeats HTTP status code usage.
- **Bad**, because requires consumers to inspect response body to detect failure even when HTTP status would suffice.

### Ad-hoc shape

- **Bad**, because reinventing a wheel that IETF has rolled.
- **Bad**, because no off-the-shelf middleware; we author and maintain every shape and parser.
- **Bad**, because we'd need to invent the versioning + extension story ourselves.
- **Neutral**, because we'd be free to make any shape we wanted; freedom we don't need.

### Plain text error messages

- **Good**, because trivially simple.
- **Bad**, because no structure, clients can't reliably distinguish error types programmatically.
- **Bad**, because no extension mechanism, adding correlation IDs or validation details requires inventing a side-channel (headers? trailing JSON?).
- **Bad**, because incompatible with the "easy to learn, hard to misuse" principle for any API surface beyond trivial ones.

## More Information

- RFC 9457: [Problem Details for HTTP APIs](https://www.rfc-editor.org/rfc/rfc9457.html)
- Supersedes [RFC 7807](https://www.rfc-editor.org/rfc/rfc7807.html); migration is mostly cosmetic.
- IANA registry for `application/problem+json`: [iana.org/assignments/media-types](https://www.iana.org/assignments/media-types/media-types.xhtml#application)
- Related: [ADR 0000](0000-record-architecture-decisions.md) backlog item #27 (v2 HTTP API design philosophy), this ADR partially fulfills that backlog item by locking in the error-response format. The remainder of #27 (endpoint shapes, versioning strategy, deprecation policy, auth tokens, schema documentation format) remains deferred until v2 implementation starts.
- Related external article: [Florian Kraemer, Most "RESTful" APIs are not really RESTful](https://florian-kraemer.net/software-architecture/2025/07/07/Most-RESTful-APIs-are-not-really-RESTful.html), informs the "HTTP API, not REST API" framing that this error format slots into.

### Operational notes for the v2 implementation

- The Nexus problem-type URI namespace is `https://nexus.app/problems/<slug>` (subdomain / path to be confirmed at backend setup time).
- Each documented problem type is a markdown file in the project's documentation site (probably `docs/problems/<slug>.md`); the URI resolves there.
- The default problem type for unspecified errors is `about:blank` (per RFC 9457's default); we avoid using it, every error should have a registered, documented type.
- Extension field naming: kebab-case for multi-word names where the JSON layer supports it; otherwise camelCase. Stay consistent with `correlationId` shape rather than introducing a new convention.
