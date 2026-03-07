use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, Type)]
pub enum LayoutPreset {
    #[serde(rename = "1x1")]
    #[default]
    OneByOne,
    #[serde(rename = "1x2")]
    OneByTwo,
    #[serde(rename = "2x2")]
    TwoByTwo,
    #[serde(rename = "2x3")]
    TwoByThree,
    #[serde(rename = "3x3")]
    ThreeByThree,
}

impl LayoutPreset {
    pub fn pane_count(self) -> u16 {
        match self {
            Self::OneByOne => 1,
            Self::OneByTwo => 2,
            Self::TwoByTwo => 4,
            Self::TwoByThree => 6,
            Self::ThreeByThree => 9,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(rename_all = "camelCase")]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Type)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum SplitNode {
    Pane {
        #[serde(rename = "paneId")]
        pane_id: String,
    },
    Split {
        direction: SplitDirection,
        ratio: u16,
        first: Box<SplitNode>,
        second: Box<SplitNode>,
    },
}
