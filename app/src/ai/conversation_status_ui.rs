use cognix_core::ui::appearance::Appearance;
use cognix_core::ui::color::coloru_with_opacity;
use cognix_core::ui::theme::{CognixTheme, Fill};
use cognixui::color::ColorU;
use cognixui::elements::{ConstrainedBox, Container, CornerRadius, Radius};
use cognixui::Element;

use crate::ai::agent::conversation::ConversationStatus;
use crate::ai::agent_conversations_model::AgentRunDisplayStatus;
use crate::ui_components::icons::Icon;

/// Padding around the status icon
pub const STATUS_ELEMENT_PADDING: f32 = 2.;

pub trait StatusElementStyle {
    fn status_icon_and_color(&self, theme: &CognixTheme) -> (Icon, ColorU);
}

impl StatusElementStyle for ConversationStatus {
    fn status_icon_and_color(&self, theme: &CognixTheme) -> (Icon, ColorU) {
        ConversationStatus::status_icon_and_color(self, theme)
    }
}

impl StatusElementStyle for AgentRunDisplayStatus {
    fn status_icon_and_color(&self, theme: &CognixTheme) -> (Icon, ColorU) {
        AgentRunDisplayStatus::status_icon_and_color(self, theme)
    }
}

/// Render the status element used by agent and conversation views.
pub fn render_status_element(
    status: &impl StatusElementStyle,
    icon_size: f32,
    appearance: &Appearance,
) -> Box<dyn Element> {
    let theme = appearance.theme();
    let (icon, color) = status.status_icon_and_color(theme);

    Container::new(
        ConstrainedBox::new(icon.to_cognixui_icon(Fill::from(color)).finish())
            .with_width(icon_size)
            .with_height(icon_size)
            .finish(),
    )
    .with_uniform_padding(STATUS_ELEMENT_PADDING)
    .with_background(coloru_with_opacity(color, 10))
    .with_corner_radius(CornerRadius::with_all(Radius::Pixels(4.)))
    .finish()
}
