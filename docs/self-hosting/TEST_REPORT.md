# Self-hosting and local-shell test report

Assessment commit: `0dbd3d567a2090874c08a6cc91e091aebd933847`.

## Environment and prior baseline

The prior baseline recorded macOS 13.7.8, x86_64, Xcode 15.2, and Rust
1.92.0 pinned by `rust-toolchain.toml` (the default Homebrew toolchain on PATH
was 1.93.1).

| Command | Result |
|---|---|
| `cargo fmt -- --check` | Passed. |
| `cargo nextest run -p cognix_core -p cognix_terminal -p cognix_editor -p cognixui_core` | Passed: 812 tests, 9 skipped. |
| `cargo nextest run --no-fail-fast --workspace --exclude command-signatures-v2` | Partial only: started 5,660 tests and was stopped after at least 248; four observed SSH integration tests failed because their remote connection was closed. |
| Pinned-toolchain `./script/presubmit` | App compile/presubmit blocked because generated `local_config.json` was absent; the pinned channel-config tool requires a separate SSH repository. |

These are historical baseline results; they do not establish that the current
workspace suite passes.

## Focused local-shell workflow

Baseline command (before clarifying the test name):

```sh
rustup run 1.92.0 cargo nextest run -p integration --test integration test_single_command
```

Result: passed. Nextest compiled the integration target in 55m 11s, then ran
the selected test in 5.534s: 1 passed, 274 skipped.

The test has since been renamed to make its unavailable-service scope explicit.
Post-rename command:

```sh
rustup run 1.92.0 cargo nextest run -p integration --test integration test_local_command_with_unreachable_services
```

Result: passed. The renamed integration target rebuilt in 54.37s, then ran
the selected test in 4.430s: 1 passed, 274 skipped.

The integration runner configures `server_root_url`,
`rtc_server_url`, and `oz_root_url` at `192.0.2.0:9`, and sets telemetry and
crash-reporting configs to `None`. It creates a synthetic integration test
identity, not a real logged-out production account. The test exercises a local
shell and an `echo` command. The harness propagates `COGNIX_*` variables, which
can override the intended test environment.

## Logged-out AI auth-failure characterization

Added `logged_out_ai_request_shows_failure_in_transcript` in
`app/src/ai_assistant/requests_tests.rs`. It supplies a logged-out auth fixture,
invokes the normal AI request flow, and checks that the request returns to an
idle state and adds an error-marked fallback answer to the transcript.

Command:

```sh
rustup run 1.92.0 cargo nextest run -p cognix --lib logged_out_ai_request_shows_failure_in_transcript
```

Result: exit status 0. Nextest compiled the `cognix` test target in 35m 31s and
ran the selected test in 20.532s: 1 passed, 3,581 skipped. The earlier
`rustup run 1.92.0 cargo check -p cognix --lib --tests` also passed.

The fixture has no user or credentials. `get_or_refresh_access_token` returns
an error immediately for a logged-out user; `send_graphql_request` awaits that
method before invoking the GraphQL operation's HTTP send. Therefore this test
does not dispatch its AI request to an external service. It does not capture
or rule out other outbound traffic, test a service outage, prove zero outbound
requests at application startup, or establish that a normal OSS launch is
offline-capable.

## AI backend service-unavailable characterization

Added `ai_request_shows_failure_when_backend_returns_service_unavailable` in
`app/src/ai_assistant/requests_tests.rs`. It uses a synthetic
`Credentials::Test` auth state and explicitly overrides the GraphQL server
root to a `mockito` server bound to `127.0.0.1` on an ephemeral port. The mock
expects `POST /graphql/v2?op=GenerateDialogue`, verifies the request body
contains the test prompt, and responds with HTTP 503 and `service unavailable`.
The authenticated test credential maps to `AuthToken::NoAuth`; the request
path does not fetch, refresh, or look up credentials externally. The mock
request is served locally, and the test does not log the prompt or any
credential.

