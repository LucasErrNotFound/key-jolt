use super::assignment::{
    SelectionAssignment, assignment_groups, assignment_state, maximum_assigned_sound_count,
    set_layout_sound_assignment, set_sound_assignment, synchronize_sound_assignments,
};
use super::layout::KeyboardLayout;
use super::selection::select_key_selection;
use super::selection::toggle_key_selection;
use crate::presets::canonical_key_identifier;

#[test]
fn unsynchronized_assignments_and_colors_stay_in_the_active_layout() {
    let mut assigned = std::array::from_fn(|_| Vec::new());
    let selected = [Vec::new(), Vec::new(), vec!["a", "space"]];
    set_layout_sound_assignment(
        &mut assigned,
        &selected,
        KeyboardLayout::Compact,
        false,
        true,
    );
    assert_eq!(assigned[2], vec!["a", "space"]);
    for index in [0, 1] {
        assert!(assigned[index].is_empty());
        assert!(
            assignment_groups([(0, assigned[index].as_slice())])
                .keys
                .is_empty()
        );
    }
    assert_eq!(
        assignment_groups([(0, assigned[2].as_slice())]).keys.len(),
        2
    );
}

#[test]
fn unsynchronized_layouts_can_assign_different_sounds_to_the_same_key() {
    let mut a = std::array::from_fn(|_| Vec::new());
    let mut b = std::array::from_fn(|_| Vec::new());
    let mut selected = [vec!["space"], Vec::new(), vec!["space"]];
    set_layout_sound_assignment(&mut a, &selected, KeyboardLayout::Compact, false, true);
    set_layout_sound_assignment(&mut b, &selected, KeyboardLayout::FullSize, false, true);
    assert_eq!(
        assignment_state(&a[2], &selected[2]),
        SelectionAssignment::All
    );
    assert_eq!(
        assignment_state(&a[0], &selected[0]),
        SelectionAssignment::None
    );
    assert_eq!(
        assignment_state(&b[0], &selected[0]),
        SelectionAssignment::All
    );
    selected[2].clear();
    assert_eq!(a[2], vec!["space"]);
    set_layout_sound_assignment(&mut b, &selected, KeyboardLayout::FullSize, false, false);
    assert!(b.iter().all(Vec::is_empty));
    assert_eq!(a[2], vec!["space"]);
}

#[test]
fn synchronized_assignment_changes_match_modifiers_and_keep_numpad_local() {
    let mut assigned = std::array::from_fn(|_| Vec::new());
    let mut selected = std::array::from_fn(|_| Vec::new());
    toggle_key_selection(&mut selected, KeyboardLayout::FullSize, "left_shift", true);
    toggle_key_selection(&mut selected, KeyboardLayout::FullSize, "num_plus_2", true);
    set_layout_sound_assignment(
        &mut assigned,
        &selected,
        KeyboardLayout::FullSize,
        true,
        true,
    );
    assert_eq!(assigned[0], vec!["left_shift", "num_plus_1"]);
    assert_eq!(assigned[1], vec!["left_shift"]);
    assert_eq!(assigned[2], vec!["left_shift"]);
    set_layout_sound_assignment(
        &mut assigned,
        &selected,
        KeyboardLayout::FullSize,
        true,
        false,
    );
    assert!(assigned.iter().all(Vec::is_empty));
}

#[test]
fn enabling_sync_merges_matching_assignments_without_erasing_existing_keys() {
    let mut assigned = [
        vec!["b".to_string()],
        vec!["c".to_string()],
        vec!["a".to_string(), "left_shift".to_string()],
    ];
    synchronize_sound_assignments(&mut assigned, KeyboardLayout::Compact);
    assert_eq!(assigned[0], vec!["a", "b", "left_shift"]);
    assert_eq!(assigned[1], vec!["a", "c", "left_shift"]);
    assert_eq!(assigned[2], vec!["a", "left_shift"]);
    let selected = [vec!["a"], vec!["a"], vec!["a"]];
    set_layout_sound_assignment(
        &mut assigned,
        &selected,
        KeyboardLayout::Compact,
        false,
        false,
    );
    assert_eq!(assigned[0], vec!["a", "b", "left_shift"]);
    assert_eq!(assigned[2], vec!["left_shift"]);
}

#[test]
fn assignment_groups_distinguish_shared_and_overlapping_sound_sets() {
    let sound_a = vec!["a".into(), "b".into(), "enter".into()];
    let sound_b = vec!["space".into(), "enter".into()];
    let groups = assignment_groups([(10, sound_a.as_slice()), (20, sound_b.as_slice())]);
    assert_eq!(groups.keys["a"], groups.keys["b"]);
    assert_ne!(groups.keys["a"], groups.keys["space"]);
    assert_ne!(groups.keys["enter"], groups.keys["space"]);
    assert_ne!(groups.keys["enter"], groups.keys["a"]);
    assert_eq!(groups.sounds[groups.keys["enter"]], vec![10, 20]);
    assert!(!groups.keys.contains_key("esc"));
}

