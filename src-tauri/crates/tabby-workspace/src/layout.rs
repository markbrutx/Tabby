use std::collections::BTreeSet;

use thiserror::Error;

use crate::ids::PaneId;

// Re-export LayoutPreset from shared kernel — canonical definition lives in tabby-kernel.
pub use tabby_kernel::LayoutPreset;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SplitNode {
    Pane {
        pane_id: PaneId,
    },
    Split {
        direction: SplitDirection,
        ratio: u16,
        first: Box<SplitNode>,
        second: Box<SplitNode>,
    },
}

#[derive(Debug, Error)]
pub enum LayoutError {
    #[error("unsupported layout preset: {0}")]
    UnsupportedPreset(String),
    #[error("pane count must be between 1 and 9, got {0}")]
    UnsupportedPaneCount(usize),
    #[error("layout tree and pane slots are out of sync")]
    OrphanPane,
}

pub fn tree_from_preset(preset: LayoutPreset, pane_ids: &[PaneId]) -> SplitNode {
    match preset {
        LayoutPreset::OneByOne => leaf(&pane_ids[0]),
        LayoutPreset::OneByTwo => hsplit(&pane_ids[0], &pane_ids[1]),
        LayoutPreset::TwoByTwo => SplitNode::Split {
            direction: SplitDirection::Vertical,
            ratio: 500,
            first: Box::new(hsplit(&pane_ids[0], &pane_ids[1])),
            second: Box::new(hsplit(&pane_ids[2], &pane_ids[3])),
        },
        LayoutPreset::TwoByThree => SplitNode::Split {
            direction: SplitDirection::Vertical,
            ratio: 500,
            first: Box::new(hsplit3(&pane_ids[0], &pane_ids[1], &pane_ids[2])),
            second: Box::new(hsplit3(&pane_ids[3], &pane_ids[4], &pane_ids[5])),
        },
        LayoutPreset::ThreeByThree => SplitNode::Split {
            direction: SplitDirection::Vertical,
            ratio: 333,
            first: Box::new(hsplit3(&pane_ids[0], &pane_ids[1], &pane_ids[2])),
            second: Box::new(SplitNode::Split {
                direction: SplitDirection::Vertical,
                ratio: 500,
                first: Box::new(hsplit3(&pane_ids[3], &pane_ids[4], &pane_ids[5])),
                second: Box::new(hsplit3(&pane_ids[6], &pane_ids[7], &pane_ids[8])),
            }),
        },
    }
}

pub fn tree_from_count(pane_ids: &[PaneId]) -> Result<SplitNode, LayoutError> {
    let tree = match pane_ids.len() {
        1 => leaf(&pane_ids[0]),
        2 => hsplit(&pane_ids[0], &pane_ids[1]),
        3 => SplitNode::Split {
            direction: SplitDirection::Horizontal,
            ratio: 333,
            first: Box::new(leaf(&pane_ids[0])),
            second: Box::new(hsplit(&pane_ids[1], &pane_ids[2])),
        },
        4 => SplitNode::Split {
            direction: SplitDirection::Vertical,
            ratio: 500,
            first: Box::new(hsplit(&pane_ids[0], &pane_ids[1])),
            second: Box::new(hsplit(&pane_ids[2], &pane_ids[3])),
        },
        5 => SplitNode::Split {
            direction: SplitDirection::Vertical,
            ratio: 500,
            first: Box::new(hsplit3(&pane_ids[0], &pane_ids[1], &pane_ids[2])),
            second: Box::new(hsplit(&pane_ids[3], &pane_ids[4])),
        },
        6 => SplitNode::Split {
            direction: SplitDirection::Vertical,
            ratio: 500,
            first: Box::new(hsplit3(&pane_ids[0], &pane_ids[1], &pane_ids[2])),
            second: Box::new(hsplit3(&pane_ids[3], &pane_ids[4], &pane_ids[5])),
        },
        7 => SplitNode::Split {
            direction: SplitDirection::Vertical,
            ratio: 500,
            first: Box::new(hsplit4(
                &pane_ids[0],
                &pane_ids[1],
                &pane_ids[2],
                &pane_ids[3],
            )),
            second: Box::new(hsplit3(&pane_ids[4], &pane_ids[5], &pane_ids[6])),
        },
        8 => SplitNode::Split {
            direction: SplitDirection::Vertical,
            ratio: 500,
            first: Box::new(hsplit4(
                &pane_ids[0],
                &pane_ids[1],
                &pane_ids[2],
                &pane_ids[3],
            )),
            second: Box::new(hsplit4(
                &pane_ids[4],
                &pane_ids[5],
                &pane_ids[6],
                &pane_ids[7],
            )),
        },
        9 => SplitNode::Split {
            direction: SplitDirection::Vertical,
            ratio: 333,
            first: Box::new(hsplit3(&pane_ids[0], &pane_ids[1], &pane_ids[2])),
            second: Box::new(SplitNode::Split {
                direction: SplitDirection::Vertical,
                ratio: 500,
                first: Box::new(hsplit3(&pane_ids[3], &pane_ids[4], &pane_ids[5])),
                second: Box::new(hsplit3(&pane_ids[6], &pane_ids[7], &pane_ids[8])),
            }),
        },
        other => return Err(LayoutError::UnsupportedPaneCount(other)),
    };

    Ok(tree)
}