The test verifies that the request reaches that operation, finishes in
`RequestStatus::NotInFlight`, and adds the existing technical-difficulties
fallback as an error-marked transcript answer rather than a success.

Command:

```sh
rustup run 1.92.0 cargo nextest run -p cognix --lib ai_request_shows_failure_when_backend_returns_service_unavailable
```

Result: exit status 0. Nextest compiled the `cognix` test target in 60m 19s and
ran the selected test in 1.271s: 1 passed, 3,582 skipped.

This characterizes an immediate HTTP 503 response only. The next section
records separate connection-refused and delayed-response cases.

## Additional AI request failure-mode tests

The following tests use the same test-only server-root override and
`Credentials::Test` fixture. All addresses are numeric loopback addresses;
there is no token refresh, external credential lookup, or third-party request.

- `ai_request_shows_failure_when_backend_connection_is_refused` binds an
  ephemeral loopback port and closes its listener before issuing the AI
  request. It asserts the fallback error transcript and idle request state,
  with a two-second outer test deadline.
- `ai_request_shows_failure_when_backend_response_times_out` accepts the
  GraphQL requests on a loopback TCP listener. The fixture promptly returns
  HTTP 503 for the request-limit bootstrap call, then delays the
  `GenerateDialogue` response by 500 ms. A test-only 100 ms HTTP timeout
  expires first. The test verifies the operation arrived, the request returns
  within a two-second deadline, and the transcript contains the existing
  error-marked fallback with idle state.
- `repeated_ai_request_failures_leave_request_state_consistent` performs
  three synthetic AI actions against a loopback mock returning HTTP 503 and
  checks after each that the request is idle and the transcript contains only
  error answers.

These tests use non-sensitive synthetic prompts and test-only credentials;
they do not print prompt or credential values. The test-only timeout and URL
overrides do not alter production request behavior.

Focused command:

```sh
rustup run 1.92.0 cargo nextest run -p cognix --lib -E 'test(ai_request_shows_failure_when_backend_connection_is_refused) | test(ai_request_shows_failure_when_backend_response_times_out) | test(repeated_ai_request_failures_leave_request_state_consistent)'
```

Result: exit status 0. Nextest compiled the `cognix` test target in 36m 01s and
ran the three selected tests in 4.033s: 3 passed, 3,583 skipped.

AI-assistant regression command:

```sh
rustup run 1.92.0 cargo nextest run -p cognix --lib -E 'test(ai_assistant)'
```

Result: exit status 0. All 16 selected AI-assistant tests passed; 3,570 were
skipped. This includes the logged-out test, HTTP 503 test, and all three
additional failure-mode tests.

## GraphQL client feature matrix and regression availability

Both GraphQL library configurations compiled successfully:

| Command | Result |
|---|---|
| `rustup run 1.92.0 cargo check -p cognix_graphql --no-default-features` | Passed. |
| `rustup run 1.92.0 cargo check -p cognix_graphql --features test-util` | Passed. |

The GraphQL crate currently defines no unit tests. A plain
`cargo nextest run -p cognix_graphql` could not compile its HTTP-client test
helper because reqwest's `tls_built_in_webpki_certs` API is not enabled by
that target's default feature set. With the test-only TLS feature enabled,
the GraphQL test target compiles:

```sh
rustup run 1.92.0 cargo test -p cognix_graphql --no-run --features 'test-util reqwest/rustls-tls-webpki-roots-no-provider'
```

Result: exit status 0; the test binary was produced. Running nextest with the
same features reported 0 tests and exit status 4 (`no tests to run`), so no
GraphQL client regression assertions were available to execute.

## Limits

- An unavailable-service shell test is not a request-capture proof that the
  app never attempts network access.
- The test does not characterize real OSS first-run onboarding, optional AI
  model listing, telemetry queues, crash reporting, or updater polling.
- No historical Preview runtime comparison or performance profile was run as
  part of this focused workflow.
