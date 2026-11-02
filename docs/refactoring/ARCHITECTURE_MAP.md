# Architecture map

## Scope and evidence

This map describes the checked-out source at commit
`0dbd3d567a2090874c08a6cc91e091aebd933847`, not the historical Preview source.
Paths below are checked against this repository. Where a feature gate,
construction path, or client exists, that is not proof it is used or makes a
network request on every launch.

## Crate and module graph

```text
channel binaries (app/src/bin/*)
  -> warp::run() (app/src/lib.rs)
      -> warpui / warpui_core application runtime
      -> app orchestration: terminal, workspace, settings, auth, server, AI
          -> warp_core, settings, persistence, warp_terminal
          -> warpui_extras, editor, graphql, http_client, websocket
          -> firebase, ai, warp_server_client
          -> ipc (platform-specific local IPC and plugin-host paths)

local terminal UI (app/src/terminal)
  -> TerminalView / TerminalModel / grid renderer / Input
  -> local_tty: PtySpawner -> PTY/EventLoop -> local shell process
  -> remote terminal and SSH/WSL paths where configured

account and remote features
  -> AuthState / secure storage
  -> ServerApiProvider / GraphQL and HTTP clients
  -> AI client, cloud objects/sync queue, settings sync, telemetry
```

This is a relevant-path graph, not an exhaustive Cargo dependency graph.
`app/Cargo.toml` directly depends on the named workspace crates; the app is the
orchestration layer. `crates/warpui` and `crates/warpui_core` are the UI runtime
and core framework, `crates/warp_core` contains shared core/platform utilities,
`crates/warp_terminal` supplies shared terminal types, `crates/editor` is the
editor crate (`warp_editor` package), `crates/graphql` is the GraphQL client,
and `crates/ipc` provides local IPC primitives.

The requested `crates/warp_tui/` path is not present in the checked-out
workspace; the actual UI crates are `crates/warpui/`, `crates/warpui_core/`,
and `crates/warpui_extras/`.

## Ownership map

| Area | Primary owner | Evidence / boundary |
|---|---|---|
| App entry points and lifecycle | `app/src/bin/*.rs`, `app/src/lib.rs` | Channel binaries set channel state and call `warp::run()`. `run`, `run_internal`, `initialize_app`, `launch`, and shutdown callback coordinate startup and teardown. |
| Terminal UI, input, and rendering | `app/src/terminal/` | `TerminalView`, `TerminalModel`, `Input`, grid renderer, and local/remote TTY modules are declared in `terminal/mod.rs`. |
| Local PTY and shell process | `app/src/terminal/local_tty/` | `PtySpawner` owns PTY setup; `EventLoop` reads/writes PTY and updates terminal state. |
| Shared terminal abstractions | `crates/warp_terminal/` | Direct app dependency for shared shell/terminal types; it is not the whole terminal implementation. |
| UI framework | `crates/warpui/`, `crates/warpui_core/`, `crates/warpui_extras/` | App builder, views, event/runtime abstractions, preferences and secure storage integrations. |
| Settings | `app/src/settings/`, `crates/settings/`, `crates/settings_value/` | Public/private preference initialization, runtime settings, optional TOML file backend, cloud preference sync. |
| Local persistence | `app/src/persistence/`, `crates/persistence/` | App persistence facade; `local_fs` selects SQLite implementation and session restoration. |
| Authentication | `app/src/auth/`, `crates/firebase/`, secure storage | Auth state loads persisted credential material and handles Firebase token state. |
| Remote API and sync | `app/src/server/`, `crates/graphql/`, `crates/http_client/`, `crates/websocket/` | Server API construction, GraphQL/HTTP operations, cloud objects and sync queue. |
| AI | `app/src/ai/`, `crates/ai/`, `app/src/server/server_api.rs` | AI client is obtained from the server API provider; the local filesystem feature alone does not establish local inference. |
| Telemetry and diagnostics | `app/src/server/telemetry/`, `crates/warp_logging/` | Collector, privacy gates, queueing/sending or local telemetry-file persistence, and shutdown flushing. |
| IPC | `crates/ipc/`, app platform/plugin callers | Native local sockets; distinct from cloud GraphQL/HTTP traffic. |
| Build/channel setup | `app/build.rs`, `app/src/bin/channel_config.rs`, `script/install_channel_config` | Generated channel configuration is needed by app binaries; installer pins a separate SSH Git dependency. |

