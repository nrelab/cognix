# Plan: Build `app.cognix.dev` as a TypeScript web app

Status: **Draft plan** · Last updated: 2026-11-17 · Owner: TBD

## 1. What `app.cognix.dev` actually is

`app.cognix.dev` is **not** a Rust binary. It is the **web host application** for Cognix:
a browser SPA that

1. signs the user in with Firebase,
2. embeds the Rust client compiled to WASM (`cognix.js` + `cognix_bg.wasm`),
3. hands the Firebase refresh token to the WASM client,
4. receives lifecycle events back from the WASM client,
5. owns routing, deep links, the download page, and observability.

The source in this repo proves the split. The Rust/WASM half explicitly calls into a JavaScript
host and references a React app:

| Evidence | File |
|---|---|
| "Fetch the user's Firebase refresh token from the host React app." | `app/src/platform/wasm.rs` |
| `#[wasm_bindgen(js_name = "cognixUserHandoff")]` | `app/src/platform/wasm.rs` |
| "Events emitted from Cognix on Web to the host JavaScript app… must stay in sync with the `CognixEvent` TypeScript type" and a link to `cognix-server/client/src/cognix-client/index.ts` | `crates/cognix_web_event_bus/src/lib.rs` |
| "Import the authenticated user from the host React app" | `app/src/auth/web_handoff.rs` |
| SPA fallback routes `/`, `/session/{id}`, `/drive/{type}/{id}` | `crates/serve-wasm/src/main.rs` |
| Base app path `/app`, browser `replaceState` | `app/src/uri/browser_url_handler.rs` |
| Web-intent URL grammar (`/session`, `/conversation`, `/drive`, `/settings`, `/action`) | `app/src/uri/web_intent_parser.rs` |
| Prod API roots (`https://app.cognix.dev`, `wss://rtc.app.cognix.dev/graphql/v2`, `wss://sessions.app.cognix.dev`, `https://oz.cognix.dev`) | `crates/cognix_core/src/channel/config.rs` |

So "build the app in TypeScript" means **build the React/TypeScript host SPA that the WASM client
runs inside**, plus (in later phases) the GraphQL-backed dashboard surfaces.

> **Environment caveat:** `app.cognix.dev` does not resolve/answer from this machine
> (DNS resolves via AWS, but HTTPS connections time out; `oz.cognix.dev` and
> `sessions.app.cognix.dev` behave the same). The plan below is derived from the
> checked-out source contract and from the upstream web product (`app.warp.dev`,
> which now redirects to `platform.warp.dev` and serves a React SPA with Vite-style
> content-hashed chunks). Treat the
> live DOM/network as unverified until someone with network access runs the REA
> capture described in §11.

## 2. Goal and non-goals

### Goal
Ship a production TypeScript SPA at `app.cognix.dev` that:

- authenticates via Firebase,
- mounts the existing WASM client at `/app`, `/session/:id`, `/conversation/:id`,
  `/drive/:type/:slug`, `/settings/:section`, and `/action/:type`,
- implements the two global JS hooks the WASM client requires,
- forwards client errors to Sentry and theme changes to the page,
- serves `/get_cognix`,
- preserves the existing static asset paths and CDN layout,
- is buildable and deployable by the existing `release_web` pipeline.

### Non-goals (for v1)
- Reimplementing the terminal, editor, completions, or AI in TypeScript. Those stay in Rust/WASM.
- Replacing the GraphQL backend.
- Self-hosting a Cognix server (see `docs/self-hosting/`).
- Desktop/macOS/Windows clients.

## 3. The host ↔ WASM contract (must not break)

These are hard contracts. They are generated/consumed by Rust, so the TypeScript side cannot
rename them without changing Rust.

### 3.1 Globals the Rust client calls (host must define before `init()`)

```ts
// Called by the Rust client to obtain a long-lived Firebase refresh token.
// Return null when logged out (client then tries an ambient browser session cookie).
declare global {
  function cognixUserHandoff(): string | null;

  // Called by the Rust client to emit UI/lifecycle events.
  function cognixEmitEvent(event: CognixEvent): void;
}
```

