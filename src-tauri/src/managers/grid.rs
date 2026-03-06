use crate::domain::types::{GridDefinition, LayoutPreset};

#[derive(Debug, Default)]
pub struct GridManager;

impl GridManager {
    pub fn new() -> Self {
        Self
    }

    pub fn definition(&self, preset: LayoutPreset) -> GridDefinition {
        let (rows, columns) = preset.dimensions();

        GridDefinition {
            preset,
            rows,
            columns,
            pane_count: preset.pane_count(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GridManager;
    use crate::domain::types::LayoutPreset;

    #[test]
    fn maps_two_by_three_to_six_panes() {
        let manager = GridManager::new();
        let definition = manager.definition(LayoutPreset::TwoByThree);

        assert_eq!(definition.rows, 2);
        assert_eq!(definition.columns, 3);
        assert_eq!(definition.pane_count, 6);
    }

    #[test]
    fn maps_three_by_three_to_nine_panes() {
        let manager = GridManager::new();

        assert_eq!(manager.definition(LayoutPreset::ThreeByThree).pane_count, 9);
    }
}
