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

    // ── The plate: the nameplate's bounds ───────────────────────────────────
    //
    // The plate is read from the top down in full-width bands. The nameplate
    // is the caption of the readout: it never swallows it, and it never
    // disappears.
    pub const SPLITTER_THICKNESS: f32 = 5.0;
    pub const NAMEPLATE_MIN_HEIGHT: f32 = 168.0;
    pub const NAMEPLATE_MAX_RATIO: f32 = 0.55;

    // ── Workbench Dimensions ─────────────────────────────────────────────────
    pub const SEARCH_INPUT_HEIGHT: f32 = 40.0;
    pub const QUERY_CONTROLS_HEIGHT: f32 = 34.0;

    // ── Sidebar Geometry ─────────────────────────────────────────────────────
    pub const SIDEBAR_EXPANDED: f32 = 190.0;
    pub const SIDEBAR_COLLAPSED: f32 = 56.0;
    /// La hauteur d'une entrée du rail. Égale pour toutes : le trait de la
    /// sélection voyage d'une rangée à l'autre, et sa position se calcule au
    /// lieu de se mesurer.
    pub const RAIL_ROW_HEIGHT: f32 = 38.0;
    /// L'écart entre deux entrées du rail.
    pub const RAIL_ROW_GAP: f32 = 2.0;
    /// Le vide au-dessus de la première entrée du rail.
    pub const RAIL_TOP_PAD: f32 = 8.0;
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
            UiMetrics::NAMEPLATE_MIN_HEIGHT > 0.0,
            "The nameplate's minimum height must be positive"
        );
        assert!(
            UiMetrics::NAMEPLATE_MAX_RATIO > 0.0 && UiMetrics::NAMEPLATE_MAX_RATIO < 1.0,
            "The nameplate's maximum share must be a share"
        );
        assert!(
            UiMetrics::SPLITTER_THICKNESS > 0.0,
            "The splitter's thickness must be positive"
        );
    }

    #[test]
    fn test_nameplate_ratio_constants() {
        assert!(UiMetrics::NAMEPLATE_MAX_RATIO > 0.5 && UiMetrics::NAMEPLATE_MAX_RATIO < 0.56);
    }
}
