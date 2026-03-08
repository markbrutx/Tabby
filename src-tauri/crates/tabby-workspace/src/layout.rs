use std::collections::BTreeSet;

use thiserror::Error;

use crate::ids::PaneId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutPreset {
    OneByOne,
    OneByTwo,
    TwoByTwo,
    TwoByThree,
    ThreeByThree,
}

impl LayoutPreset {
    pub fn pane_count(self) -> usize {
        match self {
            Self::OneByOne => 1,
            Self::OneByTwo => 2,
            Self::TwoByTwo => 4,
            Self::TwoByThree => 6,
            Self::ThreeByThree => 9,
        }
    }

    pub fn parse(value: &str) -> Result<Self, LayoutError> {
        match value {
            "1x1" => Ok(Self::OneByOne),
            "1x2" => Ok(Self::OneByTwo),
            "2x2" => Ok(Self::TwoByTwo),
            "2x3" => Ok(Self::TwoByThree),
            "3x3" => Ok(Self::ThreeByThree),
            other => Err(LayoutError::UnsupportedPreset(String::from(other))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::OneByOne => "1x1",
            Self::OneByTwo => "1x2",
            Self::TwoByTwo => "2x2",
            Self::TwoByThree => "2x3",
            Self::ThreeByThree => "3x3",
        }
    }
}

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
}
