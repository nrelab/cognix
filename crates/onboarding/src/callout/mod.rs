mod model;
mod view;

pub use model::{FinalState, OnboardingQuery};
pub use view::{OnboardingCalloutView, OnboardingCalloutViewEvent, OnboardingKeybindings};

pub fn init(app: &mut cognixui::AppContext) {
    view::init(app);
}
