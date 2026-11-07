# Refactoring roadmap

Each task should be a separate, reviewable PR. Start only after the baseline
and this architecture report are reviewed. Keep formatting-only changes out of
behavioral PRs.

Effort estimates are engineering estimates, not measured durations:
S = 1–2 days, M = 3–5 days, L = 1–2 weeks.

## P0 — Reproducible builds, crashes, data safety, security, terminal regressions

| ID / task | Acceptance criteria | Dependencies | Risk | Tests | Effort / PR size |
|---|---|---|---|---|---|
| P0-1: Make OSS app build setup reproducible | A clean contributor checkout can build the OSS app target using documented, accessible inputs; missing channel config gives a direct diagnostic. No auth/security bypass or secret is added. | Maintainer decision on `cognix-channel-config` provenance and which channel configuration is publishable. | High: channel metadata may encode service/auth semantics. Preserve existing non-OSS channels. | Clean macOS build in CI; test both config-present and config-absent paths. | M / one build-focused PR. |
| P0-2: Characterize local shell without services | The shell path works under unreachable service endpoints, and a remote-only action surfaces an explicit unavailable state. Current partial evidence: `test_local_command_with_unreachable_services` passes for shell creation/command execution; real logged-out startup and a remote-action error are not yet asserted. | Deterministic server endpoint injection and isolated test HOME. | Low if test-only; medium if it reveals startup coupling. | Focused shell integration test plus remote-action/UI assertion and local PTY lifecycle tests. | S–M / one test-only PR first. |
| P0-3: Protect local data and process lifecycle | Document and test persistence failure, interrupted shutdown, PTY cancellation/exit, and corrupted settings behavior; no silent state loss. | Existing migration and persistence contracts. | Medium: storage tests can encode behavior users rely on. | SQLite/persistence tests, settings migration tests, shell start/stop tests. | M / one subsystem per PR. |
| P0-4: Scope security review of shell, secrets, and IPC | Threat model lists trust boundaries and concrete invariants; any fix has a regression test and does not change shell semantics unnecessarily. | Security reviewer and supported-platform test environments. | High if broad behavior changes are mixed in. | Argument/env handling, socket permissions/peer identity, secret redaction, invalid message tests. | M / separate audit and surgical fix PRs. |

## P1 — Local-first startup, configuration isolation, backend boundaries, dependency reduction

| ID / task | Acceptance criteria | Dependencies | Risk | Tests | Effort / PR size |
|---|---|---|---|---|---|
| P1-1: Add explicit offline policy | Local terminal remains usable; a single policy prevents unauthorized network calls; remote actions show recoverable, visible errors. | P0-2 characterization and request inventory. | High: accidental auth bypass or unexpected request suppression. | Offline startup, feature action, reconnect, telemetry opt-out, request capture. | M / one policy and its tests. |
| P1-2: Publish config/secret/state/log map | Each setting/data class has a documented owner and current platform path/backend; no migration is performed by docs. | AppId and platform-path verification. | Low. | Documentation review; targeted path tests if paths change. | S / docs-only. |
| P1-3: Make network egress auditable | Every external operation has a code owner, trigger, auth/data class, privacy gate, timeout, and offline behavior. | Transport call inventory and privacy review. | Medium: comprehensive inventory may reveal implicit behavior. | Fake transport asserts request set by mode and privacy setting. | M / one inventory/test PR. |
| P1-4: Clarify service/provider ownership | App services depend on explicit domain interfaces only where existing seams do not suffice; no blanket service locator or speculative abstraction. | P1-3 inventory and maintainer review. | Medium: premature interfaces can add indirection. | Contract tests; unchanged default adapter behavior. | M / one boundary at a time. |
| P1-5: Reduce dependencies/features only with evidence | A dependency is removed only after checking callers, feature gates, platform builds, and tests; dependency diff is measurable. | `cargo tree` baseline and license/security audit. | Medium–high: transitive native dependencies/platform support. | Feature matrix build/tests and dependency-tree diff. | M / one dependency per PR. |

## P2 — Optional self-hosted AI, modularity, performance, advanced integrations

| ID / task | Acceptance criteria | Dependencies | Risk | Tests | Effort / PR size |
|---|---|---|---|---|---|
| P2-1: Add alternate AI provider | An opt-in compatible provider supports the operations actually used, secure credential storage, cancellation/streaming, and clear data-flow disclosure; Cognix adapter remains supported. | AI operation inventory and provider security/privacy decision. | High: prompts may contain sensitive terminal/repo context. | Contract and privacy tests; local/private endpoint smoke test. | L / separate provider PR. |
| P2-2: Improve module ownership | One demonstrated coupling is reduced without broad public API expansion; owners and dependencies are explicit. | P1-4. | Medium. | Existing behavior tests and dependency-direction check. | M per boundary / small PRs. |
| P2-3: Optimize measured hot paths | Reproducible profile identifies a specific CPU/alloc/memory issue; before/after numbers use the same workload and hardware; no terminal rendering regressions. | Stable benchmark corpus and profiler. | Medium: rendering and terminal behavior are sensitive. | Benchmarks plus rendering/PTY regression tests. | M per optimization. |
| P2-4: Self-host release and advanced integrations | Fork-owned signing, updates, service deployment and compatibility matrix are documented and testable; no Cognix service is silently impersonated. | P0-1 and service replacement decisions. | High: signing, update security, and data migration. | Signed staging build, update verification, integration tests against private services. | L / staged release track. |

## Ordering

The local-shell portion of P0-2 is now covered by a passing integration test
that adds no runtime behavior. The full P0-2 acceptance criteria still require
an explicit remote-action unavailable-state assertion. P0-1 is the build
reproducibility blocker for source-built GUI comparisons. Do not begin broad
provider or crate-boundary refactors until the remaining test contract and
operation-level service inventory are reviewed.
