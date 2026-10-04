use super::KeyboardEditorView;
use super::layout::{
    FULL_NUMPAD_ROW_1, FULL_NUMPAD_ROW_2, FULL_NUMPAD_ROW_3, FULL_NUMPAD_ROW_4, FULL_NUMPAD_ROW_5,
    KeyboardLayout, TKL_FUNCTION_ROW,
};

use super::model::SelectionGroup;
use crate::presets::canonical_key_identifier as canonical_key_id;
use gpui_kit::*;

impl KeyboardEditorView {
    pub(super) fn is_selected(&self, key_id: &str) -> bool {
        self.selected_keys[self.keyboard_layout.index()].contains(&key_id)
    }

    pub(super) fn are_keys_selected(&self, keys: &[&'static str]) -> bool {
        let selected_keys = &self.selected_keys[self.keyboard_layout.index()];
        !keys.is_empty() && keys.iter().all(|key| selected_keys.contains(key))
    }

    pub(super) fn selection_groups(&self) -> Vec<SelectionGroup> {
        const LETTER_KEYS: [&str; 26] = [
            "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q",
            "r", "s", "t", "u", "v", "w", "x", "y", "z",
        ];
        const NUMBER_ROW_KEYS: [&str; 13] = [
            "grave", "1", "2", "3", "4", "5", "6", "7", "8", "9", "0", "minus", "equal",
        ];
        const NAVIGATION_KEYS: [&str; 10] = [
            "insert",
            "home",
            "page_up",
            "delete",
            "end",
            "page_down",
            "arrow_up",
            "arrow_left",
            "arrow_down",
            "arrow_right",
        ];

        let mut groups = vec![SelectionGroup {
            id: "all-keys",
            label: "All keys",
            keys: self.keyboard_layout.key_ids(),
        }];
        groups.push(SelectionGroup {
            id: "letters",
            label: "A–Z",
            keys: LETTER_KEYS.to_vec(),
        });
        groups.push(SelectionGroup {
            id: "number-row",
            label: "Number row",
            keys: NUMBER_ROW_KEYS.to_vec(),
        });
        groups.push(SelectionGroup {
            id: "function-row",
            label: "Function row",
            keys: TKL_FUNCTION_ROW.iter().map(|key| key.id).collect(),
        });

        let modifier_keys = match self.keyboard_layout {
            KeyboardLayout::Compact => vec![
                "shift_left",
                "shift_right",
                "ctrl_left",
                "ctrl_right",
                "win_left",
                "win_right",
                "alt_left",
                "alt_right",
            ],
            KeyboardLayout::Tkl | KeyboardLayout::FullSize => vec![
                "left_shift",
                "right_shift",
                "left_ctrl",
                "right_ctrl",
                "left_meta",
                "right_meta",
                "left_alt",
                "right_alt",
            ],
        };
        groups.push(SelectionGroup {
            id: "modifiers",
            label: "Modifiers",
            keys: modifier_keys,
        });

        if self.keyboard_layout != KeyboardLayout::Compact {
            groups.push(SelectionGroup {
                id: "navigation",
                label: "Navigation keys",
                keys: NAVIGATION_KEYS.to_vec(),
            });
        }

        if self.keyboard_layout == KeyboardLayout::FullSize {
            let mut numpad_keys = Vec::new();
            for row in [
                FULL_NUMPAD_ROW_1,
                FULL_NUMPAD_ROW_2,
                FULL_NUMPAD_ROW_3,
                FULL_NUMPAD_ROW_4,
                FULL_NUMPAD_ROW_5,
            ] {
                numpad_keys.extend(row.iter().map(|key| key.id));
            }
            groups.push(SelectionGroup {
                id: "numpad",
                label: "Numpad",
                keys: numpad_keys,
            });
        }

        groups
    }

    pub(super) fn toggle_selection_group(
        &mut self,
        group_id: &'static str,
        keys: &[&'static str],
        cx: &mut Context<Self>,
    ) {
        let group_is_selected = self.are_keys_selected(keys);

        if self.sync_selections {
            let canonical_keys = keys
                .iter()
                .map(|key| canonical_key_id(key))
                .collect::<Vec<_>>();

            for layout in KeyboardLayout::ALL {
                let matching_keys = if group_id == "all-keys" {
                    layout.key_ids()
                } else {
                    layout
                        .key_ids()
                        .into_iter()
                        .filter(|key| canonical_keys.contains(&canonical_key_id(key)))
                        .collect::<Vec<_>>()
                };
                let selected_keys = &mut self.selected_keys[layout.index()];

                if group_is_selected {
                    selected_keys.retain(|key| !matching_keys.contains(key));
                } else {
                    for key in matching_keys {
                        if !selected_keys.contains(&key) {
                            selected_keys.push(key);
                        }
                    }
                }
            }
        } else {
            let selected_keys = &mut self.selected_keys[self.keyboard_layout.index()];
            if group_is_selected {
                selected_keys.retain(|key| !keys.contains(key));
            } else {
                for key in keys {
                    if !selected_keys.contains(key) {
                        selected_keys.push(*key);
                    }
                }
            }
        }

        cx.notify();
    }

    pub(super) fn clear_selected_keys(&mut self, cx: &mut Context<Self>) {
        if self.sync_selections {
            for selected_keys in &mut self.selected_keys {
                selected_keys.clear();
            }
        } else {
            self.selected_keys[self.keyboard_layout.index()].clear();
        }
        cx.notify();
    }

    pub(super) fn set_sync_selections(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if enabled && !self.sync_selections {
            let current_keys = self.selected_keys[self.keyboard_layout.index()].clone();
            let canonical_keys = current_keys
                .iter()
                .map(|key| canonical_key_id(key))
                .collect::<Vec<_>>();

            for layout in KeyboardLayout::ALL {
                let layout_keys = layout.key_ids();
                self.selected_keys[layout.index()] = layout_keys
                    .into_iter()
                    .filter(|key| canonical_keys.contains(&canonical_key_id(key)))
                    .collect();
            }
        } else if !enabled && self.sync_selections {
            let active_layout = self.keyboard_layout.index();
            for layout in KeyboardLayout::ALL {
                if layout.index() != active_layout {
                    self.selected_keys[layout.index()].clear();
                }
            }
        }

        self.sync_selections = enabled;
        cx.notify();
    }

    pub(super) fn toggle_key(&mut self, key_id: &'static str, cx: &mut Context<Self>) {
        toggle_key_selection(
            &mut self.selected_keys,
            self.keyboard_layout,
            key_id,
            self.sync_selections,
        );

        cx.notify();
    }

    pub(super) fn apply_layout_change(&mut self, next_layout: KeyboardLayout) -> bool {
        if next_layout == self.keyboard_layout {
            return false;
        }

        if !self.sync_selections {
            self.selected_keys[self.keyboard_layout.index()].clear();
        }

        self.keyboard_layout = next_layout;
        true
    }

    pub(super) fn set_layout(&mut self, index: usize, cx: &mut Context<Self>) {
        let next_layout = match index {
            0 => KeyboardLayout::FullSize,
            1 => KeyboardLayout::Tkl,
            _ => KeyboardLayout::Compact,
        };

        self.apply_layout_change(next_layout);

        cx.notify();
    }
}

pub(super) fn toggle_key_selection(
    selected_keys: &mut [Vec<&'static str>; 3],
    keyboard_layout: KeyboardLayout,
    key_id: &'static str,
    sync_selections: bool,
) {
    if sync_selections {
        let canonical_id = canonical_key_id(key_id);
        for layout in KeyboardLayout::ALL {
            let matching_key = layout
                .key_ids()
                .into_iter()
                .find(|key| canonical_key_id(key) == canonical_id);
            let Some(matching_key) = matching_key else {
                continue;
            };
            let layout_keys = &mut selected_keys[layout.index()];
            if let Some(index) = layout_keys.iter().position(|key| *key == matching_key) {
                layout_keys.remove(index);
            } else {
                layout_keys.push(matching_key);
            }
        }
    } else {
        let layout_keys = &mut selected_keys[keyboard_layout.index()];
        if let Some(index) = layout_keys.iter().position(|key| *key == key_id) {
            layout_keys.remove(index);
        } else {
            layout_keys.push(key_id);
        }
    }
}
