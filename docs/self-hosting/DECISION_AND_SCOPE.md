# Self-hosting decision and scope

Assessment snapshot: commit `0dbd3d567a2090874c08a6cc91e091aebd933847`.

## Decision

Do not build or deploy a replacement Warp backend for the first local-first
workflow. The source has a local shell path: the app creates a local PTY (or
uses its local terminal-server subprocess), then reads and writes to the local
shell. A shell command does not need a Warp account, GraphQL service, AI
provider, or cloud-storage service. The focused integration workflow uses
intentionally unreachable Warp service endpoints and exercises a local shell.

This decision is intentionally narrow. It does **not** establish that the
entire OSS application is offline, self-hostable, or behaviorally equivalent
to Warp Preview. It does not replace authentication or authorize disabling
account checks. Account, AI, cloud sync, telemetry, crash reporting, and
update behavior have separate service boundaries and runtime/build gates.

## First workflow

1. Launch the integration application with a temporary home/state directory.
2. Use the `Integration` channel, whose default HTTP and WebSocket service
   roots are `192.0.2.0:9` (a reserved test address).
3. Wait for the terminal shell to bootstrap.
4. Execute the harmless `echo` command and verify the terminal output.

The integration channel also uses a synthetic test identity. It is not a
production logged-out OSS account, so this test proves local shell usability
when the configured remote services are unavailable, not every aspect of a
real unauthenticated first-run flow. The harness propagates `WARP_*`
environment variables; unset endpoint overrides when running this test if the
goal is to retain the built-in unavailable-service configuration.

## In scope

- Record verified client/service boundaries and the local-shell call path.
- Make the existing end-to-end shell test's unavailable-service intent
  explicit without changing runtime behavior.
- Correct the local-server development guidance that currently presents
  `with_local_server` as a direct Cargo feature.
- Define the contract and evidence needed before choosing any self-hosted
  remote capability.

## Out of scope

- Implementing a Warp-compatible service or claiming compatibility with
  undocumented/proprietary service behavior.
- Changing login, entitlement, or authorization checks.
- Adding a global offline switch or silently suppressing requests.
- Replacing AI, cloud storage/sync, authentication, telemetry, updates, or
  crash reporting.
- Claiming historical Preview behavior without runtime comparison.

## Decision gate for a backend

A backend is justified only after a named user-facing workflow requires remote
behavior and its operation-level contract is known. Before implementation,
identify each request and response shape, auth/entitlement requirements,
streaming and retry behavior, data classes, retention, privacy gates, and
compatibility tests. Until then, keep local shell operation independent and
classify the remote feature as requiring a replacement or unavailable without
the proprietary service.

## Document set

- [Architecture and service boundaries](./ARCHITECTURE.md)
- [Existing API contract and replacement boundary](./API_CONTRACT.md)
- [Security model](./SECURITY_MODEL.md)
- [Development workflow](./DEVELOPMENT.md)
- [Deployment decision](./DEPLOYMENT.md)
- [Test report](./TEST_REPORT.md)
- [Compatibility matrix](./COMPATIBILITY_MATRIX.md)
