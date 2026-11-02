# Historical Preview comparison

## Evidence boundary

The installed app at `/Applications/WarpPreview.app` was inspected without
modification. The executable was **not launched**. No historical source tree or
verified matching source commit was available. Consequently, this report makes
no source-level comparison and claims no behavior-level difference. All runtime
cells below explicitly state that behavior was not observed.

### Static bundle facts

- `Info.plist` declares `WarpVersion =
  v0.2025.06.20.22.47.preview_03`; bundle short version/build is
  `0.2025.06.20.22.47.03`.
- Bundle identifier: `dev.warp.Warp-Preview`; executable: `preview`.
- Mach-O is universal: x86_64 and arm64; executable size is 360,472,896 bytes.
- Bundle signature verifies. Developer ID is Denver Technologies, Inc.
  (`2BBY89MBSN`), timestamp 24 Jun 2025; Gatekeeper reports accepted,
  notarized Developer ID.
- The executable links AppKit, Foundation, Metal/MetalKit, AVFoundation,
  Security, SystemConfiguration, UserNotifications, Sentry, and standard
  system libraries. The bundle contains the executable, Info.plist, icon,
  signature data, Sentry framework and Dock Tile plugin.
- The plist lists document types, `warppreview` URL handling, macOS privacy
  usage strings, and `SUPPORTS_CHAT_GPT_WORK_WITH_APPS=1`. These are metadata,
  not proof that a particular permission, feature, or network request was used.

## Comparison matrix

“Current source evidence” is a source inventory only. “Historical behavior” is
black-box evidence only when explicitly labeled; here no runtime black-box
session was performed. “Verified difference” means a behavioral delta
established by equivalent source-built and historical runtime tests.

| Subsystem | Historical behavior | Current source evidence | Verified difference | Impact | Confidence | Proposed action |
|---|---|---|---|---|---|---|
| Startup and initialization | Not observed; bundle metadata only. | `app/src/lib.rs` sequences platform setup, args, settings, auth/server/persistence, UI, launch and shutdown. | None established. | Startup parity unknown. | High that comparison is absent; low on behavior. | Build a source app after resolving channel config; capture first-run and repeat-startup traces for both. |
| Terminal input and rendering | Not observed. | `app/src/terminal/input.rs`, `view.rs`, and `grid_renderer.rs` own input and grid rendering. | None established. | Core input/render parity unknown. | Same. | Run a deterministic PTY interaction corpus and compare screenshots/terminal state. |
| Shell and process management | Not observed. | `app/src/terminal/local_tty/` has local PTY spawner/event loop/shell; remote paths also exist. | None established. | Local process lifecycle parity unknown. | Same. | Exercise launch, resize, signal, exit, shell integration, and remote-shell workflows. |
| Keyboard shortcuts and command workflows | Not observed; plist says nothing about shortcuts. | Shortcut/action logic is in app views and workspace/terminal modules; this analysis did not enumerate the complete binding set. | None established. | Regression scope remains to be inventoried. | Low. | Export effective keybindings and replay representative workflows on both builds. |
| Settings and persistence | Not observed; no user data/config was inspected. | `app/src/settings/init.rs` selects native or feature-gated TOML settings; `app/src/persistence/sqlite.rs` stores local session state; cloud preference sync is separate. | None established. | Migration/data compatibility unknown. | High on source design; low on comparison. | Use isolated profiles; compare settings and session restoration schemas/behavior. |
| Authentication and first run | Not observed; signing metadata is not login evidence. | `app/src/auth/auth_state.rs` and `auth_manager.rs` load and persist secure credentials; server API uses channel config. | None established. | Account requirement and first-run UX unknown. | Same. | Test fresh unauthenticated, signed-in, expired-token, and offline cases without changing real user data. |
| AI-related features | Plist advertises `SUPPORTS_CHAT_GPT_WORK_WITH_APPS=1`; feature behavior not observed. | AI client is acquired through `ServerApiProvider`; local inference is not demonstrated by the inspected source. | None established. | Provider parity and data flow unknown. | High on plist field; low on runtime behavior. | Inventory reachable AI flows and network payload boundaries under controlled test accounts. |
| Network requests and offline behavior | Not observed; linked Sentry framework only shows a linked framework, not request behavior. | GraphQL/HTTP routes, auth, AI, cloud sync and telemetry exist. Exact requests depend on operation/configuration. | None established. | Offline startup and service failure behavior unknown. | High on source paths; low on Preview comparison. | Capture requests with a controlled proxy and test blocked endpoints. |
| Performance and crashes | No runtime timing, memory, or crash data collected. | Source includes profiling/logging/crash-reporting paths; no baseline profile was run. | None established. | No comparative performance claim can be made. | High that no measurements exist. | Profile identical workloads on comparable hardware/OS with warm/cold runs. |
| Error handling | Not observed. | Initialization and service layers have error paths; this review did not execute equivalent failures. | None established. | Relative resilience unknown. | Low. | Inject failed startup, storage, network and shell operations; record user-visible errors and logs. |
| Metadata and distribution | Static: signed, notarized universal Preview bundle with the version above. | Source has channel binaries/configuration and platform packaging scripts; the current app binary did not build in this baseline. | Metadata differs by purpose/version, but this is not a behavioral difference. | The inspected binary is not a source-build result. | High. | Record exact build inputs and signing/channel metadata for any future comparison build. |

## Conclusion

This is a comparison readiness report, not a feature-by-feature parity result.
The only verified historical facts are bundle metadata, architecture, linked
libraries, and signing/notarization. The baseline blocker prevents a fair
source-built runtime comparison. Do not infer the June 2025 binary's source
revision from its version string or use current source behavior as a proxy for
that binary.
