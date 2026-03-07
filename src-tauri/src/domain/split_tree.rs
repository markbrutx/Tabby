use crate::domain::layout::{LayoutPreset, SplitDirection, SplitNode};

pub fn tree_from_preset(preset: LayoutPreset, pane_ids: &[String]) -> SplitNode {
    match preset {
        LayoutPreset::OneByOne => SplitNode::Pane {
            pane_id: pane_ids[0].clone(),
        },
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

pub fn split_pane(
    root: &SplitNode,
    target_pane_id: &str,
    direction: SplitDirection,
    new_pane_id: &str,
) -> Option<SplitNode> {
    match root {
        SplitNode::Pane { pane_id } if pane_id == target_pane_id => Some(SplitNode::Split {
            direction,
            ratio: 500,
            first: Box::new(SplitNode::Pane {
                pane_id: pane_id.clone(),
            }),
            second: Box::new(SplitNode::Pane {
                pane_id: String::from(new_pane_id),
            }),
        }),
        SplitNode::Pane { .. } => None,
        SplitNode::Split {
            direction: dir,
            ratio,
            first,
            second,
        } => {
            if let Some(new_first) = split_pane(first, target_pane_id, direction, new_pane_id) {
                return Some(SplitNode::Split {
                    direction: *dir,
                    ratio: *ratio,
                    first: Box::new(new_first),
                    second: second.clone(),
                });
            }

            if let Some(new_second) = split_pane(second, target_pane_id, direction, new_pane_id) {
                return Some(SplitNode::Split {
                    direction: *dir,
                    ratio: *ratio,
                    first: first.clone(),
                    second: Box::new(new_second),
                });
            }

            None
        }
    }
}

/// Returns `Some(Some(node))` if the pane was removed and the tree still has nodes,
/// `Some(None)` if the pane was the last node (tree is now empty),
/// `None` if the pane was not found.
pub fn close_pane(root: &SplitNode, target_pane_id: &str) -> Option<Option<SplitNode>> {
    match root {
        SplitNode::Pane { pane_id } if pane_id == target_pane_id => Some(None),
        SplitNode::Pane { .. } => None,
        SplitNode::Split {
            direction,
            ratio,
            first,
            second,
        } => {
            if let Some(result) = close_pane(first, target_pane_id) {
                return match result {
                    None => Some(Some(second.as_ref().clone())),
                    Some(new_first) => Some(Some(SplitNode::Split {
                        direction: *direction,
                        ratio: *ratio,
                        first: Box::new(new_first),
                        second: second.clone(),
                    })),
                };
            }

            if let Some(result) = close_pane(second, target_pane_id) {
                return match result {
                    None => Some(Some(first.as_ref().clone())),
                    Some(new_second) => Some(Some(SplitNode::Split {
                        direction: *direction,
                        ratio: *ratio,
                        first: first.clone(),
                        second: Box::new(new_second),
                    })),
                };
            }

            None
        }
    }
}

pub fn collect_pane_ids(root: &SplitNode) -> Vec<String> {
    let mut result = Vec::new();
    collect_pane_ids_inner(root, &mut result);
    result
}

pub fn swap_panes(root: &SplitNode, pane_id_a: &str, pane_id_b: &str) -> Option<SplitNode> {
    let ids = collect_pane_ids(root);
    if !ids.iter().any(|id| id == pane_id_a) || !ids.iter().any(|id| id == pane_id_b) {
        return None;
    }
    Some(swap_panes_inner(root, pane_id_a, pane_id_b))
}

fn swap_panes_inner(node: &SplitNode, pane_id_a: &str, pane_id_b: &str) -> SplitNode {
    match node {
        SplitNode::Pane { pane_id } => {
            if pane_id == pane_id_a {
                SplitNode::Pane {
                    pane_id: String::from(pane_id_b),
                }
            } else if pane_id == pane_id_b {
                SplitNode::Pane {
                    pane_id: String::from(pane_id_a),
                }
            } else {
                node.clone()
            }
        }
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

fn collect_pane_ids_inner(node: &SplitNode, result: &mut Vec<String>) {
    match node {
        SplitNode::Pane { pane_id } => result.push(pane_id.clone()),
        SplitNode::Split { first, second, .. } => {
            collect_pane_ids_inner(first, result);
            collect_pane_ids_inner(second, result);
        }
    }
}

fn leaf(id: &str) -> SplitNode {
    SplitNode::Pane {
        pane_id: String::from(id),
    }
}

fn hsplit(a: &str, b: &str) -> SplitNode {
    SplitNode::Split {
        direction: SplitDirection::Horizontal,
        ratio: 500,
        first: Box::new(leaf(a)),
        second: Box::new(leaf(b)),
    }
}

fn hsplit3(a: &str, b: &str, c: &str) -> SplitNode {
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

fn hsplit4(a: &str, b: &str, c: &str, d: &str) -> SplitNode {
    SplitNode::Split {
        direction: SplitDirection::Horizontal,
        ratio: 500,
        first: Box::new(hsplit(a, b)),
        second: Box::new(hsplit(c, d)),
    }
}

pub fn tree_from_count(pane_ids: &[String]) -> SplitNode {
    match pane_ids.len() {
        0 => panic!("tree_from_count requires at least 1 pane"),
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
        n => panic!("tree_from_count supports 1–9 panes, got {n}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(n: usize) -> Vec<String> {
        (1..=n).map(|i| format!("p{i}")).collect()
    }

    #[test]
    fn tree_from_1x1() {
        let tree = tree_from_preset(LayoutPreset::OneByOne, &ids(1));
        assert_eq!(collect_pane_ids(&tree), vec!["p1"]);
    }

    #[test]
    fn tree_from_2x2() {
        let tree = tree_from_preset(LayoutPreset::TwoByTwo, &ids(4));
        assert_eq!(collect_pane_ids(&tree), vec!["p1", "p2", "p3", "p4"]);
    }

    #[test]
    fn tree_from_3x3() {
        let tree = tree_from_preset(LayoutPreset::ThreeByThree, &ids(9));
        let panes = collect_pane_ids(&tree);
        assert_eq!(panes.len(), 9);
        assert_eq!(panes[0], "p1");
        assert_eq!(panes[8], "p9");
    }

    #[test]
    fn split_pane_replaces_leaf_with_branch() {
        let tree = tree_from_preset(LayoutPreset::OneByOne, &ids(1));
        let result = split_pane(&tree, "p1", SplitDirection::Horizontal, "p2");
        let new_tree = result.expect("split should succeed");
        assert_eq!(collect_pane_ids(&new_tree), vec!["p1", "p2"]);
    }

    #[test]
    fn split_pane_returns_none_for_unknown_pane() {
        let tree = tree_from_preset(LayoutPreset::OneByOne, &ids(1));
        assert!(split_pane(&tree, "unknown", SplitDirection::Horizontal, "p2").is_none());
    }

    #[test]
    fn close_pane_removes_leaf_and_collapses_parent() {
        let tree = tree_from_preset(LayoutPreset::OneByTwo, &ids(2));
        let result = close_pane(&tree, "p1");
        let remaining = result
            .expect("close should succeed")
            .expect("tree should remain");
        assert_eq!(collect_pane_ids(&remaining), vec!["p2"]);
    }

    #[test]
    fn close_last_pane_returns_none() {
        let tree = tree_from_preset(LayoutPreset::OneByOne, &ids(1));
        let result = close_pane(&tree, "p1");
        assert!(result.expect("close should succeed").is_none());
    }

    #[test]
    fn close_pane_returns_none_for_unknown_pane() {
        let tree = tree_from_preset(LayoutPreset::OneByOne, &ids(1));
        assert!(close_pane(&tree, "unknown").is_none());
    }

    #[test]
    fn split_then_close_roundtrip() {
        let tree = tree_from_preset(LayoutPreset::OneByOne, &ids(1));
        let after_split =
            split_pane(&tree, "p1", SplitDirection::Vertical, "p2").expect("split ok");
        assert_eq!(collect_pane_ids(&after_split), vec!["p1", "p2"]);

        let after_close = close_pane(&after_split, "p2")
            .expect("close ok")
            .expect("tree remains");
        assert_eq!(collect_pane_ids(&after_close), vec!["p1"]);
    }

    #[test]
    fn tree_from_count_1() {
        let tree = tree_from_count(&ids(1));
        assert_eq!(collect_pane_ids(&tree), vec!["p1"]);
    }

    #[test]
    fn tree_from_count_2() {
        let tree = tree_from_count(&ids(2));
        assert_eq!(collect_pane_ids(&tree), vec!["p1", "p2"]);
    }

    #[test]
    fn tree_from_count_3() {
        let tree = tree_from_count(&ids(3));
        assert_eq!(collect_pane_ids(&tree), vec!["p1", "p2", "p3"]);
    }

    #[test]
    fn tree_from_count_4() {
        let tree = tree_from_count(&ids(4));
        assert_eq!(collect_pane_ids(&tree), vec!["p1", "p2", "p3", "p4"]);
    }

    #[test]
    fn tree_from_count_5() {
        let tree = tree_from_count(&ids(5));
        let panes = collect_pane_ids(&tree);
        assert_eq!(panes.len(), 5);
        assert_eq!(panes, vec!["p1", "p2", "p3", "p4", "p5"]);
    }

    #[test]
    fn tree_from_count_6() {
        let tree = tree_from_count(&ids(6));
        let panes = collect_pane_ids(&tree);
        assert_eq!(panes.len(), 6);
    }

    #[test]
    fn tree_from_count_7() {
        let tree = tree_from_count(&ids(7));
        let panes = collect_pane_ids(&tree);
        assert_eq!(panes.len(), 7);
    }

    #[test]
    fn tree_from_count_8() {
        let tree = tree_from_count(&ids(8));
        let panes = collect_pane_ids(&tree);
        assert_eq!(panes.len(), 8);
    }

    #[test]
    fn tree_from_count_9() {
        let tree = tree_from_count(&ids(9));
        let panes = collect_pane_ids(&tree);
        assert_eq!(panes.len(), 9);
        assert_eq!(panes[0], "p1");
        assert_eq!(panes[8], "p9");
    }

    #[test]
    #[should_panic(expected = "at least 1 pane")]
    fn tree_from_count_0_panics() {
        tree_from_count(&ids(0));
    }

    #[test]
    #[should_panic(expected = "1–9 panes")]
    fn tree_from_count_10_panics() {
        tree_from_count(&ids(10));
    }

    #[test]
    fn swap_panes_in_horizontal_split() {
        let tree = tree_from_preset(LayoutPreset::OneByTwo, &ids(2));
        let swapped = swap_panes(&tree, "p1", "p2").expect("swap should succeed");
        assert_eq!(collect_pane_ids(&swapped), vec!["p2", "p1"]);
    }

    #[test]
    fn swap_panes_returns_none_for_unknown_pane() {
        let tree = tree_from_preset(LayoutPreset::OneByTwo, &ids(2));
        assert!(swap_panes(&tree, "p1", "unknown").is_none());
    }

    #[test]
    fn swap_panes_self_swap_returns_same_tree() {
        let tree = tree_from_preset(LayoutPreset::OneByTwo, &ids(2));
        let swapped = swap_panes(&tree, "p1", "p1").expect("self-swap should succeed");
        assert_eq!(collect_pane_ids(&swapped), vec!["p1", "p2"]);
    }

    #[test]
    fn swap_panes_deep_tree() {
        let tree = tree_from_preset(LayoutPreset::TwoByTwo, &ids(4));
        let swapped = swap_panes(&tree, "p1", "p4").expect("swap should succeed");
        let result_ids = collect_pane_ids(&swapped);
        assert_eq!(result_ids, vec!["p4", "p2", "p3", "p1"]);
    }

    #[test]
    fn deep_split_and_close() {
        let tree = tree_from_preset(LayoutPreset::OneByOne, &ids(1));
        let t2 = split_pane(&tree, "p1", SplitDirection::Horizontal, "p2").unwrap();
        let t3 = split_pane(&t2, "p2", SplitDirection::Vertical, "p3").unwrap();
        assert_eq!(collect_pane_ids(&t3), vec!["p1", "p2", "p3"]);

        let t4 = close_pane(&t3, "p2").unwrap().unwrap();
        assert_eq!(collect_pane_ids(&t4), vec!["p1", "p3"]);
    }
}
