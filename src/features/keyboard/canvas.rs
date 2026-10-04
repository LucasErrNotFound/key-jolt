use super::KeyboardEditorView;
use super::layout::{
    FULL_NUMPAD_ROW_1, FULL_NUMPAD_ROW_2, FULL_NUMPAD_ROW_3, FULL_NUMPAD_ROW_4, FULL_NUMPAD_ROW_5,
    KeySpec, KeyboardLayout, ROW_1, ROW_2, ROW_3, ROW_4, ROW_5, ROW_6, TKL_FUNCTION_ROW,
    TKL_MAIN_ROW_1, TKL_MAIN_ROW_2, TKL_MAIN_ROW_3, TKL_MAIN_ROW_4, TKL_MAIN_ROW_5, TKL_NAV_ROW_1,
    TKL_NAV_ROW_2, TKL_NAV_ROW_3, TKL_NAV_ROW_4, TKL_SYSTEM_ROW,
};

use gpui_kit::component::button::{Button, ButtonCustomVariant, ButtonVariants};

use gpui_kit::component::kbd::Kbd;
use gpui_kit::component::scroll::ScrollableElement;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl KeyboardEditorView {
    pub(super) fn render_keyboard(&self, cx: &mut Context<Self>) -> AnyElement {
        match self.keyboard_layout {
            KeyboardLayout::FullSize => self.render_full_size_keyboard(cx).into_any_element(),
            KeyboardLayout::Tkl => self.render_tkl_keyboard(cx).into_any_element(),
            KeyboardLayout::Compact => self.render_compact_keyboard(cx).into_any_element(),
        }
    }

    pub(super) fn render_keyboard_area(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let width = self.keyboard_layout.width();

        div()
            .id("keyboard-horizontal-scroll")
            .w_full()
            .min_w_0()
            .overflow_x_scrollbar()
            .child(
                div()
                    .flex_shrink_0()
                    .w(rems((width) / 16.0))
                    .pb(rems(0.75))
                    .child(self.render_keyboard(cx)),
            )
    }

    pub(super) fn render_key_row(
        &self,
        row: &[KeySpec],
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        h_flex().w_full().gap_1().children(row.iter().map(|key| {
            if key.id.starts_with("nav_up_spacer") {
                return div()
                    .w(rems((key.width) / 16.0))
                    .h(rems(2.125))
                    .into_any_element();
            }

            let selected = self.is_selected(key.id);
            let key_id = key.id;

            let button_variant = if selected {
                ButtonCustomVariant::new(cx)
                    .color(cx.theme().primary.opacity(0.08))
                    .foreground(cx.theme().primary)
                    .hover(cx.theme().primary.opacity(0.08))
                    .active(cx.theme().primary.opacity(0.12))
            } else {
                ButtonCustomVariant::new(cx)
                    .color(cx.theme().background)
                    .foreground(cx.theme().muted_foreground)
                    .hover(cx.theme().background)
                    .active(cx.theme().background)
            };

            Button::new(key.id)
                .custom(button_variant)
                .border_1()
                .border_color(if selected {
                    cx.theme().primary
                } else {
                    cx.theme().muted_foreground.opacity(0.3)
                })
                .w(rems((key.width) / 16.0))
                .h(rems(2.125))
                .px_0()
                .text_size(rems(0.75))
                .label(key.label)
                .on_click(cx.listener(move |this, _event, _window, cx| {
                    this.toggle_key(key_id, cx);
                }))
                .into_any_element()
        }))
    }

    pub(super) fn render_compact_keyboard(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_1()
            .child(self.render_key_row(&ROW_1, cx))
            .child(self.render_key_row(&ROW_2, cx))
            .child(self.render_key_row(&ROW_3, cx))
            .child(self.render_key_row(&ROW_4, cx))
            .child(self.render_key_row(&ROW_5, cx))
            .child(self.render_key_row(&ROW_6, cx))
    }

    pub(super) fn render_tkl_keyboard(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_2()
            .w(rems(48.125))
            .items_start()
            .child(
                v_flex()
                    .gap_1()
                    .child(self.render_key_row(&TKL_FUNCTION_ROW, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_1, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_2, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_3, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_4, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_5, cx)),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(self.render_key_row(TKL_SYSTEM_ROW, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_1, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_2, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_3, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_4, cx)),
            )
    }

    pub(super) fn render_full_size_keyboard(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_2()
            .w(rems(59.375))
            .items_start()
            .child(
                v_flex()
                    .gap_1()
                    .child(self.render_key_row(&TKL_FUNCTION_ROW, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_1, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_2, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_3, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_4, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_5, cx)),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(self.render_key_row(TKL_SYSTEM_ROW, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_1, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_2, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_3, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_4, cx)),
            )
            .child(
                v_flex()
                    .gap_1()
                    .child(self.render_key_row(FULL_NUMPAD_ROW_1, cx))
                    .child(self.render_key_row(FULL_NUMPAD_ROW_2, cx))
                    .child(self.render_key_row(FULL_NUMPAD_ROW_3, cx))
                    .child(self.render_key_row(FULL_NUMPAD_ROW_4, cx))
                    .child(self.render_key_row(FULL_NUMPAD_ROW_5, cx)),
            )
    }

    pub(super) fn render_hint(text: impl Into<SharedString>, cx: &App) -> impl IntoElement {
        div()
            .px_2()
            .py(rems(0.0625))
            .rounded(cx.theme().radius_tokens().md)
            .border_1()
            .border_color(cx.theme().muted_foreground.opacity(0.3))
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(text.into())
    }

    pub(super) fn render_kbd_hint(keystroke: &str, cx: &App) -> impl IntoElement {
        div()
            .px_2()
            .py(rems(0.0625))
            .rounded(cx.theme().radius_tokens().md)
            .border_1()
            .border_color(cx.theme().muted_foreground.opacity(0.3))
            .bg(cx.theme().background)
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(
                Kbd::new(Keystroke::parse(keystroke).unwrap())
                    .appearance(false)
                    .text_size(rems(0.75)),
            )
    }

    pub(super) fn render_selection_controls(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let groups = self.selection_groups();

        h_flex()
            .w_full()
            .flex_wrap()
            .gap_1()
            .children(groups.into_iter().map(|group| {
                let active = self.are_keys_selected(&group.keys);
                let keys = group.keys;
                let group_id = group.id;
                let id = format!("quick-select-{}", group.id);

                Button::new(id)
                    .label(group.label)
                    .compact()
                    .selected(active)
                    .when(active, |button| button.primary())
                    .when(!active, |button| button.outline())
                    .on_click(cx.listener(move |this, _, _, cx| {
                        this.toggle_selection_group(group_id, &keys, cx);
                    }))
                    .into_any_element()
            }))
            .child(
                Button::new("clear-selected-keys")
                    .danger()
                    .outline()
                    .compact()
                    .label("Clear selected keys")
                    .disabled(self.selected_keys[self.keyboard_layout.index()].is_empty())
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.clear_selected_keys(cx);
                    })),
            )
    }
}