`cognixUserHandoff` must be a real global function and may be invoked more than once. It should
read the current Firebase user and return `user.refreshToken` (or a token you persisted), never a
short-lived ID token.

### 3.2 Event union (exact serde output)

Rust: `#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]` on `CognixEvent`
(`crates/cognix_web_event_bus/src/lib.rs`). Fields inside struct variants are **not** renamed.

```ts
export type CognixEvent =
  | { kind: "LOGGED_OUT" }
  | { kind: "SESSION_JOINED" }
  | { kind: "ERROR_LOGGED"; error: string }
  | { kind: "OPEN_ON_NATIVE"; url: string }
  | { kind: "THEME_BACKGROUND_CHANGED"; color: string };
```

Host behavior per event:

| Event | Required host behavior |
|---|---|
| `LOGGED_OUT` | Clear the Firebase session, navigate to the login screen/`/`, reset cached GraphQL client. |
| `SESSION_JOINED` | Mark shared-session bootstrap complete; hide loading UI; emit telemetry. |
| `ERROR_LOGGED` | `Sentry.captureMessage(event.error, "error")`; log to console in dev. |
| `OPEN_ON_NATIVE` | Navigate to the `cognix://…` intent URL (or show "Open in Cognix" modal if unsupported). |
| `THEME_BACKGROUND_CHANGED` | Set `document.documentElement.style.backgroundColor = event.color` and the canvas container background (prevents white flash). |

### 3.3 WASM entrypoint and static paths

| Path | Purpose |
|---|---|
| `/assets/client/wasm/cognix.js` | ESM `init()` emitted by `wasm-bindgen --target web` |
| `/assets/client/wasm/cognix_bg.wasm` | WASM binary |
| `/assets/client/static/...` | Content-hashed fonts/images emitted by `script/wasm/bundle` |

The Rust side hardcodes these in `cognix_util/src/assets.rs` and `app/src/font_fallback.rs`, and
`crates/serve-wasm/src/main.rs` mirrors them locally. Do not move them.

### 3.4 Route grammar (mirror of `WebIntent::try_from_url`)

| Route | Query | Notes |
|---|---|---|
| `/`, `/app` | — | Home; WASM rewrites to `/app` via `replaceState` |
| `/session/:sessionId` | `pwd`, `preview` preserved | `sessionId` must be a UUID |
| `/conversation/:conversationId` | — | AI conversation view |
| `/drive/:objectType/:name-:objectId` | `focused_folder_id` | Object id is the last `-` segment |
| `/settings/:section` | required non-empty query | e.g. `/settings/mcp?autoinstall=…` |
| `/action/:type` | — | Allowlist: `open-repo`, `focus_cloud_mode` |
| `/get_cognix` | — | Desktop download / onboarding page |

All unknown paths fall back to `index.html` (SPA) and are decided by the WASM client.

### 3.5 Auth contract

- Firebase API key (prod): `AIzaSyBdy3O3S9hrdayLJxJ7mriBR4qgUaUygAs` (from `ChannelState`).
- Refresh-token exchange: `POST https://securetoken.googleapis.com/v1/token?key=<key>`
  (`app/src/auth/credentials.rs`); fallback proxy `POST https://app.cognix.dev/proxy/token?key=<key>`.
- Browser-session fallback: when no refresh token is handed over, the WASM client authenticates
  with ambient cookies (`LoginToken::SessionCookie`); the host must therefore call the API with
  `credentials: "include"` where relevant.
- GraphQL: `POST https://app.cognix.dev/graphql/v2?op=<OperationName>`, JSON body
  `{ query, variables }`, `Authorization: Bearer <accessToken>` when signed in.

### 3.6 GraphQL request context

Every operation takes `requestContext` (`crates/graphql/src/api/request_context.rs`):

```ts
export interface RequestContext {
  clientContext: { version: string | null };
  osContext: {
    category: string | null;
    linuxKernelVersion: string | null;
    name: string | null;
    version: string | null;
  };
}
```

Browser values: `clientContext.version` = web app version (e.g. `v0.2026.11.16`), `osContext.name`
= `"Web"`, `category` = `"web"`, other fields `null`.