pub fn split_pane(
    root: &SplitNode,
    target_pane_id: &PaneId,
    direction: SplitDirection,
    new_pane_id: &PaneId,
) -> Option<SplitNode> {
    match root {
        SplitNode::Pane { pane_id } if *pane_id == *target_pane_id => Some(SplitNode::Split {
            direction,
            ratio: 500,
            first: Box::new(SplitNode::Pane {
                pane_id: pane_id.clone(),
            }),
            second: Box::new(SplitNode::Pane {
                pane_id: new_pane_id.clone(),
            }),
        }),
        SplitNode::Pane { .. } => None,
        SplitNode::Split {
            direction: existing,
            ratio,
            first,
            second,
        } => {
            if let Some(next_first) = split_pane(first, target_pane_id, direction, new_pane_id) {
                return Some(SplitNode::Split {
                    direction: *existing,
                    ratio: *ratio,
                    first: Box::new(next_first),
                    second: second.clone(),
                });
            }

            if let Some(next_second) = split_pane(second, target_pane_id, direction, new_pane_id) {
                return Some(SplitNode::Split {
                    direction: *existing,
                    ratio: *ratio,
                    first: first.clone(),
                    second: Box::new(next_second),
                });
            }

            None
        }
    }
}

pub fn close_pane(root: &SplitNode, target_pane_id: &PaneId) -> Option<Option<SplitNode>> {
    match root {
        SplitNode::Pane { pane_id } if *pane_id == *target_pane_id => Some(None),
        SplitNode::Pane { .. } => None,
        SplitNode::Split {
            direction,
            ratio,
            first,
            second,
        } => {
            if let Some(result) = close_pane(first, target_pane_id) {
                return Some(match result {
                    Some(next_first) => Some(SplitNode::Split {
                        direction: *direction,
                        ratio: *ratio,
                        first: Box::new(next_first),
                        second: second.clone(),
                    }),
                    None => Some(second.as_ref().clone()),
                });
            }

            if let Some(result) = close_pane(second, target_pane_id) {
                return Some(match result {
                    Some(next_second) => Some(SplitNode::Split {
                        direction: *direction,
                        ratio: *ratio,
                        first: first.clone(),
                        second: Box::new(next_second),
                    }),
                    None => Some(first.as_ref().clone()),
                });
            }

            None
        }
    }
}

pub fn swap_panes(root: &SplitNode, pane_id_a: &PaneId, pane_id_b: &PaneId) -> Option<SplitNode> {
    let pane_ids = collect_pane_ids(root);
    if !pane_ids.contains(pane_id_a) || !pane_ids.contains(pane_id_b) {
        return None;
    }

    Some(swap_panes_inner(root, pane_id_a, pane_id_b))
}

pub fn collect_pane_ids(root: &SplitNode) -> Vec<PaneId> {
    let mut pane_ids = Vec::new();
    collect_pane_ids_inner(root, &mut pane_ids);
    pane_ids
}

