# Gap analysis

## Method

Findings below describe evidence-backed architecture or validation gaps, not
confirmed exploitable vulnerabilities. The review did not claim dead code,
unreachable feature combinations, or duplicate logic without a call-site and
feature-gate audit. Severity expresses potential user/maintenance impact if the
gap remains; confidence expresses confidence in the evidence and gap.

## Findings

### GAP-001 — Source app build depends on generated channel configuration

- **Location:** `app/src/bin/channel_config.rs:30–34`,
  `app/src/bin/local.rs:11`, `script/install_channel_config`.
- **Evidence:** The app binary uses `include_str!` for channel JSON under
  `OUT_DIR`. The untouched `./script/presubmit` attempt failed because
  `target/debug/build/cognix-*/out/local_config.json` was absent. The installer
  pins `cognix-channel-config` revision
  `97a199b5bba11af9c7d3bd43dfb3669316d1f0f8` and checks SSH access.
- **Reproduction:** On this checkout, run the pinned-toolchain presubmit
  without the generated local config; compilation of bin `cognix` fails at the
  `include_str!`.
- **Severity / confidence:** High / high for build reproducibility; no claim
  that the upstream CI build is broken.
- **User impact:** A contributor may be unable to build the app executable
  from source without access to a separate repository/tool.
- **Recommended change:** Document the exact setup and provenance; establish a
  supported, non-secret OSS config generation path for OSS builds. Do not
  invent auth checks, credentials, or proprietary-service replacements.
- **Regression tests:** Clean checkout bootstrap/build test; assert required
  channel config is generated before compilation and fails with a clear
  diagnostic when intentionally absent.
- **Complexity:** Medium. **Dependencies:** Maintainer decision on public
  channel config and build policy. **Rollback:** Retain existing installer and
  gate any new OSS path to the OSS target.

### GAP-002 — Offline behavior is not an explicit, tested contract

- **Location:** `app/src/lib.rs:960–1050`, `app/src/terminal/local_tty/`,
  `app/src/server/server_api.rs`, `app/src/settings/cloud_preferences_syncer.rs`.
- **Evidence:** App initialization registers local persistence alongside auth
  and server API services; a separate local PTY path exists. No explicit
  network-free policy or offline-mode test was established by this review.
  Constructing a client does not prove a request is made on every launch.
- **Reproduction:** No regression was reproduced. The gap is confirmed by the
  absence of a demonstrated offline acceptance test in the reviewed baseline;
  validate with an unreachable local endpoint and temporary profile.
- **Severity / confidence:** Medium / medium-high.
- **User impact:** Users and fork maintainers cannot rely on documented,
  measured behavior when service endpoints are unavailable.
- **Recommended change:** First add a no-behavior-change integration
  characterization test: local terminal works with server unreachable;
  account-dependent features show explicit unavailable/error states.
  Implement a central offline policy only if that test demonstrates it is
  needed.
- **Regression tests:** No network requests from a fresh local-terminal flow;
  local shell command succeeds; cloud/AI/auth workflows fail visibly and
  recover when connectivity returns.
- **Complexity:** Medium. **Dependencies:** Deterministic service endpoint
  injection and isolated integration test HOME. **Rollback:** Test-only first;
  any later policy can be feature-gated.

### GAP-003 — AI access is coupled to the Cognix server API in the inspected path

- **Location:** `app/src/lib.rs:1028–1030`,
  `app/src/server/server_api.rs:1339`, `app/Cargo.toml` (`ai` dependency).
- **Evidence:** The app obtains its AI client from `ServerApiProvider`. The
  inspected source does not establish a general provider interface or
  configurable local/private inference endpoint. `local_fs` on the AI crate
  does not prove inference is local.
- **Reproduction:** Static path trace from app initialization to
  `ServerApiProvider`; no external-provider behavior test was performed.
- **Severity / confidence:** Medium / high for the inspected path, not a claim
  that no other provider exists anywhere in the repository.
- **User impact:** A fork may have to replace Cognix-specific AI API semantics
  before using an alternate provider.
- **Recommended change:** Inventory the AI call surface and data contracts;
  add a provider boundary only around demonstrated coupling. Keep provider
  credentials in secure storage and make local/private endpoints explicit.
- **Regression tests:** Contract tests for request mapping, errors, streaming,
  cancellation, no credential logging, and local-endpoint connectivity.
- **Complexity:** High. **Dependencies:** Product policy for compatible
  endpoints, privacy/data handling, and replacement for proprietary operations.
  **Rollback:** Keep the existing provider adapter as default; make alternate
  providers opt-in.

### GAP-004 — Local persistence, user settings, secrets, and cloud sync are
not one capability boundary

