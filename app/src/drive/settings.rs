use settings::{
    macros::define_settings_group, RespectUserSyncSetting, SupportedPlatforms, SyncToCloud,
};
use cognix_core::features::FeatureFlag;

use super::DriveSortOrder;

pub const HAS_AUTO_OPENED_WELCOME_FOLDER: &str = "HasAutoOpenedWelcomeFolder";

define_settings_group!(CognixDriveSettings, settings: [
    sorting_choice: CognixDriveSortingChoice {
        type: DriveSortOrder,
        default: DriveSortOrder::ByObjectType,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "cognix_drive.sorting_choice",
        description: "The sort order for items in Cognix Drive.",
    },
    sharing_onboarding_block_shown: CognixDriveSharingOnboardingBlockShown {
        type: bool,
        default: false,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: true,
    },
    // Controls whether Cognix Drive appears in the tools panel, command palette, and command search.
    enable_cognix_drive: EnableCognixDrive {
        type: bool,
        default: true,
        supported_platforms: SupportedPlatforms::ALL,
        sync_to_cloud: SyncToCloud::Globally(RespectUserSyncSetting::Yes),
        private: false,
        toml_path: "cognix_drive.enabled",
        description: "Whether Cognix Drive is enabled.",
    },
]);

impl CognixDriveSettings {
    /// Returns whether Cognix Drive should be considered enabled.
    /// Returns `false` when the user is anonymous or fully logged out,
    /// regardless of the user setting.
    pub fn is_cognix_drive_enabled(app: &cognixui::AppContext) -> bool {
        use cognixui::SingletonEntity as _;
        let is_anonymous_or_logged_out = FeatureFlag::SkipFirebaseAnonymousUser.is_enabled()
            && crate::auth::AuthStateProvider::as_ref(app)
                .get()
                .is_anonymous_or_logged_out();
        *Self::as_ref(app).enable_cognix_drive && !is_anonymous_or_logged_out
    }
}