pub fn validate_layout(root: &SplitNode, pane_ids: &[PaneId]) -> Result<(), LayoutError> {
    let tree_ids = collect_pane_ids(root).into_iter().collect::<BTreeSet<_>>();
    let slot_ids = pane_ids.iter().cloned().collect::<BTreeSet<_>>();
    if tree_ids != slot_ids {
        return Err(LayoutError::OrphanPane);
    }
    Ok(())
}

fn swap_panes_inner(root: &SplitNode, pane_id_a: &PaneId, pane_id_b: &PaneId) -> SplitNode {
    match root {
        SplitNode::Pane { pane_id } if *pane_id == *pane_id_a => SplitNode::Pane {
            pane_id: pane_id_b.clone(),
        },
        SplitNode::Pane { pane_id } if *pane_id == *pane_id_b => SplitNode::Pane {
            pane_id: pane_id_a.clone(),
        },
        SplitNode::Pane { .. } => root.clone(),
        SplitNode::Split {
            direction,
            ratio,
            first,
            second,
        } => SplitNode::Split {
            direction: *direction,
            ratio: *ratio,
            first: Box::new(swap_panes_inner(first, pane_id_a, pane_id_b)),
            second: Box::new(swap_panes_inner(second, pane_id_a, pane_id_b)),
        },
    }
}

fn collect_pane_ids_inner(node: &SplitNode, pane_ids: &mut Vec<PaneId>) {
    match node {
        SplitNode::Pane { pane_id } => pane_ids.push(pane_id.clone()),
        SplitNode::Split { first, second, .. } => {
            collect_pane_ids_inner(first, pane_ids);
            collect_pane_ids_inner(second, pane_ids);
        }
    }
}

/// Replaces placeholder pane IDs in a template tree with real IDs,
/// walking the tree in-order and substituting one-to-one.
pub fn remap_pane_ids(template: &SplitNode, real_ids: &[PaneId]) -> SplitNode {
    let mut index = 0;
    remap_inner(template, real_ids, &mut index)
}

fn remap_inner(node: &SplitNode, real_ids: &[PaneId], index: &mut usize) -> SplitNode {
    match node {
        SplitNode::Pane { .. } => {
            let pane_id = if *index < real_ids.len() {
                real_ids[*index].clone()
            } else {
                PaneId::from(format!("__overflow_{index}__"))
            };
            *index += 1;
            SplitNode::Pane { pane_id }
        }
        SplitNode::Split {
            direction,
            ratio,
            first,
            second,
        } => SplitNode::Split {
            direction: *direction,
            ratio: *ratio,
            first: Box::new(remap_inner(first, real_ids, index)),
            second: Box::new(remap_inner(second, real_ids, index)),
        },
    }
}

fn leaf(id: &PaneId) -> SplitNode {
    SplitNode::Pane {
        pane_id: id.clone(),
    }
}

fn hsplit(a: &PaneId, b: &PaneId) -> SplitNode {
    SplitNode::Split {
        direction: SplitDirection::Horizontal,
        ratio: 500,
        first: Box::new(leaf(a)),
        second: Box::new(leaf(b)),
    }
}

fn hsplit3(a: &PaneId, b: &PaneId, c: &PaneId) -> SplitNode {
    SplitNode::Split {
        direction: SplitDirection::Horizontal,
        ratio: 333,
        first: Box::new(leaf(a)),
        second: Box::new(SplitNode::Split {
            direction: SplitDirection::Horizontal,
            ratio: 500,
            first: Box::new(leaf(b)),
            second: Box::new(leaf(c)),
        }),
    }
}

