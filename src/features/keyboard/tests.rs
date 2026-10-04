use super::layout::KeyboardLayout;
use super::selection::toggle_key_selection;
use crate::presets::canonical_key_identifier;

#[test]
fn every_layout_contains_function_keys_and_print_screen() {
    for layout in KeyboardLayout::ALL {
        let keys = layout.key_ids();
        for key in [
            "f1",
            "f2",
            "f3",
            "f4",
            "f5",
            "f6",
            "f7",
            "f8",
            "f9",
            "f10",
            "f11",
            "f12",
            "print_screen",
        ] {
            assert!(keys.contains(&key));
        }
        assert!(keys.iter().all(|key| !key.starts_with("nav_up_spacer")));
    }
}

#[test]
fn synchronized_modifier_selection_uses_each_layouts_identifier() {
    let mut selected = [Vec::new(), Vec::new(), Vec::new()];
    toggle_key_selection(&mut selected, KeyboardLayout::Compact, "shift_left", true);
    assert_eq!(
        selected,
        [vec!["left_shift"], vec!["left_shift"], vec!["shift_left"]]
    );

    toggle_key_selection(&mut selected, KeyboardLayout::Tkl, "left_shift", true);
    assert!(selected.iter().all(Vec::is_empty));
}

#[test]
fn independent_selection_changes_only_the_current_layout() {
    let mut selected = [vec!["a"], Vec::new(), vec!["b"]];
    toggle_key_selection(&mut selected, KeyboardLayout::Tkl, "c", false);
    assert_eq!(selected, [vec!["a"], vec!["c"], vec!["b"]]);
}

#[test]
fn synchronized_numpad_selection_stays_in_the_full_size_layout() {
    let mut selected = [Vec::new(), Vec::new(), Vec::new()];
    toggle_key_selection(&mut selected, KeyboardLayout::FullSize, "num_plus_2", true);
    assert_eq!(selected, [vec!["num_plus_1"], Vec::new(), Vec::new()]);
    assert_eq!(
        canonical_key_identifier("num_plus_1"),
        canonical_key_identifier("num_plus_2")
    );
}
