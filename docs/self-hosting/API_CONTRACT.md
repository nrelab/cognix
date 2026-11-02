# Existing client API contract and replacement boundary

## Decision

No backend is implemented in this milestone. The local-shell workflow does
not require a client-to-Warp API. The current source shows that the app's
remote capabilities use multiple existing service contracts; it does not
provide a complete, authorized specification for replacing Warp's services.

Do not infer a deployable service contract from the endpoint roots alone.
Operation schemas, authorization, entitlement, data retention, and compatibility
requirements must be inventoried for the specific feature before a replacement
is started.

## Current transport surfaces

| Surface | Client evidence | Current shape / trigger | Replacement status |
|---|---|---|---|
| GraphQL | [`crates/graphql/src/client.rs`](../../crates/graphql/src/client.rs), [`app/src/server/server_api.rs`](../../app/src/server/server_api.rs) | HTTP POST to `{server_root}/graphql/v2?op=…`; `send_graphql_request` obtains/refreshes a token if required. Calls occur when an operation is invoked. | Requires operation-by-operation schema and auth contract. |
| Auth/account | [`app/src/auth/auth_manager.rs`](../../app/src/auth/auth_manager.rs), [`app/src/server/server_api/auth.rs`](../../app/src/server/server_api/auth.rs) | `AuthClient::fetch_user`, anonymous-account creation, and credential refresh are flow-driven; anonymous creation is a GraphQL mutation. | No replacement identity protocol is specified here. Do not bypass existing checks. |
| AI | [`app/src/server/server_api/ai.rs`](../../app/src/server/server_api/ai.rs), [`app/src/ai/`](../../app/src/ai/) | Includes GraphQL operations (for example command generation) and service-backed model/usage flows. | No general provider contract or private/local inference compatibility is established. |
| Cloud preferences and objects | [`app/src/settings/cloud_preferences_syncer.rs`](../../app/src/settings/cloud_preferences_syncer.rs), [`app/src/server/cloud_objects/`](../../app/src/server/cloud_objects/) | Sync waits for cloud-object initialization and is triggered by login/settings paths. | Local persistence is not an implementation of the cloud object API. |
| Telemetry | [`app/src/server/telemetry/`](../../app/src/server/telemetry/) | Events can be queued and sent through the server API when event, privacy, and build gates permit. | Optional for a fork only after explicit egress and retention policy decisions. |
| Crash reporting | [`app/src/crash_reporting/`](../../app/src/crash_reporting/) | Build/config/privacy gated. | Requires a fork-owned collector/configuration or explicit user-controlled disablement; no silent change. |
| Updates | [`app/src/autoupdate/`](../../app/src/autoupdate/), [`app/src/server/server_api.rs`](../../app/src/server/server_api.rs) | Version checks use `/client_version/daily` or `/client_version` when updater polling is enabled. | Requires fork-owned update metadata, hosting, signing, and verification. |
| Terminal sessions | [`app/src/terminal/local_tty/spawner.rs`](../../app/src/terminal/local_tty/spawner.rs) | Local shell process through local PTY; optional local terminal-server subprocess. | No Warp backend contract is needed for ordinary local commands. |

## Required contract record before a replacement

For each selected operation, record:

1. Caller, trigger, and client module.
2. Request/response schema, including streaming or subscription behavior.
3. Authentication, authorization, tenancy, and entitlement rules.
4. Sensitive data sent (prompt context, command content, workspace data,
   identifiers) and retention expectations.
5. Timeout, retry, cancellation, idempotency, and offline behavior.
6. Error states and user-visible behavior; failures must not look successful.
7. Test fixtures and compatibility tests against a local fake service.
8. Deployment, TLS, secret storage, versioning, and migration policy.

Until that record exists, categorize the capability as requiring a replacement
or unavailable without the proprietary service. Do not create mock endpoints
that silently accept or discard requests.

## Local-shell contract for the first workflow

The end-to-end contract is intentionally local:

- The app opens a terminal pane and starts a shell through the PTY subsystem.
- User input reaches that shell; its output is rendered by the local terminal.
- A harmless `echo` command produces observable output.
- Closing the session terminates its shell/PTY lifecycle.
- The default integration configuration points HTTP, WebSocket, and Oz service
  roots at the reserved test address `192.0.2.0:9`; the shell command must not
  depend on those services.

The integration channel uses a synthetic test identity. This is a service
unavailability characterization, not proof of all real logged-out startup
behavior. The application can have separately gated telemetry or updater
activity depending on build and runtime configuration.

## Configuration contract warning

`Channel::Oss` is a release channel that does not honor server URL overrides.
The current `WARP.md` local-server snippet used `cargo run --features
with_local_server`, but this is not a current Cargo feature. `script/run`
translates that legacy argument for `warp-channel-config`; the installer is
pinned to a separate private SSH repository. Therefore the source checkout
does not currently demonstrate a supported, external-contributor path for
pointing the OSS binary at a self-hosted API.

See [development setup](./DEVELOPMENT.md) before attempting local service
configuration.
