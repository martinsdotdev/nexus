---
status: "accepted"
date: 2026-06-05
decision-makers: project owner
consulted:
informed: future contributors
---

# Passwordless first-party authentication (passkeys + email code) alongside Twitch OAuth

## Context and Problem Statement

[ADR-0009](0009-activate-v2-cloud-mode.md) activated v2 cloud mode and chose, in its decision #7, "Twitch first, others as fast-follows" for the editor login, behind an HttpOnly session cookie (#5). As the authentication work begins (it is increment 1 of v2; the relay has no auth code today), two refinements are decided together: which first-party methods to offer beyond Twitch, and how to implement the session and method machinery correctly.

The work is grounded in Pilcrow's Auth Book (the lucia-auth author's opinionated "implement auth properly yourself" guide), adopted as the reference. The question this ADR answers: should Nexus stay Twitch-OAuth-only, or add provider-independent first-party sign-in, and if so by which methods and built to what session model?

## Decision Drivers

* A provider-independent sign-in path, not everyone is a Twitch user, and the hosted product should not hard-require a third party to log in.
* Phishing-resistance and a modern default (passkeys / WebAuthn).
* Minimal credential liability, avoid storing password hashes and the breach/reset/stuffing surface they carry.
* Rigor in the session layer (token design, hashing, cookie attributes, CSRF), the part most often gotten wrong.
* One session layer regardless of how the user authenticated, so methods are additive.
* Reuse ADR-0002's `Auth` port and the single-binary shell; keep `nexus-core` pure; preserve ADR-0005 local-first (local mode keeps no-op auth).

## Considered Options

