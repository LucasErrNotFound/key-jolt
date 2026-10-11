pub(super) fn marquee_selection(
    baseline: &[&'static str],
    hits: &[&'static str],
    additive: bool,
) -> Vec<&'static str> {
    if !additive {
        return hits.to_vec();
    }
    let mut selection = baseline.to_vec();
    for key in hits {
        if let Some(index) = selection.iter().position(|selected| selected == key) {
            selection.remove(index);
        } else {
            selection.push(key);
        }
    }
    selection
}

#[cfg(test)]
mod tests {
    use super::marquee_selection;

    #[test]
    fn drag_replaces_previous_selection() {
        assert_eq!(
            marquee_selection(&["a", "b"], &["space"], false),
            vec!["space"]
        );
    }

    #[test]
    fn control_drag_uses_the_original_snapshot() {
        let baseline = ["a", "b"];
        assert_eq!(
            marquee_selection(&baseline, &["b", "c"], true),
            vec!["a", "c"]
        );
        assert_eq!(
            marquee_selection(&baseline, &["b", "c", "d"], true),
            vec!["a", "c", "d"]
        );
        assert_eq!(marquee_selection(&baseline, &["b"], true), vec!["a"]);
    }

    #[test]
    fn empty_drag_preserves_only_additive_selection() {
        assert_eq!(marquee_selection(&["a"], &[], true), vec!["a"]);
        assert!(marquee_selection(&["a"], &[], false).is_empty());
    }
}