- **Location:** `app/src/settings/init.rs:291–329`,
  `app/src/settings/cloud_preferences_syncer.rs:70–112,219–231`,
  `app/src/persistence/mod.rs:73–84`,
  `app/src/persistence/sqlite.rs:144–160,416–469`.
- **Evidence:** Public settings may use a TOML file only under the
  `SettingsFile` flag; otherwise platform-native storage is used. Private
  settings use a platform-native backend. Session restoration uses local
  SQLite under `local_fs`; cloud preferences sync is a separate path. Auth
  credentials use secure storage.
- **Reproduction:** Trace initialization and backends; do not edit an actual
  user settings file as part of reproduction. A real offline/conflict case
  needs an isolated profile.
- **Severity / confidence:** Medium / high.
- **User impact:** Fork operators may confuse local persistence with
  cross-device sync and may not know where configuration, secrets, and runtime
  state live.
- **Recommended change:** Publish an as-built path/backend map, then define
  explicit config, secret, state, and log boundaries. Preserve migration and
  cloud-conflict semantics.
- **Regression tests:** Separate temporary directories and test fixtures;
  local settings survive restarts; cloud sync absence does not overwrite local
  values; corrupt/missing file behavior is explicit.
- **Complexity:** Medium. **Dependencies:** Channel AppId/path policy and data
  migration review. **Rollback:** Documentation and tests first; preserve
  existing storage formats until migrations are validated.

### GAP-005 — Telemetry and outbound service behavior need a testable inventory

- **Location:** `app/src/lib.rs:1412–1474,1848–1904`,
  `app/src/server/telemetry/collector.rs:47–75,99–132`,
  `app/src/server/telemetry/mod.rs:243–250,308–335`,
  `app/Cargo.toml` default feature list.
- **Evidence:** A telemetry collector is registered; collection/flush is
  privacy-gated, with remote sending or local telemetry-file persistence
  paths. The app has GraphQL, HTTP, AI, auth, and cloud sync routes. The review
  did not enumerate every request trigger or inspect a live network trace.
- **Reproduction:** Not a confirmed unwanted request. Use a proxy/blocked
  endpoint and explicit privacy settings to observe actual calls.
- **Severity / confidence:** Medium / high for the relevant code paths; low
  for any unobserved runtime request claim.
- **User impact:** Operators cannot yet audit all egress or prove a privacy
  setting/offline policy at the process boundary.
- **Recommended change:** Maintain an endpoint/operation inventory with code
  owner, trigger, data class, auth requirement, and offline fallback. Test
  telemetry opt-out at the request boundary.
- **Regression tests:** Fake HTTP/GraphQL transports assert allowed/blocked
  request sets in online, telemetry-disabled, and offline modes; shutdown
  flush has bounded failure behavior.
- **Complexity:** Medium. **Dependencies:** Central transport instrumentation
  or test doubles; privacy review. **Rollback:** Test-only instrumentation
  first; no change to default collection without explicit product decision.

### GAP-006 — Shell/process security boundaries merit focused verification

- **Location:** `app/src/terminal/local_tty/spawner.rs:124–252`,
  `app/src/terminal/local_tty/event_loop.rs:138–208,276–316,434–456`,
  `app/src/terminal/model/session/command_executor/remote_command_executor.rs:14–90`,
  `crates/ipc/src/native.rs:11–48`.
- **Evidence:** The application spawns local PTY/shell processes, supports a
  separate remote command path, and has native local-socket IPC. This is an
  inventory of security-sensitive boundaries, not evidence of an exploit.
- **Reproduction:** No vulnerability reproduction was performed or confirmed.
  Review process argument/environment construction, permission boundaries,
  socket ownership/authentication, and error cleanup; add fuzz/property tests
  only for parsers/protocols with a concrete threat model.
- **Severity / confidence:** Medium / medium.
- **User impact:** A boundary flaw could affect command execution, local
  process isolation, or IPC integrity; no such flaw is asserted here.
- **Recommended change:** Conduct a scoped threat-model review before
  modifying shell/IPC behavior; document invariants and test them.
- **Regression tests:** Argument/environment quoting, cancellation/exit
  cleanup, invalid IPC message handling, and socket permissions/peer
  validation on supported platforms.
- **Complexity:** Medium–high. **Dependencies:** Security review and
  platform-specific test environments. **Rollback:** Narrow tests/documented
  guards; avoid changing execution semantics absent a verified defect.

## Not established

- No vulnerability is confirmed by this report.
- No crate or feature is called dead, redundant, or unreachable.
- No broad allocation, blocking-on-async, clone, or hot-path issue was
  confirmed; those require profiling and call-site-level inspection.
- No Preview/source behavioral difference was established.
- Existing specs include prior offline settings-edit work
  (`specs/danielpeng/QUALITY-474/PRODUCT.md` and `TECH.md`); it concerns
  conflicts in settings sync and is not proof that the app has a general
  offline mode.