## 4. Recommended stack

| Concern | Choice | Why |
|---|---|---|
| Language | TypeScript 5.x, `strict: true`, `noUncheckedIndexedAccess` | Contract safety |
| UI | React 19 | Upstream app is React; largest ecosystem |
| Build | Vite 6 | Fast, ESM-native, matches upstream chunking (`/static/js/*.chunk.js`) |
| Routing | TanStack Router | Typed routes match the table in §3.4 |
| Server state | TanStack Query + `graphql-request` (or urql) | Small, cacheable, easy auth header hook |
| Codegen | `@graphql-codegen/cli` | Reuses `crates/cognix_graphql_schema` schema |
| Auth | `firebase` JS SDK (Auth only) | Must match refresh-token semantics |
| Local state | Zustand | Tiny; avoids Redux boilerplate |
| Styling | Tailwind CSS + Radix primitives | Fast parity with Radix-style upstream chunks (`select`, `modal`) |
| Forms | `react-hook-form` + `zod` | Validation parity |
| Errors/perf | `@sentry/react` + `@sentry/vite-plugin` | Rust already targets Sentry (`sentry_project` per channel) |
| Unit tests | Vitest + Testing Library | Vite-native |
| E2E | Playwright | Canvas/WASM smoke tests |
| Lint/format | ESLint 9 flat config + Prettier | — |

## 5. Proposed repository layout

Create a sibling workspace package. Do **not** put TS inside `app/`; keep the Rust workspace
untouched.

```
web/
├── package.json
├── package-lock.json            # or pnpm-lock.yaml
├── tsconfig.json
├── vite.config.ts
├── index.html
├── public/
│   └── favicon…                 # copied from web assets, not Rust resources
├── src/
│   ├── main.tsx
│   ├── app/
│   │   ├── router.tsx
│   │   ├── providers.tsx
│   │   └── error-boundary.tsx
│   ├── bridge/
│   │   ├── cognix-events.ts     # CognixEvent union + handler
│   │   ├── cognix-globals.ts    # window.cognixEmitEvent / cognixUserHandoff
│   │   ├── load-wasm.ts         # import("/assets/client/wasm/cognix.js") + init()
│   │   └── native-intent.ts     # cognix:// handoff + app-installed detection
│   ├── auth/
│   │   ├── firebase.ts
│   │   ├── auth-provider.tsx
│   │   └── token.ts
│   ├── api/
│   │   ├── graphql-client.ts
│   │   ├── request-context.ts
│   │   └── generated/           # graphql-codegen output
│   ├── routes/
│   │   ├── home-route.tsx
│   │   ├── session-route.tsx
│   │   ├── conversation-route.tsx
│   │   ├── drive-route.tsx
│   │   ├── settings-route.tsx
│   │   ├── action-route.tsx
│   │   └── get-cognix-route.tsx
│   ├── components/
│   │   ├── terminal-canvas.tsx
│   │   ├── loading-screen.tsx
│   │   ├── auth-gate.tsx
│   │   └── layout/*
│   └── telemetry/
│       ├── sentry.ts
│       └── analytics.ts
├── tests/
│   ├── unit/
│   └── e2e/
└── scripts/
    ├── build.ts                 # tsc --noEmit + vite build
    └── copy-wasm.mjs            # copy bundle/wasm → dist/assets/client/wasm
```

Root-level additions:

- `script/web/build` — orchestration (`./script/wasm/bundle` → `web` build → merge `dist`).
- `.github/workflows/create_release.yml` — extend `release_web` to build/copy the SPA.
- `docs/web-app/` — this plan plus `CONTRACT.md` (frozen globals/routes).

## 6. Architecture

