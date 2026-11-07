use std::sync::Arc;

use parking_lot::FairMutex;
use cognixui::prelude::Empty;
use cognixui::{
    elements::{
        ChildView, Container, CrossAxisAlignment, Expanded, Flex, MainAxisSize, ParentElement,
    },
    AppContext, Element, Entity, TypedActionView, View, ViewContext, ViewHandle,
};

use crate::{
    terminal::view::{TerminalModel, PADDING_LEFT},
    ui_components::icons::Icon,
    view_components::action_button::{ActionButton, ButtonSize, KeystrokeSource, TooltipAlignment},
};

use super::{AgentFooterButtonTheme, USE_AGENT_KEYSTROKE};
use crate::terminal::view::block_banner::CognixificationMode;

/// Footer view rendered for detected subshell/SSH commands, offering both
/// "Cognixify" and "Use agent" buttons in a horizontal row.
pub(super) struct CognixifyFooterView {
    terminal_model: Arc<FairMutex<TerminalModel>>,
    cognixify_button: ViewHandle<ActionButton>,
    use_agent_button: ViewHandle<ActionButton>,
    dismiss_button: ViewHandle<ActionButton>,
    mode: Option<CognixificationMode>,
}

impl CognixifyFooterView {
    pub fn new(terminal_model: Arc<FairMutex<TerminalModel>>, ctx: &mut ViewContext<Self>) -> Self {
        let button_size = ButtonSize::XSmall;

        let cognixify_button = ctx.add_typed_action_view(|_ctx| {
            ActionButton::new("Cognixify subshell", AgentFooterButtonTheme::new(None))
                .with_icon(Icon::Cognix)
                .with_size(button_size)
                .with_tooltip("Enable Cognix shell integration in this session")
                .with_tooltip_alignment(TooltipAlignment::Left)
                .on_click(|ctx| {
                    ctx.dispatch_typed_action(CognixifyFooterViewAction::Cognixify);
                })
        });

        let use_agent_button = ctx.add_typed_action_view(|ctx| {
            ActionButton::new("Use agent", AgentFooterButtonTheme::new(None))
                .with_icon(Icon::Oz)
                .with_keybinding(KeystrokeSource::Fixed(USE_AGENT_KEYSTROKE.clone()), ctx)
                .with_size(button_size)
                .with_tooltip("Ask the Cognix agent to assist")
                .with_tooltip_alignment(TooltipAlignment::Left)
                .on_click(|ctx| {
                    ctx.dispatch_typed_action(CognixifyFooterViewAction::UseAgent);
                })
        });

        let dismiss_button = ctx.add_typed_action_view(|_ctx| {
            ActionButton::new("Dismiss", AgentFooterButtonTheme::new(None))
                .with_size(button_size)
                .on_click(|ctx| {
                    ctx.dispatch_typed_action(CognixifyFooterViewAction::Dismiss);
                })
        });

        Self {
            terminal_model,
            cognixify_button,
            use_agent_button,
            dismiss_button,
            mode: None,
        }
    }

    /// Updates the cognixify button label, keybinding, and stores the current cognixification mode.
    pub fn set_mode(&mut self, mode: CognixificationMode, ctx: &mut ViewContext<Self>) {
        let (label, binding_name) = match mode {
            CognixificationMode::Ssh { .. } => {
                ("Cognixify SSH session", "terminal:cognixify_ssh_session")
            }
            CognixificationMode::Subshell { .. } => ("Cognixify subshell", "terminal:cognixify_subshell"),
        };
        self.cognixify_button.update(ctx, |button, ctx| {
            button.set_label(label, ctx);
            button.set_keybinding(Some(KeystrokeSource::Binding(binding_name)), ctx);
        });
        self.mode = Some(mode);
        ctx.notify();
    }

    /// Returns the current cognixification mode, if set.
    pub fn mode(&self) -> Option<&CognixificationMode> {
        self.mode.as_ref()
    }

    /// Clears the cognixification mode.
    pub fn clear_mode(&mut self, ctx: &mut ViewContext<Self>) {
        self.mode = None;
        self.cognixify_button.update(ctx, |button, ctx| {
            button.set_keybinding(None, ctx);
        });
        ctx.notify();
    }
}

#[derive(Debug, Clone)]
pub enum CognixifyFooterViewAction {
    Cognixify,
    UseAgent,
    Dismiss,
}

pub enum CognixifyFooterViewEvent {
    Cognixify { mode: CognixificationMode },
    UseAgent,
    Dismiss,
}

impl Entity for CognixifyFooterView {
    type Event = CognixifyFooterViewEvent;
}

impl View for CognixifyFooterView {
    fn ui_name() -> &'static str {
        "CognixifyFooterView"
    }

    fn render(&self, _app: &AppContext) -> Box<dyn Element> {
        let terminal_model = self.terminal_model.lock();

        let button_row = Flex::row()
            .with_spacing(4.)
            .with_main_axis_size(MainAxisSize::Max)
            .with_cross_axis_alignment(CrossAxisAlignment::Center)
            .with_child(ChildView::new(&self.cognixify_button).finish())
            .with_child(ChildView::new(&self.use_agent_button).finish())
            .with_child(Expanded::new(1., Empty::new().finish()).finish())
            .with_child(ChildView::new(&self.dismiss_button).finish());

        let mut container = Container::new(button_row.finish())
            .with_horizontal_padding(*PADDING_LEFT)
            .with_vertical_padding(4.);

        if terminal_model.is_alt_screen_active() {
            if let Some(bg_color) = terminal_model.alt_screen().inferred_bg_color() {
                container = container.with_background(bg_color);
            }
        }

        container.finish()
    }
}

impl TypedActionView for CognixifyFooterView {
    type Action = CognixifyFooterViewAction;

    fn handle_action(&mut self, action: &Self::Action, ctx: &mut ViewContext<Self>) {
        match action {
            CognixifyFooterViewAction::Cognixify => {
                if let Some(mode) = self.mode.clone() {
                    self.clear_mode(ctx);
                    ctx.emit(CognixifyFooterViewEvent::Cognixify { mode });
                }
            }
            CognixifyFooterViewAction::UseAgent => {
                self.clear_mode(ctx);
                ctx.emit(CognixifyFooterViewEvent::UseAgent);
            }
            CognixifyFooterViewAction::Dismiss => {
                self.clear_mode(ctx);
                ctx.emit(CognixifyFooterViewEvent::Dismiss);
            }
        }
    }
}
