# Local-first and self-hosted design

## Principle

Keep local shell execution and personal local state independent of account
services. Represent remote capabilities as explicit providers with visible
unavailable states. Do not bypass authentication, fabricate a replacement for
proprietary services, or silently disable checks. The classifications below
describe the evidence in the checked source, not a completed fork.

Classifications:

- **LOCAL** — usable without a remote account/service in the observed path.
- **OPTIONAL REMOTE** — local core remains available; remote behavior is
  opt-in or must degrade explicitly.
- **REQUIRES REPLACEMENT** — self-hosting needs a compatible implementation,
  configuration, or release path not demonstrated by this checkout.
- **UNAVAILABLE WITHOUT PROPRIETARY SERVICE** — behavior relies on a Warp
  service whose self-hosted replacement is not evidenced.

## Capability boundary matrix

| Capability | Classification | Evidence and design boundary |
|---|---|---|
| Local terminal and shell management | **LOCAL** | `local_tty` creates local PTYs and shells; keep this startup path independent from auth, AI, cloud sync, and telemetry. Remote SSH/WSL remains an optional mode. |
| Local settings and session persistence | **LOCAL** | Platform-native preferences and `local_fs` SQLite are present. Public TOML settings are feature-flagged; document the selected backend and preserve existing migrations. |
| Authentication and account-dependent features | **REQUIRES REPLACEMENT** for self-hosted account features; **LOCAL** shell must remain usable without signing in where technically feasible. | Auth state uses secure storage/Firebase token handling. Never remove entitlement or auth checks as an offline workaround. Make account-required operations fail visibly and independently. |
| AI provider integration | **REQUIRES REPLACEMENT** for non-Warp providers; intended alternate provider may be **OPTIONAL REMOTE**. | The inspected app obtains the AI client through `ServerApiProvider`; no general configurable provider or local inference endpoint was established. Introduce a provider abstraction only after inventorying the required operations. |
| Remote storage and synchronization | **UNAVAILABLE WITHOUT PROPRIETARY SERVICE** until a compatible service is supplied. | Cloud object and preference sync use server paths. Local SQLite/settings are separate and do not imply self-hosted sync. |
| Telemetry and diagnostics | **OPTIONAL REMOTE** | Collection is privacy-gated and has remote-send/local-file paths. Add an auditable egress policy; offline/disabled telemetry must not block local operation. Exact defaults remain build/runtime-dependent. |
| Updates and release infrastructure | **REQUIRES REPLACEMENT** for an independently distributed fork. | Channel config carries autoupdate config and release identity; local-source build already depends on generated channel configuration. Signing, notarization, update hosting, and provenance need a fork-owned process. |

## Configuration and state separation

Keep current storage semantics intact until an explicit migration is designed.
The proposed separation is conceptual; do not invent paths or silently migrate
existing user data.

1. **User configuration:** Public settings through the selected existing
   backend (platform-native by default; TOML only where `SettingsFile` is
   enabled).
2. **Secrets:** Auth/provider credentials only in the platform secure-storage
   mechanism or a reviewed secret provider. Never put tokens in TOML, command
   lines, logs, crash reports, or checked-in environment files.
3. **Runtime state:** SQLite/session restoration and caches under the existing
   channel-specific Warp state paths.
4. **Logs/diagnostics:** Existing channel-specific log and telemetry paths;
   scrub credentials and command contents according to data policy.
5. **Remote service configuration:** Service roots and feature/provider
   selection as non-secret configuration; credentials remain separate.

The actual app identity/path helpers (`warp_core` and channel `AppId`) must be
used to determine concrete locations per platform/channel. Changing an AppId
can create a new state namespace and should not be used as an undocumented
data-loss workaround.

## Explicit offline mode

### Proposed contract

- Local terminal startup, PTY management, terminal input/rendering, and local
  settings/session persistence continue without an account or network.
- Remote-dependent operations expose a typed unavailable/offline state,
  retain user-entered work where safe, and can retry when connectivity returns.
- No auth check is skipped. Account-required features remain account-required;
  they simply report service unavailability rather than blocking the terminal.
- A user-visible `offline` setting/flag is explicit and discoverable. It
  prevents outbound calls by policy rather than relying on DNS failure.
- Offline mode does not delete or rewrite cached credentials, cloud state, or
  update configuration.

### Implementation sequence

1. The local-shell portion is now covered by the passing
   `test_local_command_with_unreachable_services` integration test, using the
   isolated test HOME and unreachable Integration-channel endpoints. It does
   not yet assert a visible remote-feature error or a real logged-out OSS
   first-run flow; add that assertion before treating the full characterization
   as complete.
2. Expand the initial service-boundary map in
   [`../self-hosting/API_CONTRACT.md`](../self-hosting/API_CONTRACT.md) into an
   operation-level request inventory and build an allowlist/denylist for any
   offline mode.
3. If needed, add one small network policy at the existing transport/provider
   boundary and inject it into service clients. Avoid broad global networking
   abstractions if existing client seams suffice.
4. Add offline/online transition, retry, cancellation, and privacy tests.
5. Document which features require external Warp services or replacements.

## AI provider design

Do not assume the current `ai` crate is a provider interface. First inventory
the operations requested by app code and those implemented by the Warp server.
If the operations can be represented cleanly, define the smallest interface
around them (request/response, streaming, cancellation, errors, model
capabilities). Keep the existing Warp Server API adapter and add an optional
adapter for a compatible private endpoint. Add a local inference adapter only
if the current execution and resource model permits it.

Credentials must be resolved at runtime from secure storage or the provider's
supported secret mechanism; never hardcode credentials. Provider endpoints,
TLS policy, request logging, retention, and supported capabilities must be
documented. Do not send prompt context to a provider without explicit consent
and data-flow review.

## Discoverable network policy

For each request, record owner/module, URL class (not secret URL/token),
trigger, data category, auth requirement, timeout/retry behavior, offline
behavior, and telemetry/privacy setting. Test using fake transports and
request-capture assertions. DNS-blocking tests alone cannot prove no call was
attempted. Include crash reporting, update checks, auth refresh, GraphQL,
telemetry, AI, cloud storage, and remote-terminal paths in the inventory.

## External service disclosure for a fork

Before calling a fork self-hostable, publish a deployment matrix for:

- Auth/Firebase and account-backed features.
- GraphQL/server APIs and cloud objects/settings synchronization.
- Warp AI operations and endpoint compatibility.
- Telemetry/crash reporting and data retention.
- Channel config, update feeds, signing/notarization, and release distribution.
- Remote SSH/server components, where used.

For every entry state whether it is disabled by explicit user choice, optional,
requires a fork-operated service, or remains unavailable without Warp's
proprietary service. No silent fallback may make a failed remote operation look
successful.
