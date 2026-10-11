use super::KeyboardEditorView;
use super::assignment::{AssignmentGroups, assignment_groups};
use super::layout::{
    FULL_NUMPAD_ROW_1, FULL_NUMPAD_ROW_2, FULL_NUMPAD_ROW_3, FULL_NUMPAD_ROW_4, FULL_NUMPAD_ROW_5,
    KeySpec, KeyboardLayout, ROW_1, ROW_2, ROW_3, ROW_4, ROW_5, ROW_6, TKL_FUNCTION_ROW,
    TKL_MAIN_ROW_1, TKL_MAIN_ROW_2, TKL_MAIN_ROW_3, TKL_MAIN_ROW_4, TKL_MAIN_ROW_5, TKL_NAV_ROW_1,
    TKL_NAV_ROW_2, TKL_NAV_ROW_3, TKL_NAV_ROW_4, TKL_SYSTEM_ROW,
};

use gpui_kit::component::button::{Button, ButtonCustomVariant, ButtonVariants};

use gpui_kit::component::tooltip::Tooltip;
use gpui_kit::component::*;
use gpui_kit::prelude::FluentBuilder;
use gpui_kit::*;

impl KeyboardEditorView {
    fn keyboard_column_widths(&self) -> [f32; 3] {
        if self.keyboard_layout == KeyboardLayout::Compact {
            return [
                widest_keyboard_row(&[
                    ROW_1.as_slice(),
                    ROW_2.as_slice(),
                    ROW_3.as_slice(),
                    ROW_4.as_slice(),
                    ROW_5.as_slice(),
                    ROW_6.as_slice(),
                ]),
                0.,
                0.,
            ];
        }
        let main = widest_keyboard_row(&[
            TKL_FUNCTION_ROW.as_slice(),
            TKL_MAIN_ROW_1,
            TKL_MAIN_ROW_2,
            TKL_MAIN_ROW_3,
            TKL_MAIN_ROW_4,
            TKL_MAIN_ROW_5,
        ]);
        let navigation = widest_keyboard_row(&[
            TKL_SYSTEM_ROW,
            TKL_NAV_ROW_1,
            TKL_NAV_ROW_2,
            TKL_NAV_ROW_3,
            TKL_NAV_ROW_4,
        ]);
        let numpad = if self.keyboard_layout == KeyboardLayout::FullSize {
            widest_keyboard_row(&[
                FULL_NUMPAD_ROW_1,
                FULL_NUMPAD_ROW_2,
                FULL_NUMPAD_ROW_3,
                FULL_NUMPAD_ROW_4,
                FULL_NUMPAD_ROW_5,
            ])
        } else {
            0.
        };
        [main, navigation, numpad]
    }

    pub(super) fn keyboard_canvas_width(&self) -> f32 {
        let columns = self.keyboard_column_widths();
        let gaps = match self.keyboard_layout {
            KeyboardLayout::Compact => 0.,
            KeyboardLayout::Tkl => 0.5,
            KeyboardLayout::FullSize => 1.,
        };
        (columns.into_iter().sum::<f32>() + gaps).max(self.keyboard_layout.width() / 16.)
    }

    fn assignment_groups(&self) -> AssignmentGroups {
        assignment_groups(self.files.iter().map(|file| {
            (
                file.id,
                file.assigned_keys[self.keyboard_layout.index()].as_slice(),
            )
        }))
    }

    pub(super) fn assignment_color(index: usize, cx: &App) -> Hsla {
        let theme = cx.theme();
        let colors = [
            theme.green,
            theme.magenta,
            theme.cyan,
            theme.yellow,
            theme.blue,
            theme.red,
        ];
        colors[index % colors.len()]
    }

    pub(super) fn assignment_sound_names(&self, sounds: &[u64]) -> String {
        let mut names = self
            .files
            .iter()
            .filter(|file| sounds.contains(&file.id))
            .map(|file| file.name.to_string())
            .collect::<Vec<_>>();
        names.sort();
        names.join(", ")
    }

