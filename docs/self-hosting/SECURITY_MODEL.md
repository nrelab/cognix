# Security model for local-first and self-hosted work

This is a boundary and design record, not a finding that the inspected code is
vulnerable. No authentication or authorization behavior is changed by the
local-shell workflow.

## Assets

- User credentials, OAuth/Firebase tokens, and secure-storage contents.
- Shell command input/output, environment variables, working directories, and
  local process handles.
- Local settings, SQLite/session state, and cloud-sync caches.
- AI prompts and workspace/terminal context sent to remote providers.
- Cloud objects and account/team data.
- Telemetry, crash reports, update metadata, signing keys, and release
  provenance.

## Trust boundaries

```text
User / local shell
   | terminal input, environment, process output
   v
Cognix app and PTY / optional local terminal-server process
   | IPC and local persistence
   +-----------------> local files, secure storage, SQLite
   |
   +-----------------> configured HTTP / GraphQL / WebSocket services
   +-----------------> AI / telemetry / crash / update services
```

The local shell is a powerful child process boundary: it executes the user's
commands with the user's permissions. Any local terminal-server IPC must retain
its existing peer and message validation; this work does not weaken or replace
those controls. Remote providers are separate trust boundaries and must not
receive command or prompt context without the existing feature/user consent
and data-flow review.

## Invariants

1. **No auth bypass:** an offline state must not forge login, entitlement, or
   authorization success. Account-required features stay account-required.
2. **No secret leakage:** never put auth/provider credentials in TOML,
   command-line arguments, logs, telemetry, or checked-in environment files.
   Use platform secure storage or a reviewed secret provider.
3. **Visible failure:** unavailable service calls return an explicit,
   recoverable error. A failed remote write must not be presented as a
   successful save or sync.
4. **Local data safety:** do not delete, reset, or silently migrate cached
   credentials, local SQLite state, or user preferences to simulate offline
   operation.
5. **Explicit egress:** offline policy, if later added, must be enforced at
   transport boundaries and covered by request-capture tests. DNS failure is
   not proof that no request was attempted.
6. **Least data:** each remote operation should send only the data its
   documented contract requires; AI context requires particular scrutiny.
7. **Release integrity:** self-hosted update feeds require signed/verified
   artifacts, fork-owned keys, and an auditable release process.

## Threats to address before broad self-hosting

| Threat | Design response | Verification |
|---|---|---|
| Service outage blocks local shell | Keep PTY creation and input/render path independent; characterize with unreachable integration endpoints. | Local shell end-to-end test. |
| Failed cloud write loses local edits | Preserve local state and expose sync error/retry; never report success on failure. | Fake transport, persistence, retry, and conflict tests. |
| Token or prompt appears in logs | Review request/error logging and telemetry payloads; redact secrets and user content. | Secret-redaction tests and log inspection. |
| Endpoint override redirects a release client | Preserve release-channel override restrictions; do not make a release binary accept arbitrary roots by default. | Channel config tests and release build tests. |
| Replacement service weakens authorization | Specify and test auth/tenant/entitlement contracts before implementing a service. | Contract tests; explicit authorization negative cases. |
| Offline mode silently disables account checks | Model offline as transport availability only, not auth success. | Tests for unavailable API and denied/expired credentials. |

## Incident and rollback

Any future transport or auth change must be independently reversible. Keep
local PTY and local-state behavior separate from the remote provider rollout.
If a replacement service is unavailable, disable that remote capability with a
visible error; do not redirect to production or fall back to a success-shaped
stub.
