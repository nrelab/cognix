use std::sync::Arc;
use std::time::Duration;
use std::{io::Read, io::Write};

use mockito::{Matcher, Server, ServerOpts};

use super::{RequestStatus, Requests};
use crate::{
    auth::{auth_state::AuthState, AuthStateProvider},
    server::{
        server_api::{ai::AIClient, ServerApi},
        telemetry::context_provider::AppTelemetryContextProvider,
    },
    test_util::settings::initialize_settings_for_tests,
};
use warpui::App;

#[test]
fn logged_out_ai_request_shows_failure_in_transcript() {
    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);

        let auth_state_provider = AuthStateProvider::new_logged_out_for_test();
        let auth_state = auth_state_provider.get().clone();
        app.add_singleton_model(move |_| auth_state_provider);
        app.add_singleton_model(AppTelemetryContextProvider::new_context_provider);

        let server_api = Arc::new(ServerApi::new_for_test_with_auth_state(auth_state));
        let ai_client: Arc<dyn AIClient> = server_api.clone();
        let requests =
            app.add_model(move |ctx| Requests::new(server_api.clone(), ai_client.clone(), ctx));

        requests
            .update(&mut app, |requests, ctx| {
                let future = requests.start_request("Explain this command".to_string(), ctx);
                ctx.await_spawned_future(future.future_id())
            })
            .await;

        requests.read(&app, |requests, _| {
            assert!(matches!(
                requests.request_status(),
                RequestStatus::NotInFlight
            ));

            let transcript = requests.transcript();
            assert_eq!(transcript.len(), 1);
            assert!(transcript[0].assistant.is_error);
            assert_eq!(
                transcript[0].assistant.formatted_message.raw,
                "We're experiencing technical difficulties right now. Please try again later."
            );
        });
    });
}

#[test]
fn ai_request_shows_failure_when_backend_returns_service_unavailable() {
    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);

        let mut mock_server = Server::new_with_opts_async(ServerOpts {
            host: "127.0.0.1",
            port: 0,
            ..Default::default()
        })
        .await;
        let mock = mock_server
            .mock("POST", "/graphql/v2")
            .match_query(Matcher::UrlEncoded(
                "op".to_string(),
                "GenerateDialogue".to_string(),
            ))
            .match_body(Matcher::Regex("Explain this command".to_string()))
            .with_status(503)
            .with_body("service unavailable")
            .create_async()
            .await;

        let auth_state = Arc::new(AuthState::new_for_test());
        let auth_state_provider = AuthStateProvider::new(auth_state.clone());
        app.add_singleton_model(move |_| auth_state_provider);
        app.add_singleton_model(AppTelemetryContextProvider::new_context_provider);

        let server_api = Arc::new(ServerApi::new_for_test_with_auth_state_and_server_root_url(
            auth_state,
            Some(mock_server.url()),
        ));
        let ai_client: Arc<dyn AIClient> = server_api.clone();
        let requests =
            app.add_model(move |ctx| Requests::new(server_api.clone(), ai_client.clone(), ctx));

        requests
            .update(&mut app, |requests, ctx| {
                let future = requests.start_request("Explain this command".to_string(), ctx);
                ctx.await_spawned_future(future.future_id())
            })
            .await;

        mock.assert_async().await;
        requests.read(&app, |requests, _| {
            assert!(matches!(
                requests.request_status(),
                RequestStatus::NotInFlight
            ));

            let transcript = requests.transcript();
            assert_eq!(transcript.len(), 1);
            assert!(transcript[0].assistant.is_error);
            assert_eq!(
                transcript[0].assistant.formatted_message.raw,
                "We're experiencing technical difficulties right now. Please try again later."
            );
        });
    });
}

#[test]
fn ai_request_shows_failure_when_backend_connection_is_refused() {
    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        drop(listener);
        let server_root_url = format!("http://{address}");

        let auth_state = Arc::new(AuthState::new_for_test());
        let auth_state_provider = AuthStateProvider::new(auth_state.clone());
        app.add_singleton_model(move |_| auth_state_provider);
        app.add_singleton_model(AppTelemetryContextProvider::new_context_provider);

        let server_api = Arc::new(ServerApi::new_for_test_with_auth_state_and_server_root_url(
            auth_state,
            Some(server_root_url),
        ));
        let ai_client: Arc<dyn AIClient> = server_api.clone();
        let requests =
            app.add_model(move |ctx| Requests::new(server_api.clone(), ai_client.clone(), ctx));

        match futures::future::select(
            Box::pin(requests.update(&mut app, |requests, ctx| {
                let future = requests.start_request("Explain this command".to_string(), ctx);
                ctx.await_spawned_future(future.future_id())
            })),
            Box::pin(futures_timer::Delay::new(Duration::from_secs(2))),
        )
        .await
        {
            futures::future::Either::Left(_) => {}
            futures::future::Either::Right(_) => {
                panic!("AI request did not complete after connection refusal")
            }
        }

        requests.read(&app, |requests, _| {
            assert!(matches!(
                requests.request_status(),
                RequestStatus::NotInFlight
            ));

            let transcript = requests.transcript();
            assert_eq!(transcript.len(), 1);
            assert!(transcript[0].assistant.is_error);
            assert_eq!(
                transcript[0].assistant.formatted_message.raw,
                "We're experiencing technical difficulties right now. Please try again later."
            );
        });
    });
}