    pub(super) fn render_assignment_legend(&self, cx: &App) -> impl IntoElement {
        let groups = self.assignment_groups();
        v_flex()
            .w_full()
            .gap_1()
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .child("Tinted keys share sounds. A stronger outline marks selected keys."),
            )
            .child(
                h_flex()
                    .flex_wrap()
                    .gap_2()
                    .children(groups.sounds.iter().enumerate().map(|(index, sounds)| {
                        let names = self.assignment_sound_names(sounds);
                        let tooltip: SharedString = format!("G{} · {}", index + 1, names).into();
                        h_flex()
                            .id(format!("assignment-group-{index}"))
                            .items_center()
                            .gap_1()
                            .text_xs()
                            .tooltip(move |window, cx| {
                                Tooltip::new(tooltip.clone()).build(window, cx)
                            })
                            .child(
                                div()
                                    .size(rems(0.5))
                                    .rounded_full()
                                    .bg(Self::assignment_color(index, cx)),
                            )
                            .child(format!("G{}", index + 1))
                            .child(div().max_w(rems(12.0)).truncate().child(names))
                    })),
            )
    }

    pub(super) fn render_keyboard(&self, cx: &mut Context<Self>) -> AnyElement {
        match self.keyboard_layout {
            KeyboardLayout::FullSize => self.render_full_size_keyboard(cx).into_any_element(),
            KeyboardLayout::Tkl => self.render_tkl_keyboard(cx).into_any_element(),
            KeyboardLayout::Compact => self.render_compact_keyboard(cx).into_any_element(),
        }
    }

    pub(super) fn render_keyboard_area(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let geometry = self.canvas_geometry.clone();
        self.canvas_geometry.borrow_mut().keys.clear();
        let natural_width = self.keyboard_canvas_width();
        let mut keyboard = self.render_keyboard(cx);
        let keyboard = canvas(
            move |bounds, window, cx| {
                let zoomed_rem = px((bounds.size.width.as_f32() / natural_width).max(0.01));
                window.with_rem_size(Some(zoomed_rem), |window| {
                    let _ = keyboard.prepaint_as_root(
                        bounds.origin,
                        size(
                            AvailableSpace::Definite(bounds.size.width),
                            AvailableSpace::Definite(bounds.size.height),
                        ),
                        window,
                        cx,
                    );
                });
                (keyboard, zoomed_rem)
            },
            |_, (mut keyboard, zoomed_rem), window, cx| {
                window.with_rem_size(Some(zoomed_rem), |window| keyboard.paint(window, cx));
            },
        )
        .w_full()
        .min_w_0()
        .max_w(rems(natural_width))
        .aspect_ratio(natural_width / 14.)
        .flex_shrink_0();
        div()
            .id("keyboard-canvas")
            .track_focus(&self.canvas_focus)
            .tab_stop(true)
            .relative()
            .overflow_hidden()
            .w_full()
            .min_w_0()
            .p_2()
            .rounded(cx.theme().radius_tokens().lg)
            .border_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().muted.opacity(0.3))
            .child(
                div()
                    .relative()
                    .w_full()
                    .min_w_0()
                    .child(
                        div()
                            .id("keyboard-canvas-viewport")
                            .w_full()
                            .min_w_0()
                            .on_prepaint(move |bounds, window, _| {
                                let mut geometry = geometry.borrow_mut();
                                geometry.origin = bounds.origin;
                                geometry.viewport =
                                    Some(bounds.intersect(&window.content_mask().bounds));
                            })
                            .child(keyboard),
                    )
                    .child(self.render_selection_marquee(cx)),
            )
    }

    pub(super) fn render_key_row(
        &self,
        row: &[KeySpec],
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let groups = self.assignment_groups();
        let row_width = rems(keyboard_row_width(row));
        let key_height = rems(34. / 16.);
        let font_size = rems(0.75);
        h_flex()
            .w(row_width)
            .min_w(row_width)
            .max_w(row_width)
            .flex_shrink_0()
            .gap(rems(0.25))
            .children(row.iter().map(|key| {
                if key.id.starts_with("nav_up_spacer") {
                    return div()
                        .w(rems(key.width / 16.))
                        .min_w(rems(key.width / 16.))
                        .max_w(rems(key.width / 16.))
                        .flex_shrink_0()
                        .h(key_height)
                        .into_any_element();
                }

                let selected = self.is_selected(key.id);
                let group = groups
                    .keys
                    .get(crate::presets::canonical_key_identifier(key.id))
                    .copied();
                let color = group.map(|index| Self::assignment_color(index, cx));
                let key_id = key.id;

                let button_variant = if let Some(color) = color {
                    ButtonCustomVariant::new(cx)
                        .color(color.opacity(0.14))
                        .foreground(cx.theme().foreground)
                        .hover(color.opacity(0.2))
                        .active(color.opacity(0.24))
                } else if selected {
                    ButtonCustomVariant::new(cx)
                        .color(cx.theme().primary.opacity(0.08))
                        .foreground(cx.theme().primary)
                        .hover(cx.theme().primary.opacity(0.08))
                        .active(cx.theme().primary.opacity(0.12))
                } else {
                    ButtonCustomVariant::new(cx)
                        .color(cx.theme().background)
                        .foreground(cx.theme().muted_foreground)
                        .hover(cx.theme().muted)
                        .active(cx.theme().muted.opacity(0.8))
                };

                let geometry = self.canvas_geometry.clone();
                let key_width = rems(key.width / 16.);
                let key_button = Button::new(key.id)
                    .custom(button_variant)
                    .small()
                    .border_t(rems(if selected { 0.125 } else { 0.0625 }))
                    .border_b(rems(if selected { 0.125 } else { 0.0625 }))
                    .border_l(rems(if selected { 0.125 } else { 0.0625 }))
                    .border_r(rems(if selected { 0.125 } else { 0.0625 }))
                    .border_color(if selected {
                        cx.theme().primary
                    } else {
                        color.unwrap_or(cx.theme().muted_foreground.opacity(0.3))
                    })
                    .w(key_width)
                    .min_w(key_width)
                    .max_w(key_width)
                    .h(key_height)
                    .min_h(key_height)
                    .max_h(key_height)
                    .p_0()
                    .rounded_tl(rems(cx.theme().radius_tokens().sm.as_f32() / 16.))
                    .rounded_tr(rems(cx.theme().radius_tokens().sm.as_f32() / 16.))
                    .rounded_bl(rems(cx.theme().radius_tokens().sm.as_f32() / 16.))
                    .rounded_br(rems(cx.theme().radius_tokens().sm.as_f32() / 16.))
                    .text_size(font_size)
                    .child(
                        v_flex()
                            .size_full()
                            .min_w_0()
                            .overflow_hidden()
                            .items_center()
                            .justify_center()
                            .text_size(font_size)
                            .child(key.label)
                            .when_some(group, |label, index| {
                                label.child(
                                    div().text_size(rems(0.5)).child(format!("G{}", index + 1)),
                                )
                            }),
                    )
                    .accessibility_label(match group {
                        Some(index) => format!(
                            "{} · G{} · {}{}",
                            key.label,
                            index + 1,
                            self.assignment_sound_names(&groups.sounds[index]),
                            if selected { " · selected" } else { "" }
                        ),
                        None => format!(
                            "{} · no sound assigned{}",
                            key.label,
                            if selected { " · selected" } else { "" }
                        ),
                    })
                    .tooltip(match group {
                        Some(index) => format!(
                            "{} · G{} · {}",
                            key.label,
                            index + 1,
                            self.assignment_sound_names(&groups.sounds[index])
                        ),
                        None => format!("{} · no sound assigned", key.label),
                    })
                    .on_click(cx.listener(move |this, event: &ClickEvent, window, cx| {
                        if !this.suppress_key_click || event.mouse_position().is_none() {
                            this.select_key(key_id, event.modifiers().control, cx);
                            this.canvas_focus.focus(window, cx);
                        }
                    }));
                div()
                    .id(format!("key-bounds-{key_id}"))
                    .w(key_width)
                    .min_w(key_width)
                    .max_w(key_width)
                    .flex_shrink_0()
                    .on_prepaint(move |bounds, _, _| {
                        geometry.borrow_mut().keys.insert(key_id, bounds);
                    })
                    .child(key_button)
                    .into_any_element()
            }))
    }

    pub(super) fn render_compact_keyboard(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let width = rems(self.keyboard_canvas_width());
        v_flex()
            .w(width)
            .min_w(width)
            .max_w(width)
            .flex_shrink_0()
            .gap(rems(0.25))
            .child(self.render_key_row(&ROW_1, cx))
            .child(self.render_key_row(&ROW_2, cx))
            .child(self.render_key_row(&ROW_3, cx))
            .child(self.render_key_row(&ROW_4, cx))
            .child(self.render_key_row(&ROW_5, cx))
            .child(self.render_key_row(&ROW_6, cx))
    }

    pub(super) fn render_tkl_keyboard(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let columns = self.keyboard_column_widths();
        let width = rems(self.keyboard_canvas_width());
        h_flex()
            .w(width)
            .min_w(width)
            .max_w(width)
            .flex_shrink_0()
            .gap(rems(0.5))
            .items_start()
            .child(
                v_flex()
                    .w(rems(columns[0]))
                    .min_w(rems(columns[0]))
                    .max_w(rems(columns[0]))
                    .flex_shrink_0()
                    .gap(rems(0.25))
                    .child(self.render_key_row(&TKL_FUNCTION_ROW, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_1, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_2, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_3, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_4, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_5, cx)),
            )
            .child(
                v_flex()
                    .w(rems(columns[1]))
                    .min_w(rems(columns[1]))
                    .max_w(rems(columns[1]))
                    .flex_shrink_0()
                    .gap(rems(0.25))
                    .child(self.render_key_row(TKL_SYSTEM_ROW, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_1, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_2, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_3, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_4, cx)),
            )
    }

    pub(super) fn render_full_size_keyboard(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let columns = self.keyboard_column_widths();
        let width = rems(self.keyboard_canvas_width());
        h_flex()
            .w(width)
            .min_w(width)
            .max_w(width)
            .flex_shrink_0()
            .gap(rems(0.5))
            .items_start()
            .child(
                v_flex()
                    .w(rems(columns[0]))
                    .min_w(rems(columns[0]))
                    .max_w(rems(columns[0]))
                    .flex_shrink_0()
                    .gap(rems(0.25))
                    .child(self.render_key_row(&TKL_FUNCTION_ROW, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_1, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_2, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_3, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_4, cx))
                    .child(self.render_key_row(TKL_MAIN_ROW_5, cx)),
            )
            .child(
                v_flex()
                    .w(rems(columns[1]))
                    .min_w(rems(columns[1]))
                    .max_w(rems(columns[1]))
                    .flex_shrink_0()
                    .gap(rems(0.25))
                    .child(self.render_key_row(TKL_SYSTEM_ROW, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_1, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_2, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_3, cx))
                    .child(self.render_key_row(TKL_NAV_ROW_4, cx)),
            )
            .child(
                v_flex()
                    .w(rems(columns[2]))
                    .min_w(rems(columns[2]))
                    .max_w(rems(columns[2]))
                    .flex_shrink_0()
                    .gap(rems(0.25))
                    .child(self.render_key_row(FULL_NUMPAD_ROW_1, cx))
                    .child(self.render_key_row(FULL_NUMPAD_ROW_2, cx))
                    .child(self.render_key_row(FULL_NUMPAD_ROW_3, cx))
                    .child(self.render_key_row(FULL_NUMPAD_ROW_4, cx))
                    .child(self.render_key_row(FULL_NUMPAD_ROW_5, cx)),
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
                    .on_click(cx.listener(move |this, event: &ClickEvent, _, cx| {
                        this.toggle_selection_group(group_id, &keys, event.modifiers().control, cx);
                    }))
                    .into_any_element()
            }))
            .child(
                Button::new("clear-selected-keys")
                    .danger()
                    .outline()
                    .compact()
                    .label("Clear selection")
                    .tooltip("Clear selection without removing sound assignments")
                    .disabled(self.selected_keys[self.keyboard_layout.index()].is_empty())
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.clear_selected_keys(cx);
                    })),
            )
    }
}

fn keyboard_row_width(row: &[KeySpec]) -> f32 {
    row.iter().map(|key| key.width / 16.).sum::<f32>() + row.len().saturating_sub(1) as f32 * 0.25
}

fn widest_keyboard_row(rows: &[&[KeySpec]]) -> f32 {
    rows.iter()
        .map(|row| keyboard_row_width(row))
        .fold(0., f32::max)
}
