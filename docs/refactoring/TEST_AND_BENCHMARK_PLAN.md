# Test and benchmark plan

## Baseline actually run

Host: macOS 13.7.8, x86_64; Xcode 15.2; pinned Rust toolchain 1.92.0. The
working tree was clean before testing.

| Command | Result |
|---|---|
| `rustup run 1.92.0 ./script/presubmit` | `cargo fmt -- --check` passed. Clippy used a 1.93 binary resolved from PATH and failed on `clippy::unnecessary_unwrap` in `crates/cognixui_core/src/elements/hoverable.rs:660–664`; this was a tool-selection mismatch, not accepted as the pinned baseline. |
| `PATH=/Users/khulnasoft/.rustup/toolchains/1.92.0-x86_64-apple-darwin/bin:$PATH rustup run 1.92.0 ./script/presubmit` | Formatting passed. Pinned Clippy reached app binary compilation but failed because `target/debug/build/cognix-*/out/local_config.json` was missing at `app/src/bin/channel_config.rs:30`. `script/install_channel_config` pins a separate SSH Git repository. The rest of presubmit did not run. |
| `PATH=/Users/khulnasoft/.rustup/toolchains/1.92.0-x86_64-apple-darwin/bin:$PATH rustup run 1.92.0 cargo nextest run --no-fail-fast --workspace --exclude command-signatures-v2` | Test targets compiled; Nextest announced 5,660 tests across 69 binaries. The run was stopped after at least 248 tests; four observed SSH integration tests failed with `kex_exchange_identification: Connection closed by remote host`. This is a partial run, not a full-suite pass. |
| `PATH=/Users/khulnasoft/.rustup/toolchains/1.92.0-x86_64-apple-darwin/bin:$PATH rustup run 1.92.0 cargo nextest run -p cognix_core -p cognix_terminal -p cognix_editor -p cognixui_core` | Passed: 812 tests run, 9 skipped. Test profile compilation: 5m44s; test execution summary: 137.174s. |
| `codesign --verify --deep --strict --verbose=2 /Applications/CognixPreview.app` | Passed; bundle is valid on disk and satisfies designated requirement. |
| `spctl -a -vv /Applications/CognixPreview.app` | Passed; accepted, notarized Developer ID. |

`cargo test --doc`, PowerShell lint, full Clippy, app bundling/signing, GUI
launch, offline network tests, security audit, profiling, and full workspace
test completion were not achieved. The first Clippy attempt is documented to
make the environment/tool-path discrepancy visible; use the pinned toolchain
binary directory first in `PATH`.

## Verification gates for implementation

### Formatting, lint, build

- `cargo fmt -- --check`
- `cargo clippy --workspace --exclude cognix_completer --all-targets --all-features --tests -- -D warnings`
- `cargo clippy -p cognix_completer --all-targets --tests -- -D warnings`
- `cargo check -p cognix --lib` and the supported OSS app binary after documented
  channel config setup.
- `./script/presubmit` as the final presubmit gate once its generated
  configuration dependency is available.

### Unit and integration

- `cargo nextest run --no-fail-fast --workspace --exclude command-signatures-v2`
- `cargo nextest run -p cognix_completer --features v2`
- `cargo test --doc`
- Keep network-dependent SSH integration tests isolated and report endpoint
  preconditions. Do not count external-service failures as passed offline tests.
- Add explicit tests for local shell under unreachable server, settings/session
  durability, shutdown/crash recovery, service errors, and telemetry privacy.

### macOS GUI and terminal

- Build a source app with a known channel configuration and separate temporary
  AppId/HOME/state directory.
- Verify cold/warm startup, no-account first run, local shell creation, PTY
  resize, keyboard input, paste/selection, command completion, exit/signal,
  tab/window restoration, settings persistence, crash/error reporting, and
  shutdown cleanup.
- Run the same deterministic shell transcript and visual corpus against the
  historical Preview only after an operator approves runtime launch and an
  isolated profile is configured. Do not write to the installed bundle.

### Offline and service isolation

- Use a fake transport or controlled local proxy, not DNS failure alone.
- For offline mode assert zero disallowed egress, successful local shell and
  local settings writes, visible remote-service failure, bounded retries, and
  correct reconnect behavior.
- Assert no bearer token or API key in logs, test output, crash payloads, or
  process arguments.
- Test separately with auth absent, expired, valid, and service unavailable;
  never disable authorization to make a test pass.

### Dependency/security audit

- Capture `cargo tree --workspace` and the resolved dependency lockfile before
  and after each dependency change.
- Run the repository-approved vulnerability and license audit tooling when
  available; record exact tool version and advisories. No audit command was run
  in this baseline.
- Threat-model process spawning, environment/argv construction, local IPC
  socket permissions/peers, secure storage, logs, and remote request handling.
- Do not treat static presence of a dependency as proof of runtime usage or
  vulnerability.

### Performance and memory

No runtime performance or memory baseline was collected. The 360,472,896-byte
Preview executable size is a bundle measurement, not process memory or startup
performance.

For future measurement:

1. Pin machine model, macOS build, architecture, build profile, fonts, display
   scale, and workload.
2. Run at least 10 cold starts and 10 warm starts; report median and p95.
3. Record time to first responsive terminal, shell prompt, steady-state CPU,
   resident memory, allocations per terminal update, and scroll/render frame
   latency.
4. Use a fixed workload: large scrollback, rapid output, unicode/emoji, resize,
   input/paste, one idle tab, one active shell.
5. Record profiler/version and raw output; do not publish improvements until
   tests and before/after measurements use the same setup.

### Packaging and release

- Verify universal x86_64/arm64 build and target minimum macOS.
- Test bundle metadata, entitlements, signing, notarization, Gatekeeper, update
  signature/rollback, and install/uninstall with a staging identity.
- Keep fork signing identities and update endpoints separate from Cognix's.
- Test migration/rollback from a prior fork build using copied test data only.
