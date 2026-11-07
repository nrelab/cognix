use std::time::Duration;

pub mod error;
pub mod install_tmux;
pub mod root_access;
pub mod ssh_detection;
pub mod util;
pub mod cognixify;

pub const SSH_COGNIXIFY_TIMEOUT_DURATION: Duration = Duration::from_secs(8);
