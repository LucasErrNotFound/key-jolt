use super::assignment::{
    SelectionAssignment, assignment_groups, assignment_state, clear_inactive_assignment_mirrors,
    maximum_assigned_sound_count, set_layout_sound_assignment, set_sound_assignment,
};
use super::layout::KeyboardLayout;
use super::model::AudioFile;
use super::selection::{select_key_selection, set_selection_sync, toggle_key_selection};
use crate::presets::canonical_key_identifier;

#[test]
fn ending_sync_after_changing_size_clears_the_previous_sources_mirrors() {
    for source in KeyboardLayout::ALL {
        for active in KeyboardLayout::ALL {
            let mut file = AudioFile::new(0, "sound.wav".into(), "Sound");
            file.assigned_keys[source.index()] = vec!["a".into(), "left_shift".into()];
            file.begin_assignment_sync(source);
            file.end_assignment_sync(active);
            clear_inactive_assignment_mirrors([(file.id, &mut file.assigned_keys)], active);

            for layout in KeyboardLayout::ALL {
                if layout == active {
                    assert_eq!(file.assigned_keys[layout.index()], vec!["a", "left_shift"]);
                } else {
                    assert!(file.assigned_keys[layout.index()].is_empty());
                }
            }
            assert!(file.independent_assigned_keys.is_none());
        }
    }
}

#[test]
fn clearing_mirrors_preserves_entire_different_sound_pools_and_layout_only_keys() {
    let mut first = [
        vec!["a".into(), "b".into(), "num_0".into()],
        vec!["a".into(), "b".into()],
        vec!["a".into(), "b".into()],
    ];
    let mut second = [vec!["a".into()], Vec::new(), vec!["b".into()]];
    clear_inactive_assignment_mirrors([(0, &mut first), (1, &mut second)], KeyboardLayout::Tkl);

    assert_eq!(first[0], vec!["a", "num_0"]);
    assert_eq!(second[0], vec!["a"]);
    assert_eq!(first[1], vec!["a", "b"]);
    assert!(second[1].is_empty());
    assert_eq!(first[2], vec!["b"]);
    assert_eq!(second[2], vec!["b"]);
}

#[test]
fn clearing_mirrors_compares_complete_sound_pools_using_canonical_keys() {
    let mut first = [
        vec!["shift_left".into()],
        vec!["left_shift".into()],
        vec!["left_shift".into()],
    ];
    let mut second = [
        vec!["left_shift".into()],
        vec!["shift_left".into()],
        vec!["shift_left".into()],
    ];
    clear_inactive_assignment_mirrors([(9, &mut first), (3, &mut second)], KeyboardLayout::Tkl);

    assert_eq!(first[1], vec!["left_shift"]);
    assert_eq!(second[1], vec!["shift_left"]);
    for index in [0, 2] {
        assert!(first[index].is_empty());
        assert!(second[index].is_empty());
    }
}

#[test]
fn clearing_mirrors_with_no_active_assignments_preserves_inactive_assignments() {
    let mut keys = [vec!["num_0".into()], Vec::new(), vec!["a".into()]];
    let original = keys.clone();
    clear_inactive_assignment_mirrors([(0, &mut keys)], KeyboardLayout::Tkl);
    assert_eq!(keys, original);
}

#[test]
fn synchronized_preview_copies_supported_keys_and_restores_independent_colors() {
    let mut file = AudioFile::new(0, "sound.wav".into(), "Sound");
    file.assigned_keys = [
        vec!["b".into(), "num_plus_1".into()],
        vec!["c".into()],
        vec!["a".into(), "left_shift".into()],
    ];
    let original = file.assigned_keys.clone();
    file.begin_assignment_sync(KeyboardLayout::Compact);
    assert_eq!(
        file.assigned_keys[0],
        vec!["a", "b", "left_shift", "num_plus_1"]
    );
    assert_eq!(file.assigned_keys[1], vec!["a", "c", "left_shift"]);
    assert_eq!(file.assigned_keys[2], vec!["a", "left_shift"]);
    assert_eq!(file.edited_assigned_keys(), &original);
    assert!(
        assignment_groups([(0, file.assigned_keys[0].as_slice())])
            .keys
            .contains_key("a")
    );
    file.end_assignment_sync(KeyboardLayout::Compact);
    assert_eq!(file.assigned_keys, original);
    assert!(
        !assignment_groups([(0, file.assigned_keys[0].as_slice())])
            .keys
            .contains_key("a")
    );
}

#[test]
fn synced_sound_edits_survive_preview_rollback_without_keeping_automatic_copies() {
    let mut file = AudioFile::new(0, "sound.wav".into(), "Sound");
    file.assigned_keys[2] = vec!["a".into()];
    file.begin_assignment_sync(KeyboardLayout::Compact);
    let selected = [vec!["space"], vec!["space"], vec!["space"]];
    file.set_sound_assignment(&selected, KeyboardLayout::Compact, true, true);
    file.end_assignment_sync(KeyboardLayout::Compact);
    assert_eq!(
        file.assigned_keys,
        [Vec::new(), Vec::new(), vec!["a", "space"]]
    );
}

