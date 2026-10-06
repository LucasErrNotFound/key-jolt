use super::title_bar::AppTitleBarEvent;
use crate::settings::appearance::{AppearanceMode, AppearanceSelection, ThemeDescriptor};

use gpui_kit::assets::IconName;
use gpui_kit::base::{h_flex, v_flex};

use gpui_kit::component::{ActiveTheme as _, Disableable as _, Selectable as _};

use gpui_kit::component::button::{Button, ButtonVariants};

use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::switch::Switch;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

pub(super) struct AppearancePicker {
    focus_handle: FocusHandle,
    selection: AppearanceSelection,
    themes: Vec<ThemeDescriptor>,
    run_at_startup: bool,
    startup_busy: bool,
    pub(super) error_message: Option<SharedString>,
}

impl EventEmitter<AppTitleBarEvent> for AppearancePicker {}

impl Focusable for AppearancePicker {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl AppearancePicker {
    pub(super) fn new(
        themes: Vec<ThemeDescriptor>,
        selection: AppearanceSelection,
        run_at_startup: bool,
        error_message: Option<String>,
        cx: &mut Context<Self>,
    ) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            selection,
            themes,
            run_at_startup,
            startup_busy: false,
            error_message: error_message.map(Into::into),
        }
    }

    pub(super) fn set_startup_state(&mut self, enabled: bool, busy: bool, cx: &mut Context<Self>) {
        self.run_at_startup = enabled;
        self.startup_busy = busy;
        cx.notify();
    }

    pub(super) fn set_selection(&mut self, selection: AppearanceSelection, cx: &mut Context<Self>) {
        self.selection = selection;
        self.error_message = None;
        cx.notify();
    }
}

impl Render for AppearancePicker {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mode = self.selection.mode;
        let theme_rows = self
            .themes
            .iter()
            .filter(|theme| theme.mode == mode)
            .map(|theme| {
                let theme_id = theme.id.clone();
                let selected = theme_id == self.selection.theme_id;
                Button::new(SharedString::from(format!("appearance-theme-{theme_id}")))
                    .ghost()
                    .w_full()
                    .flex_shrink_0()
                    .justify_start()
                    .accessibility_label(theme.display_name.clone())
                    .selected(selected)
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .gap_2()
                            .child(IconName::Palette)
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .text_left()
                                    .child(theme.display_name.clone()),
                            )
                            .child(
                                div()
                                    .w_4()
                                    .h_4()
                                    .flex_shrink_0()
                                    .when(selected, |indicator| indicator.child(IconName::Check)),
                            ),
                    )
                    .on_click(cx.listener(move |_, _, _, cx| {
                        cx.emit(AppTitleBarEvent::Theme(theme_id.clone()));
                    }))
            })
            .collect::<Vec<_>>();

        v_flex()
            .w(rems(17.5))
            .gap_3()
            .track_focus(&self.focus_handle)
            .child(
                h_flex()
                    .items_center()
                    .gap_3()
                    .child("Dark mode")
                    .child(div().flex_1())
                    .child(
                        Switch::new("appearance-dark-mode")
                            .checked(mode == AppearanceMode::Dark)
                            .accessibility_label("Dark mode")
                            .on_change(cx.listener(|this, dark, _, cx| {
                                if *dark != (this.selection.mode == AppearanceMode::Dark) {
                                    cx.emit(AppTitleBarEvent::Mode(this.selection.mode.opposite()));
                                }
                            })),
                    ),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        h_flex()
                            .items_center()
                            .gap_3()
                            .child("Run at startup")
                            .child(div().flex_1())
                            .child(
                                Switch::new("run-at-startup")
                                    .checked(self.run_at_startup)
                                    .disabled(
                                        self.startup_busy
                                            || !crate::platform::startup::is_supported(),
                                    )
                                    .accessibility_label("Run KeyJolt at startup")
                                    .on_change(cx.listener(|this, enabled, _, cx| {
                                        if !this.startup_busy && *enabled != this.run_at_startup {
                                            cx.emit(AppTitleBarEvent::RunAtStartup(*enabled));
                                        }
                                    })),
                            ),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child(if self.startup_busy {
                                "Saving startup preference…"
                            } else if crate::platform::startup::is_supported() {
                                "Start in the system tray when you sign in."
                            } else {
                                "Available on Windows."
                            }),
                    ),
            )
            .child(
                v_flex().gap_1().child("Themes").child(
                    div()
                        .text_sm()
                        .child(format!("Current: {}", self.selection.theme_id)),
                ),
            )
            .child(
                div().w_full().h(rems(22.5)).flex_shrink_0().child(
                    v_flex()
                        .id("appearance-theme-list")
                        .size_full()
                        .gap_1()
                        .pr_3()
                        .children(theme_rows)
                        .overflow_y_scrollbar(),
                ),
            )
            .when_some(self.error_message.clone(), |panel, message| {
                panel.child(
                    div()
                        .id("appearance-error")
                        .w_full()
                        .max_h(rems(6.25))
                        .text_sm()
                        .text_color(cx.theme().danger)
                        .child(message)
                        .overflow_y_scrollbar(),
                )
            })
    }
}
