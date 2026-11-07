cfg_if::cfg_if! {
    if #[cfg(not(target_family = "wasm"))] {
        mod native;
        pub use native::{IntoCognixJs, FromCognixJs, util};
    }
}