## Startup and shutdown

1. Channel wrapper selects `ChannelState` and calls `warp::run()`.
2. `run()` initializes platform/feature flags, parses args, applies permitted
   server URL overrides, and dispatches CLI/worker commands before GUI startup.
3. `run_internal()` sets up logging/tracing, profiling and resource limits;
   supported builds create the PTY spawner when `local_tty` is enabled.
4. `AppBuilder::run` registers preferences and calls `initialize_app`.
5. Initialization registers secure storage and settings, auth, server API,
   SQLite persistence (when `local_fs` is enabled), application models, UI and
   telemetry.
6. `launch()` attempts session/window restoration, processes supplied URLs,
   and opens a window if necessary.
7. Shutdown closes notebooks, stops the SQLite writer, flushes telemetry,
   terminates LSPs, prepares the PTY spawner, tears down services/profiling,
   and uninitializes crash reporting when enabled.

Primary evidence: `app/src/lib.rs` (`run`: 495–650,
`run_internal`: 661–820, `initialize_app`: 960–1050 and 1412–1474,
`launch`: 2214–2244, shutdown: 1848–1904).

## Local and remote dependencies

| Capability | Observed source path | Classification at this stage |
|---|---|---|
| Local shell/PTy | `app/src/terminal/local_tty/{spawner,event_loop,shell}.rs` | Local; independent execution path exists. |
| Preferences | `app/src/settings/init.rs` | Local storage exists; public settings use TOML only when `SettingsFile` is enabled, otherwise platform-native storage; private settings use platform-native storage. |
| Session restoration | `app/src/persistence/{mod,sqlite}.rs` | Local SQLite path under the Warp state directory when `local_fs` is enabled. Persistence failure is documented to degrade restoration. |
| Authentication | `app/src/auth/`, secure storage | Account/service-dependent. Local PTY existence does not imply account access. |
| AI | `app/src/server/server_api.rs`, `app/src/ai/` | AI client is server-provider based in the inspected path; local inference is not established. |
| Cloud object/settings synchronization | `app/src/server/cloud_objects/`, `app/src/settings/cloud_preferences_syncer.rs` | Remote service path, separate from local persistence. |
| Telemetry | `app/src/server/telemetry/` | Privacy-gated collection can be sent remotely or persisted locally; exact per-build runtime behavior is configuration-dependent. |
| IPC | `crates/ipc/` and platform/plugin callers | Local socket path, not inherently remote networking. |
| SSH/remote TTY | `app/src/remote_server/`, remote command executors | Optional remote terminal path; not a prerequisite for local shell use in source. |

`SERVER_ROOT_URL` and `WS_SERVER_URL` can target a local Warp server in a
documented feature configuration (`WARP.md`), but that is not an explicit
network-free mode. The service client can be constructed at startup without
that proving every service performs a request at startup.

## Behaviors to preserve

- Local shell creation, PTY read/write, resize and exit handling.
- Terminal parsing, rendering, keyboard/mouse input, selection, clipboard,
  shell integration, prompt and command-block behavior.
- Session/window restoration and local settings persistence.
- Existing keyboard shortcuts and command workflows, to be characterized
  against the source-built app before refactoring.
- Explicit authentication boundaries for account-protected operations.
- Graceful, visible failure of cloud, AI, telemetry, update, and remote-shell
  functions when their services are unavailable.
- Existing platform-specific startup, shutdown, accessibility, and packaging
  behavior.

## Limits

No exhaustive per-module call graph, allocator profile, hot-path allocation
profile, service-request trace, or Preview runtime trace was produced. Those
are validation items in the accompanying plan, not implied findings.
