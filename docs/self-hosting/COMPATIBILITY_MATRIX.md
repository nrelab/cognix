# Self-hosting compatibility matrix

This matrix describes the checked-out source and the scope of the first
local-shell workflow. It does not compare the client with an unavailable
historical source commit or claim runtime parity with Cognix Preview.

| Subsystem / behavior | Current source evidence | Classification | Compatible without a new backend? | Remaining dependency / verification |
|---|---|---|---|---|
| Local shell creation and PTY lifecycle | `PtySpawner` starts local PTYs directly or through the local terminal-server subprocess. | **LOCAL** | Yes, for ordinary local shells. | Run shell lifecycle and integration tests on each supported OS. |
| Terminal input, output, parsing, and rendering | Local terminal model/UI and `cognix_terminal` path. | **LOCAL** | Yes. | Preserve interactive, resize, signal, and shell-exit regressions. |
| Local preferences/session persistence | Local settings backends and SQLite/session state. | **LOCAL** | Yes, for local state. | Verify migration/corruption/recovery; remote sync is separate. |
| Logged-out auth initialization | Auth state reads persisted secure-storage state; client construction alone is not a request. | **LOCAL** startup state; account features remain remote | Local shell does not need a real account in the tested integration workflow. | Real OSS first-run UI and any onboarding model-list calls need separate verification. |
| Account creation, login, refresh, account actions | AuthManager and ServerApi auth operations. | **REQUIRES REPLACEMENT** for independent account service | No for those account actions. | Identity, authorization, token, and entitlement contract is not replaced here. |
| GraphQL/server API | HTTP GraphQL operation transport at `{server_root}/graphql/v2?op=…`; operations are caller-driven. | **REQUIRES REPLACEMENT** for operations a fork keeps | Not applicable to local shell. | Enumerate required operations and schemas first. |
| AI | App AI operations use Cognix service calls; some public model listing may be requested during onboarding. | **REQUIRES REPLACEMENT** for non-Cognix AI service | No general provider compatibility established. | Provider API, streaming, cancellation, secrets, and prompt-data policy. |
| Cloud preferences and objects | Syncer waits for cloud state and is triggered after login/settings paths; local state remains separate. | **UNAVAILABLE WITHOUT PROPRIETARY SERVICE** | Local-only use can preserve local state, not cross-device sync. | Compatible storage/object API and conflict contract. |
| Telemetry | Gated event queue and remote send. | **OPTIONAL REMOTE** | Yes only under an explicit egress/privacy policy. | Startup may flush eligible events depending on build/config and queued state. |
| Crash reporting | Build/config/privacy-gated initialization. | **REQUIRES REPLACEMENT** for independent reporting | Not proven for every OSS binary. | Inspect effective build config; choose fork-owned collector or explicit opt-out. |
| Updates | Startup updater registration; remote check under feature/execution gates. | **REQUIRES REPLACEMENT** for a fork distribution | No independent release pipeline established. | Fork-owned signing, feed, artifact verification, and rollout. |
| Local-server development | `script/run` maps legacy `with_local_server` to channel-config environment; OSS release ignores URL overrides. | **REQUIRES REPLACEMENT** for public self-host targeting | Not via the current OSS binary. | Pinned channel generator access and a reviewed OSS dev configuration path. |
| Historical Preview behavior | Static bundle metadata/signature was inspected in earlier baseline; runtime comparison was not performed. | **UNKNOWN** | Not established. | Authorized source-built and Preview runtime comparisons are still needed. |

## First compatibility claim

Once the focused integration run passes, the supported claim is limited to:

> The existing integration test harness can create a local shell and execute
> a harmless command while its configured Cognix service endpoints are
> unavailable.

Do not broaden this to "Cognix is fully offline," "no startup request is ever
attempted," "the client is self-hostable," or "Preview behavior is preserved"
without separate request-capture and runtime evidence.