#[test]
fn synchronized_sound_removal_updates_original_mappings_as_well_as_preview() {
    let mut file = AudioFile::new(0, "sound.wav".into(), "Sound");
    file.assigned_keys = [vec!["b".into()], vec!["c".into()], vec!["a".into()]];
    file.begin_assignment_sync(KeyboardLayout::Compact);
    let selected = [vec!["a"], vec!["a"], vec!["a"]];
    file.set_sound_assignment(&selected, KeyboardLayout::FullSize, true, false);
    file.end_assignment_sync(KeyboardLayout::Compact);
    assert_eq!(file.assigned_keys, [vec!["b"], vec!["c"], Vec::new()]);
}

#[test]
fn beginning_sync_again_does_not_replace_the_original_snapshot() {
    let mut file = AudioFile::new(0, "sound.wav".into(), "Sound");
    file.assigned_keys[2] = vec!["a".into()];
    let original = file.assigned_keys.clone();
    file.begin_assignment_sync(KeyboardLayout::Compact);
    file.begin_assignment_sync(KeyboardLayout::FullSize);
    file.end_assignment_sync(KeyboardLayout::Compact);
    assert_eq!(file.assigned_keys, original);
}

#[test]
fn disabling_sync_keeps_the_active_layouts_visible_assignments() {
    let mut file = AudioFile::new(0, "sound.wav".into(), "Sound");
    file.assigned_keys[2] = vec!["a".into()];
    file.begin_assignment_sync(KeyboardLayout::Compact);
    file.end_assignment_sync(KeyboardLayout::Tkl);
    assert_eq!(file.assigned_keys[1], vec!["a"]);
    assert!(file.assigned_keys[0].is_empty());
    assert_eq!(file.assigned_keys[2], vec!["a"]);
}

#[test]
fn a_new_sound_uploaded_during_sync_keeps_explicit_assignments() {
    let mut file = AudioFile::new(0, "sound.wav".into(), "Sound");
    let selected = [vec!["space"], vec!["space"], vec!["space"]];
    file.set_sound_assignment(&selected, KeyboardLayout::Tkl, true, true);
    file.end_assignment_sync(KeyboardLayout::Tkl);
    assert_eq!(file.assigned_keys, [Vec::new(), vec!["space"], Vec::new()]);
}

#[test]
fn full_size_sync_preview_never_copies_numpad_keys_into_smaller_layouts() {
    let mut file = AudioFile::new(0, "sound.wav".into(), "Sound");
    file.assigned_keys[0] = vec!["a".into(), "num_plus_1".into()];
    file.begin_assignment_sync(KeyboardLayout::FullSize);
    assert_eq!(file.assigned_keys[0], vec!["a", "num_plus_1"]);
    assert_eq!(file.assigned_keys[1], vec!["a"]);
    assert_eq!(file.assigned_keys[2], vec!["a"]);
    file.end_assignment_sync(KeyboardLayout::FullSize);
    assert!(file.assigned_keys[1].is_empty());
    assert!(file.assigned_keys[2].is_empty());
}

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
fn synchronized_sound_edits_preserve_other_keys_and_survive_disabling_sync() {
    let mut assigned = [
        vec!["b".to_string()],
        vec!["c".to_string()],
        vec!["a".to_string(), "left_shift".to_string()],
    ];
    let mut selected = [Vec::new(), Vec::new(), vec!["space"]];
    set_selection_sync(&mut selected, KeyboardLayout::Compact, true);
    assert_eq!(assigned[0], vec!["b"]);
    assert_eq!(assigned[1], vec!["c"]);
    assert_eq!(assigned[2], vec!["a", "left_shift"]);
    set_layout_sound_assignment(
        &mut assigned,
        &selected,
        KeyboardLayout::Compact,
        true,
        true,
    );
    set_selection_sync(&mut selected, KeyboardLayout::Compact, false);
    assert_eq!(assigned[0], vec!["b", "space"]);
    assert_eq!(assigned[1], vec!["c", "space"]);
    assert_eq!(assigned[2], vec!["a", "left_shift", "space"]);
    set_layout_sound_assignment(
        &mut assigned,
        &selected,
        KeyboardLayout::Compact,
        false,
        false,
    );
    assert_eq!(assigned[0], vec!["b", "space"]);
    assert_eq!(assigned[1], vec!["c", "space"]);
    assert_eq!(assigned[2], vec!["a", "left_shift"]);
}

#[test]
fn sync_on_off_clears_inactive_selection_and_keeps_the_current_target() {
    let mut selected = [Vec::new(), Vec::new(), vec!["shift_left"]];
    set_selection_sync(&mut selected, KeyboardLayout::Compact, true);
    assert_eq!(
        selected,
        [vec!["left_shift"], vec!["left_shift"], vec!["shift_left"]]
    );
    set_selection_sync(&mut selected, KeyboardLayout::Compact, false);
    assert_eq!(selected, [Vec::new(), Vec::new(), vec!["shift_left"]]);
}

#[test]
fn disabling_sync_after_switching_sizes_keeps_only_the_active_selection() {
    let mut selected = [Vec::new(), Vec::new(), vec!["space"]];
    set_selection_sync(&mut selected, KeyboardLayout::Compact, true);
    set_selection_sync(&mut selected, KeyboardLayout::Tkl, false);
    assert_eq!(selected, [Vec::new(), vec!["space"], Vec::new()]);
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