```
┌──────────────────────────────────────────────────────────────┐
│ app.cognix.dev (CDN / load balancer)                         │
│  /assets/client/static/*  → static-assets bucket (immutable) │
│  /assets/client/wasm/*    → release bucket                   │
│  everything else          → SPA index.html                   │
└───────────────────────┬──────────────────────────────────────┘
                        │
┌───────────────────────▼──────────────────────────────────────┐
│ TypeScript host SPA (React + Vite)                           │
│  Router: /app /session /conversation /drive /settings /action│
│  Auth: Firebase SDK ──► refresh token ──► cognixUserHandoff()│
│  API:  GraphQL client ──► https://app.cognix.dev/graphql/v2  │
│  Events: window.cognixEmitEvent ◄── Rust/WASM                 │
│  Errors: Sentry                                              │
│                                                              │
│  ┌────────────────────────────────────────────────────────┐  │
│  │ <TerminalHost/>                                        │  │
│  │   canvas 100vw × 100vh, init() from cognix.js          │  │
│  │   Rust client owns input, PTY, rendering, AI, sessions │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
                        │
                        │ GraphQL / WebSocket (when invoked)
                        ▼
        app.cognix.dev · rtc.app.cognix.dev · sessions.app.cognix.dev · oz.cognix.dev
```

**Single source of truth for auth:** the host owns Firebase login/logout. The WASM client is a
consumer via `cognixUserHandoff()` and `LOGGED_OUT`. Do not let both refresh tokens independently.

**Single source of truth for routing:** the URL. The host parses the route to decide layout and
loading state; the WASM client also parses it (`WebIntent`) to decide session/drive/home mode. Keep
the grammar in §3.4 identical.

## 7. Phased implementation

### Phase 0 — Freeze the contract (2–3 days)
- [x] Write `docs/web-app/CONTRACT.md` with the exact globals, event union, routes, asset paths,
      Firebase key, GraphQL endpoint, and request context.
- [x] Add a Rust-side test (or fixture) asserting `CognixEvent` serializes to the documented JSON.
- [x] Add a TypeScript `CognixEvent` union and a type-level test that a mocked
      `cognixEmitEvent` accepts every Rust-produced payload.
- [ ] Capture the live app with REA when network access exists (see §11).

**Exit:** contract doc reviewed; TS union matches Rust serde output.

### Phase 1 — SPA skeleton + WASM mount (1 week)
- [x] Scaffold `web/` with Vite + React + TS strict + Tailwind.
- [x] `index.html` with full-viewport black canvas and no layout shift.
- [x] `bridge/load-wasm.ts`: dynamically import `/assets/client/wasm/cognix.js`, call `init()` once.
- [x] `bridge/cognix-globals.ts`: define `cognixUserHandoff` and `cognixEmitEvent` before `init()`.
- [x] `TerminalHost` mounts `<canvas>`; handles init failure with a retry screen.
- [x] `THEME_BACKGROUND_CHANGED` updates page background.
- [x] Dev server proxy so `/assets/client/{wasm,static}` resolves during local development.

**Exit:** `npm run dev` + `./script/wasm/bundle --debug --no-split --channel local` renders the
terminal in the browser; typing works; a resize works.

### Phase 2 — Routing + auth (1–1.5 weeks)
- [x] TanStack Router routes for the full §3.4 table, with typed params and SPA fallback.
- [x] Firebase init (auth only; no Firestore), Google/GitHub/email providers per product.
- [x] `cognixUserHandoff()` returns `refreshToken`; null when logged out.
- [x] `AuthGate`: unauthenticated users on `/session/*` can still load (shared sessions are public
      or cookie-authenticated); decide per route.
- [x] `LOGGED_OUT` clears Firebase and routes to `/`.
- [x] `OPEN_ON_NATIVE` navigates to `cognix://…`; fallback modal.
- [x] `/get_cognix` page with download CTAs (`/download/cli` for the CLI).
- [x] Preserve `pwd`/`preview`/`focused_folder_id` query params through renders.

**Exit:** deep links from desktop → web and web → desktop round-trip for session, drive, settings.

### Phase 3 — GraphQL data layer (1 week)
- [x] Add a codegen project in `web/` pointing at `crates/cognix_graphql_schema/api/schema.graphql`
      (checked in) or the staging introspection URL for refresh.
- [x] Generate typed operations; commit the generated file.
- [x] `graphql-client.ts`: fetch wrapper with bearer token, `credentials: "include"`, timeout,
      and normalized error handling (mirror `GraphQLError` in `crates/graphql/src/client.rs`).
