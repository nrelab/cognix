# Contract: `app.cognix.dev` web host ↔ WASM client

Status: **Frozen for v1** · Last updated: 2026-11-17
Source of truth: `docs/web-app/PLAN.md` §3. Any change here requires a matching Rust change
and a re-run of the contract tests in `web/tests/unit/` and `crates/cognix_web_event_bus`.

## 1. Globals the host must define before `init()`

Defined in `web/src/bridge/cognix-globals.ts`, backed by `app/src/platform/wasm.rs` and
`app/src/auth/web_handoff.rs`:

```ts
function cognixUserHandoff(): string | null;   // Firebase *refresh* token, or null
function cognixEmitEvent(event: CognixEvent): void;
```

- `cognixUserHandoff` may be called multiple times and must be a real global function.
- It must return `user.refreshToken` (long-lived), **never** a short-lived ID token.
- Return `null` when logged out; the WASM client then falls back to an ambient browser
  session cookie (`LoginToken::SessionCookie`), so API calls must use `credentials: "include"`.
- The host owns Firebase login/logout and token refresh. The WASM client is a consumer only.

## 2. `CognixEvent` union (exact serde output)

Rust: `#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]` on `CognixEvent`
(`crates/cognix_web_event_bus/src/lib.rs`). Variant fields are **not** renamed.

```ts
export type CognixEvent =
  | { kind: "LOGGED_OUT" }
  | { kind: "SESSION_JOINED" }
  | { kind: "ERROR_LOGGED"; error: string }
  | { kind: "OPEN_ON_NATIVE"; url: string }
  | { kind: "THEME_BACKGROUND_CHANGED"; color: string };
```

| Event | Required host behavior |
|---|---|
| `LOGGED_OUT` | Clear Firebase session, reset cached GraphQL client, navigate to `/`. |
| `SESSION_JOINED` | Mark shared-session bootstrap complete; hide loading UI; emit telemetry. |
| `ERROR_LOGGED` | `Sentry.captureMessage(error, "error")`; `console.error` in dev. |
| `OPEN_ON_NATIVE` | Navigate to the `cognix://…` intent URL; show "Open in Cognix" fallback if unsupported. |
| `THEME_BACKGROUND_CHANGED` | Set `document.documentElement.style.backgroundColor` and the app container background to `color` (prevents white flash). |

Serialization test: `crates/cognix_web_event_bus/src/lib.rs` (`cargo test -p cognix_web_event_bus`).
Type-level/fixture test: `web/tests/unit/cognix-events.test.ts`.

## 3. WASM static paths (frozen)

| Path | Contents |
|---|---|
| `/assets/client/wasm/cognix.js` | ESM `init()` from `wasm-bindgen --target web` |
| `/assets/client/wasm/cognix_bg.wasm` | WASM binary |
| `/assets/client/static/...` | Content-hashed fonts/images from `script/wasm/bundle` |

Hardcoded in `cognix_util/src/assets.rs`, `app/src/font_fallback.rs`, and mirrored in
`crates/serve-wasm/src/main.rs`. Do not move or rename.

## 4. Route grammar (mirror of `WebIntent::try_from_url`)

`app/src/uri/web_intent_parser.rs`. Base app path `/app`; the WASM client rewrites `/` → `/app`
via `history.replaceState`.

| Route | Query | Validation |
|---|---|---|
| `/`, `/app` | — | Home |
| `/session/:sessionId` | `pwd`, `preview` preserved | `sessionId` must be a UUID |
| `/conversation/:conversationId` | — | Non-empty id |
| `/drive/:objectType/:slug` | `focused_folder_id` optional | Object id = substring after last `-` in `:slug` |
| `/settings/:section` | **required, non-empty** | e.g. `/settings/mcp?autoinstall=…` |
| `/action/:type` | — | Allowlist: `open-repo`, `focus_cloud_mode` |
| `/get_cognix` | — | Desktop download / onboarding page (host-owned, no WASM) |

All unknown paths fall back to `index.html`; the WASM client decides via `WebIntent`.

## 5. Auth endpoints

- Firebase API key (prod): `AIzaSyBdy3O3S9hrdayLJxJ7mriBR4qgUaUygAs` (from `ChannelState`).
- Firebase web config (JS SDK, `cognix-server` project): `apiKey` =
  `AIzaSyDxv9pPEncqMuMVMx6YaDSe72sDOEn80Ck`, `authDomain` = `cognix-server.firebaseapp.com`,
  `projectId` = `cognix-server`, `appId` = `1:547450301010:web:541abe8a988e611864cd8b`.
  (The REST token-exchange key in `ChannelState` differs; both are valid Firebase
  API keys for the project.) Overridable via `VITE_FIREBASE_*` env vars.
- Refresh-token exchange: `POST https://securetoken.googleapis.com/v1/token?key=<key>`
  (`app/src/auth/credentials.rs`); host-side fallback proxy `POST https://app.cognix.dev/proxy/token?key=<key>`.
- GraphQL: `POST https://app.cognix.dev/graphql/v2?op=<OperationName>`, JSON body
  `{ query, variables }`, `Authorization: Bearer <firebase id token>` when signed in.

## 6. `requestContext` (every GraphQL operation)

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

Browser values: `clientContext.version` = web app version, `osContext.name = "Web"`,
`osContext.category = "web"`, remaining fields `null`.
