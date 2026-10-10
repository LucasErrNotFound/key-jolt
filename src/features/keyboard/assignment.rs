use super::layout::KeyboardLayout;
use crate::presets::canonical_key_identifier;
use std::collections::{BTreeMap, BTreeSet};

pub(super) struct AssignmentGroups {
    pub(super) keys: BTreeMap<String, usize>,
    pub(super) sounds: Vec<Vec<u64>>,
}

pub(super) fn assignment_groups<'a>(
    assignments: impl IntoIterator<Item = (u64, &'a [String])>,
) -> AssignmentGroups {
    let mut sounds_by_key = BTreeMap::<String, BTreeSet<u64>>::new();
    for (id, keys) in assignments {
        for key in keys {
            sounds_by_key
                .entry(canonical_key_identifier(key).to_string())
                .or_default()
                .insert(id);
        }
    }
    let mut groups = AssignmentGroups {
        keys: BTreeMap::new(),
        sounds: Vec::new(),
    };
    for (key, sounds) in sounds_by_key {
        let sounds = sounds.into_iter().collect::<Vec<_>>();
        let index = groups
            .sounds
            .iter()
            .position(|group| group == &sounds)
            .unwrap_or_else(|| {
                groups.sounds.push(sounds);
                groups.sounds.len() - 1
            });
        groups.keys.insert(key, index);
    }
    groups
}

pub(super) fn clear_inactive_assignment_mirrors<'a>(
    assignments: impl IntoIterator<Item = (u64, &'a mut [Vec<String>; 3])>,
    active_layout: KeyboardLayout,
) {
    let mut assignments = assignments.into_iter().collect::<Vec<_>>();
    let active = assignment_groups(
        assignments
            .iter()
            .map(|(id, keys)| (*id, keys[active_layout.index()].as_slice())),
    );
    for layout in KeyboardLayout::ALL {
        if layout == active_layout {
            continue;
        }
        let inactive = assignment_groups(
            assignments
                .iter()
                .map(|(id, keys)| (*id, keys[layout.index()].as_slice())),
        );
        let mirrors = inactive
            .keys
            .iter()
            .filter(|(key, group)| {
                active.keys.get(*key).is_some_and(|active_group| {
                    active.sounds[*active_group] == inactive.sounds[**group]
                })
            })
            .map(|(key, _)| key.clone())
            .collect::<BTreeSet<_>>();
        for (_, keys) in &mut assignments {
            keys[layout.index()].retain(|key| !mirrors.contains(canonical_key_identifier(key)));
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum SelectionAssignment {
    None,
    All,
    Mixed,
}

pub(super) fn assignment_state(
    assigned_keys: &[String],
    selected_keys: &[&str],
) -> SelectionAssignment {
    let selected = selected_keys
        .iter()
        .map(|key| canonical_key_identifier(key))
        .collect::<BTreeSet<_>>();
    let assigned_count = selected
        .iter()
        .filter(|key| assigned_keys.iter().any(|assigned| assigned == **key))
        .count();
    if assigned_count == 0 {
        SelectionAssignment::None
    } else if assigned_count == selected.len() {
        SelectionAssignment::All
    } else {
        SelectionAssignment::Mixed
    }
}

pub(super) fn set_sound_assignment(
    assigned_keys: &mut Vec<String>,
    selected_keys: &[&str],
    enabled: bool,
) {
    let selected = selected_keys
        .iter()
        .map(|key| canonical_key_identifier(key).to_string())
        .collect::<BTreeSet<_>>();
    if enabled {
        assigned_keys.extend(selected);
        assigned_keys.sort();
        assigned_keys.dedup();
    } else {
        assigned_keys.retain(|key| !selected.contains(key));
    }
}

pub(super) fn maximum_assigned_sound_count<'a>(
    assignments: impl IntoIterator<Item = &'a [String]>,
) -> usize {
    let mut counts = BTreeMap::new();
    for keys in assignments {
        for key in keys.iter().collect::<BTreeSet<_>>() {
            *counts.entry(key).or_insert(0) += 1;
        }
    }
    counts.into_values().max().unwrap_or(0)
}

pub(super) fn set_layout_sound_assignment(
    assigned_keys: &mut [Vec<String>; 3],
    selected_keys: &[Vec<&'static str>; 3],
    active_layout: KeyboardLayout,
    sync: bool,
    enabled: bool,
) {
    for layout in KeyboardLayout::ALL {
        if sync || layout == active_layout {
            set_sound_assignment(
                &mut assigned_keys[layout.index()],
                &selected_keys[layout.index()],
                enabled,
            );
        }
    }
}

pub(super) fn synchronize_sound_assignments(
    assigned_keys: &mut [Vec<String>; 3],
    active_layout: KeyboardLayout,
) {
    let source = assigned_keys[active_layout.index()].clone();
    for layout in KeyboardLayout::ALL {
        if layout == active_layout {
            continue;
        }
        let keys = layout
            .key_ids()
            .into_iter()
            .filter(|key| {
                source
                    .iter()
                    .any(|assigned| assigned == canonical_key_identifier(key))
            })
            .collect::<Vec<_>>();
        set_sound_assignment(&mut assigned_keys[layout.index()], &keys, true);
    }
}
