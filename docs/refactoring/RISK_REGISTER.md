# Risk register

Ratings are preliminary assessment priorities, not claims that an incident or
vulnerability exists. Reassess after runtime/network testing and before any
behavioral change.

| ID | Risk | Likelihood / impact | Evidence and trigger | Mitigation / verification | Rollback or contingency |
|---|---|---|---|---|---|
| R-01 | OSS source build depends on generated channel configuration that may not be available to external contributors. | High / High | Presubmit failed on missing `local_config.json`; installer pins `warp-channel-config` over SSH. | Publish a supported OSS config path and clean-checkout build test without secrets or proprietary behavior. | Keep current channel builds untouched; revert only the OSS build-path change. |
| R-02 | Offline work is blocked or loses state when remote startup/sync paths fail. | Medium / High | Local PTY and local persistence exist alongside auth/server/sync initialization; no general offline contract was demonstrated. | Add unreachable-service characterization, then policy tests and state durability tests. | Test-only PR first; feature-gate later behavior. |
| R-03 | Incorrectly removing auth gates or emulating a proprietary API could expose protected features or misrepresent results. | Medium / High | Auth and server API are separate from local shell; self-host replacements are not evidenced. | Preserve auth checks; explicit provider/service interfaces and visible unavailable errors. | Keep Warp adapter/default authorization; revert alternate adapters independently. |
| R-04 | Configuration/backend changes can overwrite user settings or session state. | Medium / High | Native/TOML settings selection, SQLite restoration and cloud settings sync have distinct behavior. | Isolated-profile migrations, round-trip tests, backups, compatibility/versioning. | Leave existing stores/formats intact; migration behind explicit version gate. |
| R-05 | Outbound telemetry, crash reporting, AI context, or sync may be unclear to fork users. | Medium / High | Collector and remote paths exist; runtime request trace was not performed. | Endpoint/operation inventory, request-capture tests, opt-out/offline assertions, data disclosure. | Keep current behavior until privacy review approves a change; restore prior policy if tests fail. |
| R-06 | Shell process, remote command, or local IPC boundary regressions affect command execution or local trust. | Low–Medium / High | PTY spawner, remote executor, and native socket IPC are security-sensitive surfaces; no exploit confirmed. | Scoped threat model and platform-specific regression tests before refactoring. | Surgical fixes only; avoid broad process model replacement. |
| R-07 | Preview/source comparisons are mistaken for verified behavioral parity. | High / Medium | Historical app was statically inspected only; source build was blocked; no matching source commit established. | Label all unobserved behavior unknown; compare isolated runtimes after build setup. | Withdraw unverified conclusions and retain raw metadata evidence only. |
| R-08 | Performance work optimizes the wrong hot path or regresses terminal rendering. | Medium / Medium | No CPU/allocation/memory profile or comparable baseline exists. | Fixed workload, profile before change, regression tests and same-hardware before/after metrics. | Revert each isolated optimization independently. |
| R-09 | Fork update/signing infrastructure can install untrusted or incompatible builds. | Low–Medium / High | Preview is signed/notarized; current source includes channel configuration; fork-owned release process is not established. | Separate identities/feeds, signature verification, rollback and staging validation. | Disable update distribution only through explicit user-visible policy; provide manual signed releases. |
| R-10 | Full baseline results may be overclaimed despite missing setup and remote test preconditions. | High / Medium | Full presubmit stopped at missing config; workspace nextest only partially ran; four SSH tests failed. | Keep exact commands/results in report; distinguish targeted success from full-suite status. | Rerun after dependency/network prerequisites are addressed; supersede this baseline explicitly. |

## Escalation criteria

Pause implementation and request security/privacy review if a proposed change
touches auth enforcement, credential storage, command spawning, shell
environment, IPC peer trust, telemetry payloads, AI prompt/context transmission,
remote sync conflict handling, or update signature validation.
