use crate::workspace::domain::layout::{LayoutPreset, SplitDirection, SplitNode};

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
        n => panic!("tree_from_count supports 1\u{2013}9 panes, got {n}"),
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
