# Development workflow for local-first characterization

## Toolchain and build configuration

The checked-out commit is `0dbd3d567a2090874c08a6cc91e091aebd933847`;
`rust-toolchain.toml` pins Rust 1.92.0. Prefer the pinned toolchain for
reproducible checks:

```sh
rustup run 1.92.0 cargo fmt -- --check
rustup run 1.92.0 cargo nextest run -p integration --test integration <test-name>
```

The current integration harness creates an isolated HOME and launches the
integration binary with the named test. It propagates `WARP_*` variables, so
clear service-root overrides if validating the harness's built-in unreachable
service configuration.

The focused shell test to run is named in
[`crates/integration/tests/integration/shell_integration_tests.rs`](../../crates/integration/tests/integration/shell_integration_tests.rs).
For manual iteration, the repository guide documents
`WARPUI_USE_REAL_DISPLAY_IN_INTEGRATION_TESTS=1 cargo run --bin integration -- <test-name>` in
[`crates/integration/tests/INTEGRATION_TESTING.md`](../../crates/integration/tests/INTEGRATION_TESTING.md).

## Local Warp server instructions

The previous `WARP.md` snippet used `cargo run --features
with_local_server`. That is stale: `with_local_server` is not a Cargo feature.
The compatibility wrapper accepts it:

```sh
./script/run --features with_local_server
```

`script/run` translates the legacy flag into `WITH_LOCAL_SERVER`, then selects
the internal Local channel when `warp-channel-config` is available. The script
first invokes `script/install_channel_config`, which pins a separate
`warp-channel-config` Git revision accessible through SSH. An external
contributor without access to that repository cannot assume this command will
fall back cleanly to the OSS binary.

The OSS binary sets `Channel::Oss`, production server configuration, and
release-channel URL-override restrictions. It is not a supported way to point
the shipped OSS channel at a self-hosted API. Do not infer the private channel
generator's complete environment-variable contract from the public source.
The `SERVER_ROOT_URL` and `WS_SERVER_URL` examples in the old WARP.md section
are removed pending verification against that pinned tool.

## Workflow test scope

The integration runner configures HTTP, RTC/WebSocket, and Oz service roots at
`192.0.2.0:9`, and disables telemetry and crash-reporting channel configs. The
test channel creates a synthetic test identity; it is not a real account
scenario. The focused test should establish that a local shell can bootstrap
and execute a harmless command even when Warp's configured remote services
are not reachable.

This does not prove that no startup request is attempted in every build:
telemetry can flush eligible persisted events and updater polling can be
enabled in other channel/build configurations. Use captured/fake transports
for any future explicit offline-mode guarantee.

## Baseline limitations

The previous baseline recorded:

- `cargo fmt -- --check`: passed.
- Focused `warp_core`, `warp_terminal`, `warp_editor`, and `warpui_core` tests:
  812 passed, 9 skipped.
- Full workspace nextest was stopped after at least 248 of 5,660 tests; four
  observed SSH integration tests failed because their remote connection was
  closed. This was not a full-suite result.
- Full app/presubmit was blocked because generated `local_config.json` was
  absent; the pinned channel-config generator requires a separate SSH repo.

See [test report](./TEST_REPORT.md) for the current focused run.
