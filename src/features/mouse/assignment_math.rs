use std::collections::BTreeMap;

pub(super) fn copy_changes_assignments(rows: &[[bool; 3]], source: usize) -> bool {
    source < 3
        && rows.iter().any(|row| row[source])
        && (0..3)
            .any(|target| target != source && rows.iter().any(|row| row[target] != row[source]))
}

pub(super) fn copy_requires_confirmation(rows: &[[bool; 3]], source: usize) -> bool {
    copy_changes_assignments(rows, source)
        && (0..3).any(|target| {
            target != source
                && rows.iter().any(|row| row[target])
                && rows.iter().any(|row| row[target] != row[source])
        })
}

pub(super) fn copy_pool(rows: &mut [[bool; 3]], source: usize) -> bool {
    if !copy_changes_assignments(rows, source) {
        return false;
    }
    for row in rows {
        *row = [row[source]; 3];
    }
    true
}

pub(super) struct BulkAssignmentUndo {
    before: BTreeMap<u64, [bool; 3]>,
    applied: BTreeMap<u64, [bool; 3]>,
}

impl BulkAssignmentUndo {
    pub(super) fn apply(rows: &mut [(u64, [bool; 3])], source: usize) -> Option<Self> {
        let before: BTreeMap<_, _> = rows.iter().copied().collect();
        if before.len() != rows.len() {
            return None;
        }
        let mut flags: Vec<_> = rows.iter().map(|(_, flags)| *flags).collect();
        if !copy_pool(&mut flags, source) {
            return None;
        }
        for ((_, assignments), copied) in rows.iter_mut().zip(flags) {
            *assignments = copied;
        }
        Some(Self {
            before,
            applied: rows.iter().copied().collect(),
        })
    }

    pub(super) fn restore(&self, rows: &mut [(u64, [bool; 3])]) -> bool {
        let current: BTreeMap<_, _> = rows.iter().copied().collect();
        if current.len() != rows.len() || current != self.applied {
            return false;
        }
        for (id, assignments) in rows {
            if let Some(before) = self.before.get(id) {
                *assignments = *before;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assigning_empty_destinations_needs_no_confirmation() {
        let mut rows = [[true, false, false], [true, false, false]];
        assert!(!copy_requires_confirmation(&rows, 0));
        assert!(copy_pool(&mut rows, 0));
        assert_eq!(rows, [[true; 3], [true; 3]]);
    }

    #[test]
    fn replacing_a_different_pool_requires_confirmation() {
        let mut rows = [[true, false, true], [false, true, false]];
        assert!(copy_requires_confirmation(&rows, 0));
        assert!(copy_pool(&mut rows, 0));
        assert_eq!(rows, [[true; 3], [false; 3]]);
    }

    #[test]
    fn adding_to_an_existing_subset_requires_confirmation() {
        assert!(copy_requires_confirmation(
            &[[true, true, false], [true, false, false]],
            0,
        ));
    }

    #[test]
    fn identical_pools_are_unchanged() {
        let mut rows = [[true; 3], [false; 3]];
        assert!(!copy_requires_confirmation(&rows, 1));
        assert!(!copy_pool(&mut rows, 1));
    }

    #[test]
    fn empty_source_cannot_clear_other_buttons() {
        let mut rows = [[false, true, true]];
        assert!(!copy_pool(&mut rows, 0));
        assert_eq!(rows, [[false, true, true]]);
    }

    #[test]
    fn every_source_button_copies_its_entire_pool() {
        for source in 0..3 {
            let mut rows = [
                [true, false, false],
                [false, true, false],
                [false, false, true],
            ];
            assert!(copy_pool(&mut rows, source));
            for (index, row) in rows.iter().enumerate() {
                assert_eq!(*row, [index == source; 3]);
            }
        }
    }

    #[test]
    fn invalid_source_is_ignored() {
        let mut rows = [[true, false, false]];
        assert!(!copy_requires_confirmation(&rows, 3));
        assert!(!copy_pool(&mut rows, 3));
    }
    #[test]
    fn undo_restores_different_button_pools_exactly() {
        let original = [(10, [true, false, false]), (22, [false, true, true])];
        let mut rows = original;
        let undo = BulkAssignmentUndo::apply(&mut rows, 0).unwrap();
        assert!(undo.restore(&mut rows));
        assert_eq!(rows, original);
    }

    #[test]
    fn undo_uses_file_ids_after_reordering() {
        let mut rows = [(10, [true, false, true]), (22, [false, true, false])];
        let undo = BulkAssignmentUndo::apply(&mut rows, 0).unwrap();
        rows.reverse();
        assert!(undo.restore(&mut rows));
        assert_eq!(
            rows,
            [(22, [false, true, false]), (10, [true, false, true])]
        );
    }

    #[test]
    fn undo_cannot_overwrite_a_later_assignment_edit() {
        let mut rows = [(10, [true, false, true]), (22, [false, true, false])];
        let undo = BulkAssignmentUndo::apply(&mut rows, 0).unwrap();
        rows[1].1[1] = true;
        let edited = rows;
        assert!(!undo.restore(&mut rows));
        assert_eq!(rows, edited);
    }

    #[test]
    fn undo_cannot_restore_deleted_files_or_touch_new_files() {
        let mut rows = vec![(10, [true, false, true]), (22, [false, true, false])];
        let undo = BulkAssignmentUndo::apply(&mut rows, 0).unwrap();
        rows.remove(1);
        assert!(!undo.restore(&mut rows));
        rows.push((33, [false, true, false]));
        let edited = rows.clone();
        assert!(!undo.restore(&mut rows));
        assert_eq!(rows, edited);
    }

    #[test]
    fn no_op_bulk_action_does_not_create_undo_state() {
        let mut rows = [(10, [true; 3])];
        assert!(BulkAssignmentUndo::apply(&mut rows, 0).is_none());
        assert_eq!(rows, [(10, [true; 3])]);
    }
}