#[test]
fn assignment_groups_normalize_aliases_and_ignore_duplicate_membership() {
    let keys = vec![
        "shift_left".into(),
        "left_shift".into(),
        "a".into(),
        "a".into(),
    ];
    let groups = assignment_groups([(7, keys.as_slice()), (7, keys.as_slice())]);
    assert_eq!(groups.keys.len(), 2);
    assert_eq!(groups.keys["left_shift"], groups.keys["a"]);
    assert_eq!(groups.sounds, vec![vec![7]]);
    assert!(assignment_groups(std::iter::empty()).keys.is_empty());
}

#[test]
fn assignment_group_labels_survive_sound_reordering_and_recreated_file_ids() {
    let keys_a = vec!["a".into(), "b".into()];
    let keys_b = vec!["space".into()];
    let original = assignment_groups([(4, keys_a.as_slice()), (9, keys_b.as_slice())]);
    let restored = assignment_groups([(0, keys_b.as_slice()), (1, keys_a.as_slice())]);
    assert_eq!(original.keys, restored.keys);
}

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

#[test]
fn selecting_space_does_not_inherit_the_letters_sound() {
    let letters = [
        "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r",
        "s", "t", "u", "v", "w", "x", "y", "z",
    ];
    let mut selected = [Vec::new(), Vec::new(), letters.to_vec()];
    let mut sound_a = Vec::new();
    let mut sound_b = Vec::new();
    set_sound_assignment(&mut sound_a, &letters, true);

    select_key_selection(
        &mut selected,
        KeyboardLayout::Compact,
        "space",
        false,
        false,
    );
    assert_eq!(selected[2], vec!["space"]);
    assert_eq!(
        assignment_state(&sound_a, &selected[2]),
        SelectionAssignment::None
    );
    set_sound_assignment(&mut sound_b, &selected[2], true);

    assert_eq!(sound_a.len(), 26);
    assert!(!sound_a.iter().any(|key| key == "space"));
    assert_eq!(sound_b, vec!["space"]);
    assert_eq!(
        assignment_state(&sound_a, &letters),
        SelectionAssignment::All
    );
}

#[test]
fn replacing_one_keys_sound_preserves_the_other_assignments() {
    let mut sound_a = Vec::new();
    set_sound_assignment(&mut sound_a, &["a", "b", "space"], true);
    set_sound_assignment(&mut sound_a, &["space"], false);
    let mut sound_b = Vec::new();
    set_sound_assignment(&mut sound_b, &["space"], true);
    assert_eq!(sound_a, vec!["a", "b"]);
    assert_eq!(sound_b, vec!["space"]);
}

#[test]
fn control_click_adds_and_removes_keys_without_replacing_the_group() {
    let mut selected = [Vec::new(), Vec::new(), vec!["a", "b"]];
    select_key_selection(&mut selected, KeyboardLayout::Compact, "space", false, true);
    assert_eq!(selected[2], vec!["a", "b", "space"]);
    select_key_selection(&mut selected, KeyboardLayout::Compact, "a", false, true);
    assert_eq!(selected[2], vec!["b", "space"]);
}

#[test]
fn plain_click_replaces_synchronized_selection_in_every_layout() {
    let mut selected = [vec!["a", "b"], vec!["a", "b"], vec!["a", "b"]];
    select_key_selection(
        &mut selected,
        KeyboardLayout::Compact,
        "shift_left",
        true,
        false,
    );
    assert_eq!(
        selected,
        [vec!["left_shift"], vec!["left_shift"], vec!["shift_left"]]
    );
}

#[test]
fn sound_checkboxes_reflect_all_none_and_mixed_assignments() {
    let assigned = vec!["a".to_string(), "b".to_string()];
    assert_eq!(
        assignment_state(&assigned, &["a", "b"]),
        SelectionAssignment::All
    );
    assert_eq!(
        assignment_state(&assigned, &["space"]),
        SelectionAssignment::None
    );
    assert_eq!(
        assignment_state(&assigned, &["a", "space"]),
        SelectionAssignment::Mixed
    );
    assert_eq!(assignment_state(&assigned, &[]), SelectionAssignment::None);
}

#[test]
fn assignments_share_canonical_keys_without_duplicates() {
    let mut assigned = Vec::new();
    set_sound_assignment(&mut assigned, &["left_shift", "shift_left", "a"], true);
    assert_eq!(assigned.len(), 2);
    assert_eq!(
        assignment_state(&assigned, &["left_shift"]),
        SelectionAssignment::All
    );
    set_sound_assignment(&mut assigned, &["shift_left"], false);
    assert_eq!(assigned, vec!["a"]);
}

#[test]
fn random_playback_requires_multiple_sounds_for_the_same_key() {
    let a = vec!["a".to_string()];
    let b = vec!["space".to_string()];
    assert_eq!(
        maximum_assigned_sound_count([a.as_slice(), b.as_slice()]),
        1
    );
    let c = vec!["a".to_string()];
    assert_eq!(
        maximum_assigned_sound_count([a.as_slice(), b.as_slice(), c.as_slice()]),
        2
    );
}
