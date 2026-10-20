pub struct UiMetrics;

impl UiMetrics {
    // ── Package Card Dimensions ──────────────────────────────────────────────
    pub const CARD_WRAPPER_NORMAL: f32 = 88.0;
    pub const CARD_HEIGHT_NORMAL: f32 = 80.0;
    pub const CARD_WRAPPER_COMPACT: f32 = 70.0;
    pub const CARD_HEIGHT_COMPACT: f32 = 62.0;

    // ── Package Table Dimensions ─────────────────────────────────────────────
    pub const ROW_HEIGHT_HEADER: f32 = 32.0;
    pub const ROW_HEIGHT_NORMAL: f32 = 36.0;
    pub const ROW_HEIGHT_COMPACT: f32 = 30.0;

    // ── Workspace Splitter & Pane Limits ─────────────────────────────────────
    pub const SPLITTER_WIDTH: f32 = 5.0;
    pub const LIST_MIN_USABLE: f32 = 340.0;
    pub const INSPECTOR_MIN_USABLE: f32 = 320.0;
    pub const INSPECTOR_MAX_WIDTH: f32 = 520.0;
    pub const INSPECTOR_MAX_RATIO: f32 = 3.0 / 7.0;
    pub const HORIZONTAL_SPLIT_MIN_CONTENT_WIDTH: f32 = 320.0 * (7.0 / 3.0);

    // ── Workbench Dimensions ─────────────────────────────────────────────────
    pub const SEARCH_INPUT_HEIGHT: f32 = 40.0;
    pub const QUERY_CONTROLS_HEIGHT: f32 = 34.0;

    // ── Sidebar Geometry ─────────────────────────────────────────────────────
    pub const SIDEBAR_EXPANDED: f32 = 190.0;
    pub const SIDEBAR_COLLAPSED: f32 = 56.0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_card_wrapper_vs_content_height_invariants() {
        assert!(
            UiMetrics::CARD_WRAPPER_NORMAL > UiMetrics::CARD_HEIGHT_NORMAL,
            "Wrapper must accommodate padding around normal card content"
        );
        assert!(
            UiMetrics::CARD_WRAPPER_COMPACT > UiMetrics::CARD_HEIGHT_COMPACT,
            "Wrapper must accommodate padding around compact card content"
        );
        assert!(
            UiMetrics::CARD_HEIGHT_COMPACT < UiMetrics::CARD_HEIGHT_NORMAL,
            "Compact card height must be strictly smaller than normal card height"
        );
        assert!(
            UiMetrics::CARD_WRAPPER_COMPACT < UiMetrics::CARD_WRAPPER_NORMAL,
            "Compact card wrapper must be strictly smaller than normal card wrapper"
        );
    }

    #[test]
    fn test_table_row_height_invariants() {
        assert!(
            UiMetrics::ROW_HEIGHT_COMPACT < UiMetrics::ROW_HEIGHT_NORMAL,
            "Compact table row must be smaller than normal table row"
        );
        assert!(
            UiMetrics::ROW_HEIGHT_HEADER > 0.0,
            "Header height must be positive"
        );
    }

    #[test]
    fn test_splitter_and_sidebar_geometry_invariants() {
        assert!(
            UiMetrics::SIDEBAR_COLLAPSED < UiMetrics::SIDEBAR_EXPANDED,
            "Collapsed sidebar must be narrower than expanded sidebar"
        );
        assert!(
            UiMetrics::LIST_MIN_USABLE > 0.0,
            "List minimum usable width must be positive"
        );
        assert!(
            UiMetrics::INSPECTOR_MIN_USABLE > 0.0,
            "Inspector minimum usable width must be positive"
        );
        assert!(
            UiMetrics::SPLITTER_WIDTH > 0.0,
            "Splitter width must be positive"
        );
    }

    #[test]
    fn test_inspector_ratio_constants() {
        assert!(UiMetrics::INSPECTOR_MAX_RATIO > 0.42 && UiMetrics::INSPECTOR_MAX_RATIO < 0.43);
        assert_eq!(UiMetrics::INSPECTOR_MAX_WIDTH, 520.0);
        assert_eq!(UiMetrics::INSPECTOR_MIN_USABLE, 320.0);
        assert!(UiMetrics::HORIZONTAL_SPLIT_MIN_CONTENT_WIDTH > 746.0);
    }
}