#[test]
fn ai_request_shows_failure_when_backend_response_times_out() {
    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);

        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let server_thread = std::thread::spawn(move || {
            let mut saw_request_limit_info = false;
            let mut saw_generate_dialogue = false;
            let accept_deadline = std::time::Instant::now() + Duration::from_secs(3);

            while !saw_request_limit_info || !saw_generate_dialogue {
                let (mut stream, _) = loop {
                    match listener.accept() {
                        Ok((stream, address)) => break (stream, address),
                        Err(error)
                            if error.kind() == std::io::ErrorKind::WouldBlock
                                && std::time::Instant::now() < accept_deadline =>
                        {
                            std::thread::sleep(Duration::from_millis(10));
                        }
                        Err(error) => panic!("Failed to accept local AI request: {error}"),
                    }
                };
                stream.set_nonblocking(false).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                stream
                    .set_write_timeout(Some(Duration::from_secs(2)))
                    .unwrap();
                let mut request_line = Vec::new();
                loop {
                    let mut byte = [0];
                    let bytes_read = stream.read(&mut byte).unwrap();
                    assert_eq!(bytes_read, 1);
                    request_line.push(byte[0]);
                    if request_line.ends_with(b"\r\n") {
                        break;
                    }
                }
                assert!(request_line.starts_with(b"POST /graphql/v2?op="));

                if request_line
                    .windows(b"op=GetRequestLimitInfo".len())
                    .any(|window| window == b"op=GetRequestLimitInfo")
                {
                    saw_request_limit_info = true;
                } else if request_line
                    .windows(b"op=GenerateDialogue".len())
                    .any(|window| window == b"op=GenerateDialogue")
                {
                    saw_generate_dialogue = true;
                    std::thread::sleep(Duration::from_millis(500));
                } else {
                    panic!("Unexpected GraphQL operation received by local test server");
                }

                let response_result = stream.write_all(
                    b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                );
                if let Err(error) = response_result {
                    assert!(matches!(
                        error.kind(),
                        std::io::ErrorKind::BrokenPipe
                            | std::io::ErrorKind::ConnectionReset
                            | std::io::ErrorKind::ConnectionAborted
                    ));
                }
            }
            (saw_request_limit_info, saw_generate_dialogue)
        });
        let server_root_url = format!("http://{address}");
        let request_timeout = Duration::from_millis(100);

        let auth_state = Arc::new(AuthState::new_for_test());
        let auth_state_provider = AuthStateProvider::new(auth_state.clone());
        app.add_singleton_model(move |_| auth_state_provider);
        app.add_singleton_model(AppTelemetryContextProvider::new_context_provider);

        let server_api = Arc::new(ServerApi::new_for_test_with_auth_state_and_overrides(
            auth_state,
            Some(server_root_url),
            Some(request_timeout),
        ));
        let ai_client: Arc<dyn AIClient> = server_api.clone();
        let requests =
            app.add_model(move |ctx| Requests::new(server_api.clone(), ai_client.clone(), ctx));

        match futures::future::select(
            Box::pin(requests.update(&mut app, |requests, ctx| {
                let future = requests.start_request("Explain this command".to_string(), ctx);
                ctx.await_spawned_future(future.future_id())
            })),
            Box::pin(futures_timer::Delay::new(Duration::from_secs(2))),
        )
        .await
        {
            futures::future::Either::Left(_) => {}
            futures::future::Either::Right(_) => {
                panic!("AI request did not complete after the configured timeout")
            }
        }
        assert_eq!(server_thread.join().unwrap(), (true, true));

        requests.read(&app, |requests, _| {
            assert!(matches!(
                requests.request_status(),
                RequestStatus::NotInFlight
            ));

            let transcript = requests.transcript();
            assert_eq!(transcript.len(), 1);
            assert!(transcript[0].assistant.is_error);
            assert_eq!(
                transcript[0].assistant.formatted_message.raw,
                "We're experiencing technical difficulties right now. Please try again later."
            );
        });
    });
}

#[test]
fn repeated_ai_request_failures_leave_request_state_consistent() {
    App::test((), |mut app| async move {
        initialize_settings_for_tests(&mut app);

        let mut mock_server = Server::new_with_opts_async(ServerOpts {
            host: "127.0.0.1",
            port: 0,
            ..Default::default()
        })
        .await;
        let mock = mock_server
            .mock("POST", "/graphql/v2")
            .match_query(Matcher::UrlEncoded(
                "op".to_string(),
                "GenerateDialogue".to_string(),
            ))
            .with_status(503)
            .with_body("service unavailable")
            .expect(3)
            .create_async()
            .await;

        let auth_state = Arc::new(AuthState::new_for_test());
        let auth_state_provider = AuthStateProvider::new(auth_state.clone());
        app.add_singleton_model(move |_| auth_state_provider);
        app.add_singleton_model(AppTelemetryContextProvider::new_context_provider);

        let server_api = Arc::new(ServerApi::new_for_test_with_auth_state_and_server_root_url(
            auth_state,
            Some(mock_server.url()),
        ));
        let ai_client: Arc<dyn AIClient> = server_api.clone();
        let requests =
            app.add_model(move |ctx| Requests::new(server_api.clone(), ai_client.clone(), ctx));

        for request_index in 0..3 {
            requests
                .update(&mut app, |requests, ctx| {
                    let future = requests
                        .start_request(format!("Synthetic test prompt {request_index}"), ctx);
                    ctx.await_spawned_future(future.future_id())
                })
                .await;

            requests.read(&app, |requests, _| {
                assert!(matches!(
                    requests.request_status(),
                    RequestStatus::NotInFlight
                ));
                assert_eq!(requests.transcript().len(), request_index + 1);
                assert!(requests
                    .transcript()
                    .iter()
                    .all(|part| part.assistant.is_error));
            });
        }

        mock.assert_async().await;
    });
}
