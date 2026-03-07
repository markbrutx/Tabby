use crate::domain::types::{LayoutPreset, SplitDirection, SplitNode};

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
        SplitNode::Pane { pane_id } if pane_id == target_pane_id => {
            Some(SplitNode::Split {
                direction,
                ratio: 500,
                first: Box::new(SplitNode::Pane {
                    pane_id: pane_id.clone(),
                }),
                second: Box::new(SplitNode::Pane {
                    pane_id: String::from(new_pane_id),
                }),
            })
        }
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

#[cfg(test)]
fn collect_pane_ids(root: &SplitNode) -> Vec<String> {
    let mut result = Vec::new();
    collect_pane_ids_inner(root, &mut result);
    result
}

#[cfg(test)]
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
        let remaining = result.expect("close should succeed").expect("tree should remain");
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
    fn deep_split_and_close() {
        let tree = tree_from_preset(LayoutPreset::OneByOne, &ids(1));
        let t2 = split_pane(&tree, "p1", SplitDirection::Horizontal, "p2").unwrap();
        let t3 = split_pane(&t2, "p2", SplitDirection::Vertical, "p3").unwrap();
        assert_eq!(collect_pane_ids(&t3), vec!["p1", "p2", "p3"]);

        let t4 = close_pane(&t3, "p2").unwrap().unwrap();
        assert_eq!(collect_pane_ids(&t4), vec!["p1", "p3"]);
    }
}
