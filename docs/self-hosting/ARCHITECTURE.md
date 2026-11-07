# Client architecture and service boundaries

This is a source-based map of the checked-out client, not a reconstruction of
Cognix Preview internals.

## Local terminal workflow

```text
app entry point / ChannelState
            |
            v
Cognix app initialization and workspace UI
            |
            v
Terminal pane / terminal model
            |
            v
PtySpawner
      |                 |
      | direct          | optional local terminal-server subprocess
      +--------+--------+
               v
         local PTY + shell process
               |
               v
      terminal input/output and rendering
```

`PtySpawner` owns local process creation in
[`app/src/terminal/local_tty/spawner.rs`](../../app/src/terminal/local_tty/spawner.rs).
Local terminal input, shell I/O, parsing, and rendering remain within the
client. The integration workflow `test_local_command_with_unreachable_services`
creates a bootstrapped
terminal and executes `execute_echo`; its test runner configures unreachable
service URLs in
[`crates/integration/src/bin/integration.rs`](../../crates/integration/src/bin/integration.rs).

## Startup and shutdown outline

1. The selected binary sets a `ChannelState` and channel configuration. The
   OSS entry point sets production Cognix service roots and does not provide
   telemetry, crash-reporting, or updater configuration by default.
2. `cognix::run` initializes app state, reads persisted auth state, constructs
   `ServerApiProvider`, registers auth and settings models, and initializes
   the UI. Constructing a client is not itself a network request.
3. Auth refresh is conditional on existing login state and a usable token;
   unauthenticated account creation is flow-driven.
4. Settings, local SQLite/session state, telemetry, crash reporting, and
   updater models initialize according to build, channel, privacy, and
   execution-mode gates.
5. When a terminal pane starts, `PtySpawner` creates the local shell path.
   Closing the pane/session propagates shutdown to its shell/PTY lifecycle.

The exact startup egress of a particular binary cannot be inferred from its
channel label alone. Effective build features, execution mode, privacy
settings, persisted telemetry events, and runtime overrides matter. See the
gated-service table below.

## Trust and network map

```text
                         +----------------------+
                         |  Cognix desktop client |
                         +----------+-----------+
                                    |
                +-------------------+-------------------+
                |                   |                   |
                v                   v                   v
          local PTY/shell     local settings,       configured service
          and terminal I/O    SQLite, secure store  roots in ChannelState
                                                        |
            +----------------+----------------+---------+----------+
            |                |                |                    |
            v                v                v                    v
       HTTP GraphQL       RTC/WebSocket     Oz/AI flows       telemetry,
       / server API       remote sessions   when invoked      crash and
       when invoked       when invoked                         updater gates
```

The diagram shows logical boundaries, not a complete list of every transport
or a guarantee that every listed request occurs at startup. The OSS channel
uses production Cognix configuration; the Integration channel sets its service
roots to the reserved unavailable address for tests. Telemetry, crash
reporting, and update paths have separate build, privacy, and execution gates.

## Ownership and locality

| Subsystem | Primary owner | Local behavior | Remote dependency / trigger |
|---|---|---|---|
| Terminal process and PTY | `app/src/terminal/local_tty/` | Starts and controls local shells; direct PTY path and optional local server process. | Remote SSH/shared-session modes are separate features. |
| Terminal rendering and input | `app/src/terminal/`, `crates/cognix_terminal/`, `crates/cognixui/` | Local input, terminal state, parsing, and rendering. | No Cognix account is needed for ordinary local shell input/output. |
| Auth/account | `app/src/auth/`, `app/src/server/server_api/auth.rs` | Reads stored auth state and local secure-storage credentials. | Login, anonymous-account creation, user fetch, and token refresh call service APIs when their flows run. |
| GraphQL and shared server client | `app/src/server/server_api.rs`, `crates/graphql/src/client.rs` | Builds shared clients and operation wrappers. | GraphQL operation sends are remote and operation-triggered. |
| Local settings/state | `app/src/settings/`, persistence modules | Platform preferences, local files, and SQLite/session state. | Cloud preference/object sync is a separate remote path. |
| AI | `app/src/ai/`, `app/src/server/server_api/ai.rs` | UI and request construction. | Authenticated operations and some public model-list requests call Cognix services when invoked. |
| Cloud sync/objects | `app/src/settings/cloud_preferences_syncer.rs`, `app/src/server/cloud_objects/` | Local settings and cached state remain local. | Sync waits for cloud-object load and is triggered by login/settings changes; polling stops while logged out. |
| Telemetry | `app/src/server/telemetry/` | Local queue and privacy decisions. | Remote flush requires eligible events, privacy enablement, and release/sandbox gates. |
| Crash reporting | `app/src/crash_reporting/` | Local initialization and consent/config checks. | Reporting is build-, config-, and privacy-gated. |
| Updates | `app/src/autoupdate/`, `app/src/server/server_api.rs` | Version comparison and UI. | Polling requires updater configuration/feature and permitted execution mode; manual checks are user-triggered. |

## Remote-call gates relevant to offline claims

| Boundary | Startup construction vs request | Logged-out OSS caveat |
|---|---|---|
| Auth/account | Auth state and API objects are constructed locally. Account creation and user fetch are explicit auth flows. | Merely initializing logged-out auth state does not create an account request. |
| GraphQL | The shared GraphQL transport posts to `{server_root}/graphql/v2?op=…` when an operation is invoked. | Client construction alone is not a GraphQL startup request. |
| AI | AI clients are initialized as part of app services. | Authenticated AI calls require an invoked feature. Some public model-list fetches can occur in onboarding. |
| Cloud sync | A preferences syncer is constructed during app startup. | Construction alone is not sync; normal triggers follow login/user fetch or settings changes. Cloud polling stops while logged out. |
| Telemetry | Collector setup can flush persisted events or schedule queue flushing. | A startup flush is possible if events exist and privacy/build gates permit it; logged-out status alone does not prevent it. |
| Crash reporting | Initialization runs only in applicable builds and checks configuration/privacy. | Effective reporting configuration cannot be concluded from OSS channel identity alone. |
| Updates | Updater registration is local. | A remote startup check is possible only when updater and execution-mode gates allow polling. |

## Configuration limitation

The OSS entry point uses `Channel::Oss` and production service configuration.
Release channels (`Stable`, `Preview`, `Oss`) ignore server URL overrides.
Internal `Local` and `Integration` channels have different configuration
behavior. `script/run` translates the legacy `with_local_server` flag into an
environment setting for `cognix-channel-config`; it is not a Cargo feature.
The installer pins a separate SSH Git dependency. A self-hosted service cannot
be connected by assuming that setting `SERVER_ROOT_URL` while running the OSS
binary will redirect it.

## Local vs remote data path

- Shell commands and terminal I/O: local PTY/process path.
- User preferences and session database: local persistence; cloud sync is a
  separate operation.
- Credentials: platform secure storage and auth flow; not application TOML.
- AI prompts, cloud objects, and account data: remote operations when invoked.
- Telemetry and crash data: governed by privacy/build/config gates; do not
  treat an unavailable server as proof that no egress was attempted.
- Update metadata and downloads: depend on updater configuration and release
  infrastructure.

For service operations and compatibility requirements, see
[API contract](./API_CONTRACT.md).
