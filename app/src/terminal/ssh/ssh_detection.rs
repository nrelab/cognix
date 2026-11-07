use serde::{Deserialize, Serialize};
use cognix_core::{features::FeatureFlag, settings::Setting};
use cognix_util::path::ShellFamily;

use crate::terminal::cognixify::settings::CognixifySettings;

/// The different possible outcomes of detecting an interactive SSH session.
/// Also the payload for the [`crate::server::telemetry::TelemetryEvent::SshInteractiveSessionDetected`] event.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SshInteractiveSessionDetected {
    #[serde(rename = "feature_disabled")]
    FeatureDisabled,
    #[serde(rename = "host_denylisted")]
    HostDenylisted,
    #[serde(rename = "cognixify_prompt")]
    ShouldPromptCognixification {
        #[serde(skip)]
        command: String,
        #[serde(skip)]
        host: Option<String>,
    },
}

/// Determines whether a host could be cognixified.
pub fn evaluate_cognixify_ssh_host(
    command: &str,
    ssh_host: Option<&str>,
    shell_family: ShellFamily,
    cognixify_settings: &CognixifySettings,
) -> SshInteractiveSessionDetected {
    let should_prompt_ssh_tmux_wrapper = *cognixify_settings.enable_ssh_cognixification.value()
        && *cognixify_settings.use_ssh_tmux_wrapper.value();
    let matches_subshell = cognixify_settings.is_denylisted_subshell_command(command)
        || cognixify_settings.is_compatible_subshell_command(command, shell_family);
    if !should_prompt_ssh_tmux_wrapper
        || matches_subshell
        || !FeatureFlag::SSHTmuxWrapper.is_enabled()
    {
        return SshInteractiveSessionDetected::FeatureDisabled;
    }

    if let Some(ssh_host) = ssh_host {
        if cognixify_settings.is_ssh_host_denylisted(ssh_host) {
            return SshInteractiveSessionDetected::HostDenylisted;
        }
    }

    SshInteractiveSessionDetected::ShouldPromptCognixification {
        host: ssh_host.map(|host| host.to_owned()),
        command: command.to_string(),
    }
}