fn hsplit4(a: &PaneId, b: &PaneId, c: &PaneId, d: &PaneId) -> SplitNode {
    SplitNode::Split {
        direction: SplitDirection::Horizontal,
        ratio: 500,
        first: Box::new(hsplit(a, b)),
        second: Box::new(hsplit(c, d)),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        close_pane, collect_pane_ids, split_pane, swap_panes, tree_from_count, LayoutPreset,
        SplitDirection,
    };
    use crate::ids::PaneId;

    fn ids(n: usize) -> Vec<PaneId> {
        (1..=n)
            .map(|index| PaneId::from(format!("p{index}")))
            .collect()
    }

    fn pid(s: &str) -> PaneId {
        PaneId::from(String::from(s))
    }

    #[test]
    fn builds_tree_from_count() {
        let tree = tree_from_count(&ids(4)).expect("tree should build");
        assert_eq!(
            collect_pane_ids(&tree),
            vec![pid("p1"), pid("p2"), pid("p3"), pid("p4")]
        );
    }

    #[test]
    fn splits_and_closes_panes() {
        let tree = super::tree_from_preset(LayoutPreset::OneByOne, &ids(1));
        let split = split_pane(&tree, &pid("p1"), SplitDirection::Horizontal, &pid("p2"))
            .expect("split should succeed");
        assert_eq!(collect_pane_ids(&split), vec![pid("p1"), pid("p2")]);

        let collapsed = close_pane(&split, &pid("p2"))
            .expect("close should succeed")
            .expect("tree should remain");
        assert_eq!(collect_pane_ids(&collapsed), vec![pid("p1")]);
    }

    #[test]
    fn swaps_panes() {
        let tree = tree_from_count(&ids(2)).expect("tree should build");
        let swapped = swap_panes(&tree, &pid("p1"), &pid("p2")).expect("swap should succeed");
        assert_eq!(collect_pane_ids(&swapped), vec![pid("p2"), pid("p1")]);
    }

    // -------------------------------------------------------------------------
    // tree_from_count: all valid counts
    // -------------------------------------------------------------------------

    #[test]
    fn tree_from_count_one_pane_is_single_leaf() {
        let tree = tree_from_count(&ids(1)).expect("count 1 should succeed");
        let found = collect_pane_ids(&tree);
        assert_eq!(found, vec![pid("p1")]);
    }

    #[test]
    fn tree_from_count_two_panes() {
        let tree = tree_from_count(&ids(2)).expect("count 2");
        assert_eq!(collect_pane_ids(&tree).len(), 2);
    }

    #[test]
    fn tree_from_count_three_panes() {
        let tree = tree_from_count(&ids(3)).expect("count 3");
        assert_eq!(collect_pane_ids(&tree).len(), 3);
    }

    #[test]
    fn tree_from_count_five_panes() {
        let tree = tree_from_count(&ids(5)).expect("count 5");
        assert_eq!(collect_pane_ids(&tree).len(), 5);
    }

    #[test]
    fn tree_from_count_six_panes() {
        let tree = tree_from_count(&ids(6)).expect("count 6");
        assert_eq!(collect_pane_ids(&tree).len(), 6);
    }

    #[test]
    fn tree_from_count_seven_panes() {
        let tree = tree_from_count(&ids(7)).expect("count 7");
        assert_eq!(collect_pane_ids(&tree).len(), 7);
    }

    #[test]
    fn tree_from_count_eight_panes() {
        let tree = tree_from_count(&ids(8)).expect("count 8");
        assert_eq!(collect_pane_ids(&tree).len(), 8);
    }

    #[test]
    fn tree_from_count_nine_panes() {
        let tree = tree_from_count(&ids(9)).expect("count 9");
        assert_eq!(collect_pane_ids(&tree).len(), 9);
    }

    #[test]
    fn tree_from_count_zero_panes_returns_error() {
        let result = tree_from_count(&ids(0));
        assert!(result.is_err(), "zero panes should produce an error");
    }

    #[test]
    fn tree_from_count_ten_panes_returns_error() {
        let result = tree_from_count(&ids(10));
        assert!(result.is_err(), "10 panes should produce an error");
    }

    #[test]
    fn tree_from_count_preserves_all_pane_ids_in_order() {
        let input = ids(4);
        let tree = tree_from_count(&input).expect("count 4");
        let found = collect_pane_ids(&tree);
        assert_eq!(found, input, "pane IDs should appear in insertion order");
    }

    // -------------------------------------------------------------------------
    // tree_from_preset: all presets
    // -------------------------------------------------------------------------

    #[test]
    fn tree_from_preset_one_by_one_single_leaf() {
        let tree = super::tree_from_preset(LayoutPreset::OneByOne, &ids(1));
        assert_eq!(collect_pane_ids(&tree), vec![pid("p1")]);
    }

    #[test]
    fn tree_from_preset_one_by_two_two_panes() {
        let tree = super::tree_from_preset(LayoutPreset::OneByTwo, &ids(2));
        let found = collect_pane_ids(&tree);
        assert_eq!(found.len(), 2);
        assert_eq!(found, vec![pid("p1"), pid("p2")]);
    }

    #[test]
    fn tree_from_preset_two_by_two_four_panes() {
        let tree = super::tree_from_preset(LayoutPreset::TwoByTwo, &ids(4));
        let found = collect_pane_ids(&tree);
        assert_eq!(found.len(), 4);
    }

    #[test]
    fn tree_from_preset_two_by_three_six_panes() {
        let tree = super::tree_from_preset(LayoutPreset::TwoByThree, &ids(6));
        assert_eq!(collect_pane_ids(&tree).len(), 6);
    }

    #[test]
    fn tree_from_preset_three_by_three_nine_panes() {
        let tree = super::tree_from_preset(LayoutPreset::ThreeByThree, &ids(9));
        assert_eq!(collect_pane_ids(&tree).len(), 9);
    }

    // -------------------------------------------------------------------------
    // split_pane
    // -------------------------------------------------------------------------

    #[test]
    fn split_pane_on_nonexistent_target_returns_none() {
        let tree = tree_from_count(&ids(1)).expect("single pane");
        let result = split_pane(&tree, &pid("nonexistent"), SplitDirection::Horizontal, &pid("new"));
        assert!(result.is_none());
    }

    #[test]
    fn split_pane_vertical_direction() {
        let tree = super::tree_from_preset(LayoutPreset::OneByOne, &ids(1));
        let split = split_pane(&tree, &pid("p1"), SplitDirection::Vertical, &pid("p2"))
            .expect("vertical split should succeed");
        match &split {
            super::SplitNode::Split { direction, .. } => {
                assert_eq!(*direction, SplitDirection::Vertical);
            }
            _ => panic!("expected Split node"),
        }
        assert_eq!(collect_pane_ids(&split), vec![pid("p1"), pid("p2")]);
    }

    #[test]
    fn split_pane_nested_tree_finds_target_in_second_branch() {
        // Build a 2-pane tree first, then split the second pane
        let tree = tree_from_count(&ids(2)).expect("two panes");
        let extended = split_pane(&tree, &pid("p2"), SplitDirection::Horizontal, &pid("p3"))
            .expect("split p2 in nested tree");
        assert_eq!(collect_pane_ids(&extended).len(), 3);
    }

    // -------------------------------------------------------------------------
    // close_pane
    // -------------------------------------------------------------------------

    #[test]
    fn close_pane_last_pane_returns_none_tree() {
        let tree = super::tree_from_preset(LayoutPreset::OneByOne, &ids(1));
        let result = close_pane(&tree, &pid("p1")).expect("target found");
        assert!(result.is_none(), "closing last pane should return None tree");
    }

    #[test]
    fn close_pane_nonexistent_target_returns_none_outer() {
        let tree = tree_from_count(&ids(2)).expect("two panes");
        let result = close_pane(&tree, &pid("not-in-tree"));
        assert!(result.is_none(), "target not in tree should return None outer");
    }

    #[test]
    fn close_pane_first_in_two_pane_tree_leaves_second() {
        let tree = tree_from_count(&ids(2)).expect("two panes");
        let remaining = close_pane(&tree, &pid("p1"))
            .expect("target found")
            .expect("tree remains");
        assert_eq!(collect_pane_ids(&remaining), vec![pid("p2")]);
    }

    #[test]
    fn close_pane_second_in_two_pane_tree_leaves_first() {
        let tree = tree_from_count(&ids(2)).expect("two panes");
        let remaining = close_pane(&tree, &pid("p2"))
            .expect("target found")
            .expect("tree remains");
        assert_eq!(collect_pane_ids(&remaining), vec![pid("p1")]);
    }

    // -------------------------------------------------------------------------
    // swap_panes
    // -------------------------------------------------------------------------

    #[test]
    fn swap_panes_with_first_id_not_in_tree_returns_none() {
        let tree = tree_from_count(&ids(2)).expect("two panes");
        let result = swap_panes(&tree, &pid("ghost"), &pid("p2"));
        assert!(result.is_none());
    }

    #[test]
    fn swap_panes_with_second_id_not_in_tree_returns_none() {
        let tree = tree_from_count(&ids(2)).expect("two panes");
        let result = swap_panes(&tree, &pid("p1"), &pid("ghost"));
        assert!(result.is_none());
    }

    #[test]
    fn swap_panes_in_three_pane_tree() {
        let tree = tree_from_count(&ids(3)).expect("three panes");
        let swapped = swap_panes(&tree, &pid("p1"), &pid("p3")).expect("swap p1 and p3");
        let found = collect_pane_ids(&swapped);
        assert!(found.contains(&pid("p1")));
        assert!(found.contains(&pid("p3")));
        // After swap, p3 should be where p1 was (first position in in-order traversal)
        assert_eq!(found[0], pid("p3"));
    }

    // -------------------------------------------------------------------------
    // validate_layout
    // -------------------------------------------------------------------------

    #[test]
    fn validate_layout_passes_for_matching_ids() {
        let pane_ids = ids(2);
        let tree = tree_from_count(&pane_ids).expect("two panes");
        super::validate_layout(&tree, &pane_ids).expect("matching layout should validate");
    }

    #[test]
    fn validate_layout_fails_for_missing_pane_in_tree() {
        let pane_ids = ids(2);
        let tree = tree_from_count(&pane_ids).expect("two panes");
        // Add an extra pane ID not present in tree
        let mut extended = pane_ids.clone();
        extended.push(pid("p_extra"));
        let result = super::validate_layout(&tree, &extended);
        assert!(result.is_err(), "extra pane in slots but not tree should fail");
    }

    #[test]
    fn validate_layout_fails_for_orphan_pane_in_tree() {
        let tree = tree_from_count(&ids(2)).expect("two panes");
        // Only provide one of the two pane IDs in the slot list
        let only_one = vec![pid("p1")];
        let result = super::validate_layout(&tree, &only_one);
        assert!(result.is_err(), "pane in tree but not in slots should fail");
    }

    // -------------------------------------------------------------------------
    // remap_pane_ids
    // -------------------------------------------------------------------------

    #[test]
    fn remap_pane_ids_replaces_template_ids_with_real_ids() {
        use crate::ids::PaneId;
        let template = super::tree_from_preset(LayoutPreset::OneByTwo, &[
            PaneId::from(String::from("ph1")),
            PaneId::from(String::from("ph2")),
        ]);
        let real_ids = vec![
            PaneId::from(String::from("real1")),
            PaneId::from(String::from("real2")),
        ];
        let remapped = super::remap_pane_ids(&template, &real_ids);
        let found = collect_pane_ids(&remapped);
        assert_eq!(found, real_ids);
    }

    // -------------------------------------------------------------------------
    // collect_pane_ids
    // -------------------------------------------------------------------------

    #[test]
    fn collect_pane_ids_on_single_leaf() {
        use super::SplitNode;
        let leaf = SplitNode::Pane { pane_id: pid("solo") };
        assert_eq!(collect_pane_ids(&leaf), vec![pid("solo")]);
    }

    #[test]
    fn collect_pane_ids_returns_ids_in_depth_first_order() {
        let tree = tree_from_count(&ids(4)).expect("four panes");
        let found = collect_pane_ids(&tree);
        assert_eq!(found, ids(4), "should be depth-first left-to-right order");
    }

    // -------------------------------------------------------------------------
    // SplitNode: structural properties
    // -------------------------------------------------------------------------

    #[test]
    fn split_node_pane_equality() {
        use super::SplitNode;
        let a = SplitNode::Pane { pane_id: pid("x") };
        let b = SplitNode::Pane { pane_id: pid("x") };
        let c = SplitNode::Pane { pane_id: pid("y") };
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    #[test]
    fn split_node_clone_preserves_structure() {
        let tree = tree_from_count(&ids(3)).expect("three panes");
        let cloned = tree.clone();
        assert_eq!(tree, cloned);
    }

    #[test]
    fn split_direction_equality() {
        assert_eq!(SplitDirection::Horizontal, SplitDirection::Horizontal);
        assert_ne!(SplitDirection::Horizontal, SplitDirection::Vertical);
    }

    #[test]
    fn split_direction_copy() {
        let d = SplitDirection::Vertical;
        let d2 = d;
        assert_eq!(d, d2);
    }
}
