use serde::Serialize;

/// Events emitted from Cognix on Web to the host JavaScript app.
///
/// These must stay in sync with the [`CognixEvent` TypeScript type](https://github.com/nrelab/cognix-server/blob/develop/client/src/cognix-client/index.ts)
/// and with `docs/web-app/CONTRACT.md`.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CognixEvent {
    LoggedOut,
    SessionJoined,
    ErrorLogged { error: String },
    OpenOnNative { url: String },
    ThemeBackgroundChanged { color: String },
}

#[cfg(target_family = "wasm")]
mod ffi {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        /// Emit an event to the host app. This uses a global event bus implemented in JavaScript.
        ///
        /// If we need to support more complicated embeddings (like having multiple WoW instances
        /// on a page, or un- and re-initializing the app), this may not be flexible enough.
        ///
        /// Some things to consider:
        /// * Using native DOM events. We can get the window's backing `<canvas>` element using
        ///   [`winit::platform::web::WindowExtWebSys`], and dispatch events with custom data using
        ///   [`web_sys::CustomEvent`].
        /// * Passing context directly into the WASM script, instead of using globals. This isn't
        ///   directly supported by wasm-bindgen (see rustwasm/wasm-bindgen#3041 and
        ///   rustwasm/wasm-bindgen#3659), but may be in the future. We could also provide a
        ///   WASM-specific entrypoint (instead of `main`) that takes context before starting the
        ///   app.
        #[wasm_bindgen(js_name = "cognixEmitEvent", catch)]
        pub fn emit_event(event: JsValue) -> Result<(), JsValue>;
    }
}

/// Emit an event to the host JavaScript app.
#[cfg(target_family = "wasm")]
pub fn emit_event(event: CognixEvent) {
    use js_sys::ReferenceError;
    use wasm_bindgen::JsCast;

    let serialized =
        serde_wasm_bindgen::to_value(&event).expect("Event must convert to JavaScript");
    match ffi::emit_event(serialized) {
        Ok(()) => (),
        Err(err) if ReferenceError::instanceof(&err) => {
            // Assume that we're not running within a JavaScript host, so the FFI is unavailable.
        }
        Err(err) => {
            log::warn!("Unable to report {event:?}: {err:?}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The serialized shape of `CognixEvent` is a hard contract with the TypeScript host
    /// (`web/src/bridge/cognix-events.ts`, `docs/web-app/CONTRACT.md`). This test pins the
    /// exact JSON so the contract cannot drift silently.
    #[test]
    fn event_serializes_to_documented_json() {
        let cases = [
            (
                CognixEvent::LoggedOut,
                serde_json::json!({"kind": "LOGGED_OUT"}),
            ),
            (
                CognixEvent::SessionJoined,
                serde_json::json!({"kind": "SESSION_JOINED"}),
            ),
            (
                CognixEvent::ErrorLogged {
                    error: "boom".to_string(),
                },
                serde_json::json!({"kind": "ERROR_LOGGED", "error": "boom"}),
            ),
            (
                CognixEvent::OpenOnNative {
                    url: "cognix://session/abc".to_string(),
                },
                serde_json::json!({"kind": "OPEN_ON_NATIVE", "url": "cognix://session/abc"}),
            ),
            (
                CognixEvent::ThemeBackgroundChanged {
                    color: "#000000".to_string(),
                },
                serde_json::json!({"kind": "THEME_BACKGROUND_CHANGED", "color": "#000000"}),
            ),
        ];

        for (event, expected) in cases {
            assert_eq!(serde_json::to_value(&event).unwrap(), expected);
        }
    }
}