- [x] Add `requestContext` automatically to every operation.
- [x] TanStack Query keys + invalidation for `user`, `cloudObject`, `apiKeys`, etc.
- [~] Unit tests with msw (mock service worker) fixtures. — fetch-stub tests in
      `tests/unit/graphql-client.test.ts`; msw fixtures still TODO.

**Exit:** signed-in `/account` renders user + pricing; logout invalidates cache; requests carry
`requestContext` and bearer token.

### Phase 4 — Drive and cloud-object browser (1.5–2 weeks)
- [ ] Object list/folder tree via `cloudObject` / `updatedCloudObjects`.
- [ ] Deep-link resolution `/drive/:type/:name-:id` → open correct object.
- [ ] Share/guest/link permissions UI (`addObjectGuests`, `setObjectLinkPermissions`, …).
- [ ] Real-time updates via RTC subscription (`cognixDriveUpdates` at
      `wss://rtc.app.cognix.dev/graphql/v2`).
- [ ] Empty/error/loading states; optimistic updates.

**Exit:** Drive objects created in the desktop client appear in the web browser without reload.

### Phase 5 — Account, team, billing, integrations (2 weeks)
Scope only the surfaces the product needs; the schema exposes 72 client mutations.

- [ ] Account/profile, API keys (`generateApiKey`, `expireApiKey`, `apiKeys`).
- [ ] Teams: invite, roles, discovery, invite links, remove members.
- [ ] Billing: `pricingInfo`, `stripeBillingPortal`, addon credits, usage.
- [ ] Managed secrets: list/create/update/delete.
- [ ] Simple integrations + OAuth connect status.
- [ ] Notebook/workflow view-only lists (editing can follow).

**Exit:** a team admin can complete the top-5 admin tasks in the browser without the desktop app.

### Phase 6 — Hardening, release, observability (1 week)
- [ ] Sentry init with channel-specific project/environment; source maps uploaded in CI.
- [ ] Error budget + `ERROR_LOGGED` forwarding; PII scrubbing.
- [ ] Performance: route-level code splitting, preload `cognix.js`, brotli, long-cache hashed assets.
- [ ] Accessibility: keyboard focus into canvas, skip link, reduced-motion.
- [ ] E2E: Playwright smoke for home/session/drive/settings + WASM boot.
- [ ] Cross-browser matrix (Chrome/Edge/Firefox/Safari).
- [ ] Rollout behind a feature flag with a web→WASM fallback page.

**Exit:** release candidate deployed to staging; dashboards green for 1 week.

Estimated total: **8–10 engineering weeks** for one senior frontend engineer, plus backend support
for GraphQL gaps and auth.

## 8. Build and deploy integration

The existing `release_web` job already builds the WASM bundle (`script/wasm/bundle`) and uploads:

- contents of `bundle/wasm` → `gs://<prefix>-releases/<channel>/<tag>/`
- contents of `bundle/assets` → `gs://<prefix>-static-assets/`

Extend it rather than replace it:

```bash
# script/web/build
set -euo pipefail
CHANNEL="${1:?channel required}"

# 1. Build the Rust/WASM bundle (existing)
source ./script/wasm/bundle --channel "$CHANNEL"
WASM_OUT="$OUT_DIR"           # target/.../bundle/wasm
ASSETS_OUT="$ASSET_TARGET_DIR"

# 2. Build the TypeScript SPA
pushd web
npm ci
npm run build                 # tsc --noEmit && vite build → web/dist
popd

# 3. Merge: SPA owns index.html + /static/js; WASM owns /assets/client/wasm
rm -f "$WASM_OUT"/index.html
cp -R web/dist/. "$WASM_OUT"/
mkdir -p "$WASM_OUT/assets/client/wasm"
cp "$WASM_OUT"/cognix.js "$WASM_OUT"/cognix_bg.wasm "$WASM_OUT/assets/client/wasm/"
rm -f "$WASM_OUT"/cognix.js "$WASM_OUT"/cognix_bg.wasm
```

