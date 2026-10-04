use super::appearance_picker::AppearancePicker;
use crate::settings::appearance::{AppearanceMode, AppearanceSelection, ThemeDescriptor};

use gpui_kit::assets::IconName;
use gpui_kit::base::h_flex;
use gpui_kit::component::{ActiveTheme as _, Selectable as _, TitleBar};

use gpui_kit::component::button::{Button, ButtonVariants};

use gpui_kit::component::popover::Popover;
use gpui_kit::*;

const GITHUB_REPOSITORY_URL: &str = "https://github.com/LucasErrNotFound/key-jolt";

pub(super) struct AppTitleBar {
    appearance_picker: Entity<AppearancePicker>,
    popover_open: bool,
    _picker_subscription: Subscription,
}

#[derive(Clone, Debug)]
pub(super) enum AppTitleBarEvent {
    ModeSelected(AppearanceMode),
    ThemeSelected(String),
}

impl EventEmitter<AppTitleBarEvent> for AppTitleBar {}

impl AppTitleBar {
    pub(super) fn new(
        themes: Vec<ThemeDescriptor>,
        selection: AppearanceSelection,
        error_message: Option<String>,
        cx: &mut Context<Self>,
    ) -> Self {
        let appearance_picker =
            cx.new(|cx| AppearancePicker::new(themes, selection, error_message, cx));
        let picker_subscription =
            cx.subscribe(&appearance_picker, |_, _, event: &AppTitleBarEvent, cx| {
                cx.emit(event.clone());
            });

        Self {
            appearance_picker,
            popover_open: false,
            _picker_subscription: picker_subscription,
        }
    }

    pub(super) fn set_selection(&mut self, selection: AppearanceSelection, cx: &mut Context<Self>) {
        self.appearance_picker.update(cx, |picker, cx| {
            picker.set_selection(selection, cx);
        });
        cx.notify();
    }

    pub(super) fn set_error(&mut self, message: String, cx: &mut Context<Self>) {
        self.appearance_picker.update(cx, |picker, cx| {
            picker.error_message = Some(message.into());
            cx.notify();
        });
    }
}

impl Render for AppTitleBar {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let picker_focus = self.appearance_picker.focus_handle(cx);

        TitleBar::new()
            .child(div().flex().items_center().child("KeyJolt"))
            .child(div().flex_1())
            .child(
                h_flex()
                    .items_center()
                    .gap_2()
                    .child(
                        Popover::new("appearance-popover")
                            .anchor(Anchor::TopRight)
                            .offset(cx.theme().font_size * 0.25)
                            .overlay_closable(true)
                            .open(self.popover_open)
                            .track_focus(&picker_focus)
                            .trigger(
                                Button::new("settings")
                                    .ghost()
                                    .selected(self.popover_open)
                                    .icon(IconName::Settings)
                                    .accessibility_label("Settings")
                                    .tooltip("Settings"),
                            )
                            .child(self.appearance_picker.clone())
                            .on_open_change(cx.listener(|this, open, _, cx| {
                                this.popover_open = *open;
                                cx.notify();
                            })),
                    )
                    .child(
                        div()
                            .on_mouse_down(MouseButton::Left, |_, window, cx| {
                                window.prevent_default();
                                cx.stop_propagation();
                            })
                            .child(
                                Button::new("github")
                                    .ghost()
                                    .icon(IconName::Github)
                                    .accessibility_label("Open KeyJolt on GitHub")
                                    .tooltip("GitHub repository")
                                    .on_click(|_, _, cx| {
                                        cx.open_url(GITHUB_REPOSITORY_URL);
                                    }),
                            ),
                    ),
            )
    }
}