* **Providers:** Twitch only (ADR-0009 #7 as written) / **Twitch + first-party** (chosen) / first-party only.
* **First-party method:** password-based (Argon2/bcrypt) / **passwordless: passkeys + email code** (chosen) / email-code only / passkeys only.
* **Session mechanism:** stateless JWT / **server-side session cookie per the Auth Book** (chosen).
* **CSRF:** `SameSite` cookie only / **`SameSite=Lax` + server-side origin checks** (chosen).
* **Where auth lives:** the qubit control-plane RPC (ADR-0005's plan) / **plain axum HTTP routes** (chosen).

## Decision Outcome

Keep Twitch OAuth and add **first-party passwordless authentication, passkeys plus email one-time codes**, multi-method with account linking, all over one session layer built to the Auth Book. No passwords. Auth is implemented as plain axum HTTP routes (OAuth redirects, WebAuthn, and `Set-Cookie` are HTTP-native and fit poorly into typed RPC; qubit stays available for later control-plane RPC such as workspace listing). The specific design, taken from the book:

* **Session** ([Sessions](https://auth.pilcrowonpaper.com/sessions)): a token of `id` + a 32-byte random `secret`; the database stores only `SHA-256(secret)`, so a database leak cannot forge a session; validation re-hashes the presented secret and compares in constant time; a fresh session is issued after authentication. The cookie is `HttpOnly` + `Secure` + `SameSite=Lax` + `Path=/`.
* **CSRF** ([CSRF](https://auth.pilcrowonpaper.com/csrf)): `SameSite=Lax` is necessary but not sufficient; every non-GET request is additionally checked server-side for a same-origin `Sec-Fetch-Site` (falling back to an `Origin` whitelist). No GET request changes state.
* **Email code** ([Email code authentication](https://auth.pilcrowonpaper.com/email-code-authentication)): an 8-character code from A-Z and digits excluding `I O 0 1` (>= 40 bits of entropy), hashed with **Argon2id (16 MiB, t=3, p=1)**, valid at most one hour, rate-limited by a token bucket (capacity 5, refill 1/minute), one code per attempt bound to the initiating session, invalidated on use and on email change.
* **Passkeys** ([Passkeys](https://auth.pilcrowonpaper.com/passkeys), [registration](https://auth.pilcrowonpaper.com/passkey-registration), [authentication](https://auth.pilcrowonpaper.com/passkey-authentication)): discoverable WebAuthn credentials requiring user verification; the server stores the credential id, COSE public key, sign count, and relying-party id; ES256 (ECDSA P-256) is the primary algorithm; users may register at least ten named passkeys; the challenge and relying-party id / origin are verified on each ceremony.

### The two-hashes rationale

A deliberate asymmetry the book makes explicit: the 32-byte session **secret** is high-entropy, so a **fast** `SHA-256` is correct (speed matters; brute force is infeasible). The 8-character email **code** is low-entropy (~40 bits), so it demands a deliberately **slow** `Argon2id` (a fast hash would let an attacker brute-force it within the code's one-hour validity). Same codebase, opposite hashing choices, driven by the entropy of the secret.

### Consequences

* Good, because sign-in is provider-independent (Twitch for streamers, passkeys/email for everyone) and the strongest method (passkeys) is phishing-resistant.
* Good, because no passwords means no password-hash breach surface, no reset flow, no credential-stuffing exposure.
* Good, because one session layer serves every method, so adding or linking a method never touches the session machinery.
* Good, because the implementation follows a rigorous, audited reference rather than ad-hoc choices, and the session secret is unforgeable from a database leak.
* Good, because ADR-0005 local-first is untouched: local mode keeps no-op auth; only cloud mode enables the adapter.
* Bad, because three methods plus account-linking is a large first increment (WebAuthn ceremonies, an OAuth flow, email delivery, the session + CSRF core).
* Bad, because email-code adds an external email-provider dependency, and WebAuthn carries inherent client/ceremony complexity.
* Neutral, because qubit is not used for auth (deferred to later typed control-plane RPC), and secondary OAuth providers + the password option are explicitly out.

### Confirmation

The build confirms compliance per sub-increment: `cargo test -p nexus-server` for the session core (token round-trip, `SHA-256` + constant-time, expiry), the CSRF middleware (non-same-origin rejection), email-code (charset/entropy, Argon2id verify, the token bucket, expiry/invalidation), and each method; Playwright e2e for each sign-in flow ending in a set cookie and `GET /auth/me`; and the `/sync` gate (authenticated join succeeds, unauthenticated is rejected, the overlay token is read-only). `cargo tree` continues to prove `nexus-core` does not depend on `nexus-server`.

## Pros and Cons of the Options

### Providers: Twitch + first-party (chosen)

* Good, because it keeps streamer-native one-click while not locking non-Twitch users out.
* Neutral, because accounts must support linking multiple methods.
* Bad (alternatives), because Twitch-only excludes non-Twitch users and hard-couples login to a third party, and first-party-only discards the streamer one-click and the future Twitch chat/channel tie-in ADR-0009 valued.

### Method: passwordless passkeys + email code (chosen)

* Good, because passkeys are phishing-resistant and email-code is a low-friction passwordless fallback and recovery path; neither stores a reusable secret a breach can replay.
* Bad, because it is the largest method surface (WebAuthn + email delivery).
* Bad (alternatives), because passwords add the most liability for the least modern benefit, email-code alone is less phishing-resistant, and passkeys alone lack a recovery path on device loss.

### Session: server-side cookie per the book (chosen) vs JWT

* Good, because a hashed-secret server session is revocable and unforgeable from a DB leak, and one same-origin cookie covers both HTTP and the `/sync` WebSocket upgrade.
* Bad (alternative), because stateless JWTs are hard to revoke and still need a cookie or query param to authenticate the WebSocket.

### CSRF: SameSite + origin checks (chosen)

* Good, because defense-in-depth covers the subdomain and legacy-browser gaps `SameSite` alone leaves.
* Neutral, because it adds an origin-checking middleware on state-changing routes.

### Where: plain axum routes (chosen) vs qubit

* Good, because OAuth redirects, WebAuthn ceremonies, and `Set-Cookie` are HTTP-native and straightforward as routes.
* Neutral, because qubit remains available for later typed control-plane RPC (workspace listing/management), per ADR-0005.

## More Information

* **Extends [ADR-0009](0009-activate-v2-cloud-mode.md) #7** ("Twitch first") to "Twitch + first-party passwordless"; it does not supersede ADR-0009 (ADRs are append-only). It realizes ADR-0009 #5 (the session cookie) with a concrete, book-grounded design, and #6 (the overlay read-only token) remains the overlay's separate path.
* **Grounded in Pilcrow's Auth Book**, adopted as the auth reference: [Authentication methods](https://auth.pilcrowonpaper.com/authentication-methods), [Sessions](https://auth.pilcrowonpaper.com/sessions), [Auth sessions](https://auth.pilcrowonpaper.com/auth-sessions), [CSRF](https://auth.pilcrowonpaper.com/csrf), [Email code authentication](https://auth.pilcrowonpaper.com/email-code-authentication), [Passkeys](https://auth.pilcrowonpaper.com/passkeys) and the WebAuthn pages, [Argon2](https://auth.pilcrowonpaper.com/argon2). Its [passwordless example](https://passwordless-example.auth.pilcrowonpaper.com/) (passkey + email code) is the closest reference implementation. Broader security concerns outside the book's scope defer to the [OWASP Cheat Sheet Series](https://cheatsheetseries.owasp.org/).
* **Deferred:** secondary OAuth providers and the account-linking-by-provider UX; the WebAuthn crate selection (`webauthn-rs` is the likely choice) and email provider (`EmailSender` port; a provider such as Resend/Postmark/SES in cloud, log-only locally); session absolute-lifetime vs sliding-window tuning.