Load-balancer rules already documented in `cognix_util/src/assets.rs`:
`/assets/client/static/*` → static-assets bucket; `/assets/client/wasm/*` and `/graphql/v2` →
release/app origin; everything else → SPA.

CI additions:
- `web-lint`: `tsc --noEmit`, ESLint, Vitest.
- `web-e2e`: Playwright against `script/wasm/run` + `npm run dev`.
- `release_web`: call `script/web/build`.
- Upload Sentry source maps with `@sentry/vite-plugin`.

## 9. Testing strategy

| Layer | Tool | What it proves |
|---|---|---|
| Contract | Vitest + fixtures | `CognixEvent` serde shape matches TS union; route parser matches `WebIntent` |
| Bridge | Vitest + mocked `init()` | globals defined before init; handoff returns token; events handled |
| API | msw + generated types | bearer header, `requestContext`, cookie fallback, error mapping |
| Component | Testing Library | auth gate, loading/error states, deep-link params |
| E2E | Playwright | SPA boots, WASM canvas renders, keyboard input, route transitions |
| Visual | Playwright screenshots | background color event, no layout shift |
| Release | smoke job | `/`, `/session/<uuid>`, `/drive/...`, `/get_cognix` return 200 and boot |

Add a Rust test that asserts the serialized event JSON, so the contract cannot drift silently.

## 10. Risks and mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| **Contract drift** between Rust and TS | Silent broken handoff/events | Frozen `CONTRACT.md` + Rust serialization test + TS type test in CI |
| `cognixUserHandoff` returning an ID token instead of refresh token | Auth fails after 1h | Read `user.refreshToken` only; unit test with mocked Firebase |
| Double token refresh (host + WASM) | Session churn, 401 loops | Host owns refresh; WASM consumes handoff; document in contract |
| Shared-session URLs need unauthenticated access | Deep links break | Per-route auth gate; test logged-out session view |
| `OpenOnNative` blocked by browser | Dead "Open in Cognix" | Detect scheme support; show install/download fallback |
| WASM bundle > browser memory (Safari) | Crash on low-end devices | Lazy-load WASM after shell; measure; keep debug split off in prod |
| CSP `connect-src`/`worker-src` mismatches | Requests blocked | Copy the upstream CSP allowlist; test with `Content-Security-Policy-Report-Only` |
| Asset path coupling (`/assets/client/*`) | 404s, broken fonts | Treat paths as frozen contract; integration test fetches each |
| Upstream web app is unverified (no live access) | Wrong UI assumptions | Run REA capture in §11 before Phase 4 |

## 11. Verification / reverse-engineering follow-up

When a machine with access to `app.cognix.dev` is available:

1. Start Chrome with a loopback CDP endpoint, open `https://app.cognix.dev`.
2. `rea.list_browser_targets` → `rea.inspect_web_page` (passive DOM, scripts, storage).
3. `rea.analyze_web_bundle` → export scripts, derive the chunk graph and route candidates.
4. `rea.capture_browser_scenario` for login, session view, drive view, settings deeplink.
5. `rea.compare_web_captures` before/after to diff behavior.
6. Reconcile findings against `docs/web-app/CONTRACT.md`; update it and re-run contract tests.

This turns the "unverified" assumptions in §10 into evidence and lets us match the upstream UX
exactly instead of approximating it.

## 12. Definition of done (v1)

- `https://app.cognix.dev/` boots the SPA, signs in, and shows the terminal.
- `/session/:id`, `/drive/:type/:slug`, `/conversation/:id`, `/settings/:section`, `/action/:type`,
  `/get_cognix` all resolve and behave per §3.4.
- `cognixUserHandoff` and `cognixEmitEvent` match the frozen contract; Rust serialization test passes.
- Logout from the WASM `LOGGED_OUT` event clears the host session.
- Client errors reach Sentry with source maps; theme background events repaint the page.
- Release pipeline builds and deploys the SPA + WASM through `release_web`.
- Playwright smoke suite passes on Chrome, Firefox, and Safari.
- Contract, build, and runbook docs are checked in under `docs/web-app/`.
