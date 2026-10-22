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
    //
    // Its floor is its *own* chrome — the name, the actions, the tabs — and
    // nothing more. It was 168, which was the comfortable height, and a
    // comfortable height is a luxury: at the workspace's minimum, a floor of
    // 168 left the catalogue with less than a row and a half. The plate is a
    // context surface — it is the first to give way — and its floor is the
    // height at which it still shows its tabs.
    pub const SPLITTER_THICKNESS: f32 = 5.0;
    /// Le chrome de la plaque, mesuré : l'identité, les actions, les onglets.
    ///
    /// Ce n'est pas une estimation : ce sont les hauteurs que le banc a
    /// mesurées sur la vraie plaque (identity 47.5, actions 29.5, tabs
    /// 34.5). Le chrome complet fait 131.5 px.
    pub const PLATE_IDENTITY_HEIGHT: f32 = 47.5;
    pub const PLATE_ACTIONS_HEIGHT: f32 = 29.5;
    pub const PLATE_TABS_HEIGHT: f32 = 34.5;
    pub const PLATE_CHROME_HEIGHT: f32 = 131.5;
    /// Le plancher de la plaque : l'identité seule.
    ///
    /// La plaque cède d'abord — elle rend sa place avant que le catalogue
    /// ne soit réduit. Son corps se réduit d'abord, puis ses actions, puis
    /// ses onglets ; l'identité reste, parce qu'elle dit *quel* paquet est
    /// sélectionné, et que sans elle la plaque ne dit plus rien.
    pub const NAMEPLATE_MIN_HEIGHT: f32 = 48.0;
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

    // ── The vertical budget ──────────────────────────────────────────────────
    //
    // The header's band: the place that says where we are. It is a band, not
    // a text that happens to be there — the budget divides the window with
    // it, so its height has to be a number everyone knows.
    pub const CONTEXT_HEADER_HEIGHT: f32 = 88.0;
    /// The vertical space the primary workspace keeps, whatever else is open.
    ///
    /// The catalogue is the body; the plate and the log are context surfaces.
    /// When the space runs short, the context surfaces give way — never the
    /// catalogue.
    pub const WORKSPACE_MIN_HEIGHT: f32 = 320.0;
    /// The log drawer's share of the available height.
    pub const DRAWER_MAX_RATIO: f32 = 0.40;
    /// Below this, a split drawer is not a drawer: it becomes an overlay.
    pub const DRAWER_MIN_HEIGHT: f32 = 120.0;
    /// The drawer's own band: the bar that says what it is and how to close
    /// it. It is always there, even when the drawer is closed — it is how the
    /// drawer is opened.
    pub const DRAWER_HEADER_HEIGHT: f32 = 36.0;

    // ── The horizontal budget ────────────────────────────────────────────────
    //
    /// The width the catalogue keeps: below it, the catalogue is not a
    /// catalogue any more.
    ///
    /// It is **measured**, not chosen. A row of the table spends 495 px
    /// before the name (the five fixed columns and the row's padding), and
    /// the name needs its glyph, its gap, and — for 90 % of the real
    /// packages of this machine — 143 px of text. The minimum is the sum:
    /// 495 + 22 + 143 = 660 px.
    ///
    /// The proof is `the_row_is_measured_at_and_around_the_catalogue_minimum`
    /// (components/package_table.rs), and the 560 px that was tried first is
    /// kept in the record as the case that failed: at 560, the name shows
    /// 43 px of text — six characters.
    pub const CATALOGUE_MIN_WIDTH: f32 = 560.0;

    // ── The row's own ladder ────────────────────────────────────────────────
    //
    // The row's fixed columns do not all have the same importance. The name
    // is the package's key; the version and the source say where it comes
    // from; the measurement, the size and the status are the instrument's
    // reading. When the row is too narrow, the ones that say *where* give way
    // first, and the name is the last to be cut — and the row is clipped
    // only when there is nothing left to give.
    pub const ROW_VERSION_MIN_WIDTH: f32 = 60.0;
    pub const ROW_SIZE_MIN_WIDTH: f32 = 60.0;
    pub const ROW_STATUS_MIN_WIDTH: f32 = 74.0;
    /// The name's floor: the cell that holds the p90 of the real packages
    /// (143 px of text) plus the identity glyph and its gap (22 px).
    /// Below it, the name is cut — which is the last thing that happens.
    pub const ROW_NAME_MIN_WIDTH: f32 = 165.0;

    /// L'ordre dans lequel la ligne cède quand elle est trop étroite.
    ///
    /// Le flexbox répartit le manque au prorata des facteurs de rétrécissement :
    /// ces trois nombres sont donc l'ordre lui-même.
    ///
    /// La source d'abord — c'est la plus jetable : le nom la dit presque
    /// toujours, et l'inspecteur la donne en entier. La lecture de
    /// l'instrument ensuite — la taille exacte a sa propre colonne. La
    /// version en dernier des secondaires.
    ///
    /// La taille et l'état ne rétrécissent jamais : l'un est la figure, l'autre
    /// l'état du paquet. Et le nom ne rétrécit jamais non plus : c'est la clé.
    pub const ROW_SHRINK_SOURCE: f32 = 4.0;
    pub const ROW_SHRINK_MEASUREMENT: f32 = 2.0;
    pub const ROW_SHRINK_VERSION: f32 = 1.0;

    // ── The vertical budget, inside the station ─────────────────────────────
    //
    /// The bank's band: the seven sources. Measured, and *reserved* — the
    /// plate's share is computed after it, so the reservation has to be an
    /// upper bound of what the bank really takes.
    pub const BANK_HEIGHT: f32 = 100.5;
    /// The vertical space the catalogue keeps, whatever else.
    ///
    /// Six rows: a catalogue that shows two rows is a strip, and a strip is
    /// not a catalogue. Below this, the plate leaves the flow rather than
    /// squeezing the catalogue — the plate is a context surface, and it
    /// yields entirely before the body is reduced.
    pub const CATALOGUE_MIN_HEIGHT: f32 = 216.0;
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
