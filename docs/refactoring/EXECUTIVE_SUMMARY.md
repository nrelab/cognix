# Cognix local-first refactoring: executive summary

Assessment date: 2026-10-28

## Decision

The checked-out source has a real local terminal path: local PTY creation, shell
I/O, terminal parsing/rendering, local preferences, and SQLite-backed session
state are present. The same application also initializes or owns substantial
authentication, server API, AI, telemetry, and cloud-sync functionality. The
evidence does **not** establish that the full GUI starts or remains useful
offline, nor that the app can be self-hosted without replacing or configuring
Cognix-operated services.

The Preview binary is available for static inspection, but its runtime behavior
was not compared with a source-built app. A source build of the app executable
was blocked by a generated `local_config.json` missing from the build output;
the repository's installer pins a separate GitHub SSH dependency. The
historical binary was not launched. No source commit is attributed to it.

At the time of this baseline, no behavior-changing code was edited. The
follow-up work adds self-hosting decision/architecture documents and a
test-only name clarification; runtime behavior remains unchanged.

## Evidence snapshot

- Source commit: `0dbd3d567a2090874c08a6cc91e091aebd933847`.
- The worktree was clean before baseline commands.
- `rust-toolchain.toml` pins Rust 1.92.0. The installed default `rustc` and
  `cargo` resolve to Homebrew 1.93.1; the 1.92.0 rustup toolchain is installed.
- Host: macOS 13.7.8, x86_64, Xcode 15.2.
- Available build tools include Apple clang 15, CMake, Ninja, pkg-config,
  clang-format, `wgslfmt`, `cargo-nextest`, and `cargo-bundle`; Docker and
  `gcloud` are installed. `pwsh` is absent, so the non-CI presubmit script
  skips PowerShell analysis. These probes do not establish that every native
  SDK/toolchain component is installed.
- The repository has `README.md`, `CONTRIBUTING.md`, `COGNIX.md`, many Cargo
  manifests, and 216 PRODUCT/TECH spec files. No `AGENTS.md` or FAQ file was
  found in the repository or its parent search scope.
- Targeted spec review found prior work on offline settings-file conflict
  handling (`specs/danielpeng/QUALITY-474/`) and retryable network failures
  (`specs/APP-4014/`). These are narrower behaviors, not proof of a general
  offline mode.
- Historical app: `CognixVersion` is
  `v0.2025.06.20.22.47.preview_03`; the signed executable is a 360,472,896-byte
  universal x86_64/arm64 Mach-O. The bundle signature verifies and Gatekeeper
  reports a notarized Developer ID application.

## Baseline verification

| Command / inspection | Result |
|---|---|
| `cargo fmt -- --check` via `./script/presubmit` | Passed. |
| Pinned-toolchain `./script/presubmit` | Clippy/build stopped: `include_str!` could not find `target/debug/build/cognix-*/out/local_config.json` at `app/src/bin/channel_config.rs:30`; no app executable produced. |
| `cargo nextest run --no-fail-fast --workspace --exclude command-signatures-v2` | Compiled test targets and began 5,660 tests. Stopped after reaching at least 248/5,660; four observed SSH integration tests failed because their remote connection was closed. This is not a full-suite result. |
| `cargo nextest run -p cognix_core -p cognix_terminal -p cognix_editor -p cognixui_core` | Passed: 812 tests, 9 skipped. |
| Preview `codesign --verify --deep --strict` and `spctl -a -vv` | Signature valid; accepted as notarized Developer ID. Static inspection only. |

The `local_config.json` failure is a reproducibility dependency, not evidence
of a source-code defect. `script/install_channel_config` pins the separate
`cognix-channel-config` repository at
`97a199b5bba11af9c7d3bd43dfb3669316d1f0f8` and checks for SSH access. It was
not installed during this analysis.

## Follow-up result and next step

The existing integration harness now has a focused
`test_local_command_with_unreachable_services` characterization. It passed
with the Integration channel's configured HTTP, WebSocket, and Oz service
roots unavailable: a local shell bootstrapped and executed a harmless command.
The harness uses a synthetic test identity, so this does not prove real
unauthenticated OSS first-run behavior or assert a remote-only UI error.

No backend is justified for this local-shell workflow. Before adding a general
offline policy, characterize an explicit remote-only action and its visible
unavailable state, then test request suppression with a fake/captured
transport. Do not change auth checks or silently disable remote features.
See the follow-up [self-hosting decision](../self-hosting/DECISION_AND_SCOPE.md)
and [test report](../self-hosting/TEST_REPORT.md).

## Review boundary

The comparison and gap analysis distinguish source facts from hypotheses and
unknowns. No exploitable vulnerability, dead code, or Preview/source behavioral
difference is asserted without direct evidence. Review the detailed reports
before approving any P0–P2 implementation work.

## Report index

- [Architecture map](./ARCHITECTURE_MAP.md)
- [Preview comparison](./PREVIEW_COMPARISON.md)
- [Gap analysis](./GAP_ANALYSIS.md)
- [Self-hosting design](./SELF_HOSTING_DESIGN.md)
- [Refactoring roadmap](./REFACTORING_ROADMAP.md)
- [Test and benchmark plan](./TEST_AND_BENCHMARK_PLAN.md)
- [Risk register](./RISK_REGISTER.md)
- [Self-hosting decision and scope](../self-hosting/DECISION_AND_SCOPE.md)
