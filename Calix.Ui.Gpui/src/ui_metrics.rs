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
    pub const SPLITTER_THICKNESS: f32 = 8.0;
    /// Le plancher de la plaque : l'identité seule.
    ///
    /// La plaque cède d'abord — elle rend sa place avant que le catalogue
    /// ne soit réduit. Son corps se réduit d'abord, puis ses actions, puis
    /// ses onglets ; l'identité reste, parce qu'elle dit *quel* paquet est
    /// sélectionné, et que sans elle la plaque ne dit plus rien.
    pub const NAMEPLATE_MIN_HEIGHT: f32 = 48.0;
    pub const NAMEPLATE_MAX_RATIO: f32 = 0.55;
    /// What the plate asks for under the catalogue until the user drags it:
    /// enough for its name, its actions, its tabs and a first paragraph. It
    /// asked for its floor, and a plate at its floor says only a name.
    pub const NAMEPLATE_REQUESTED_HEIGHT: f32 = 300.0;

    // ── The inspector beside the catalogue ──────────────────────────────────
    //
    // Where the window has the room for the catalogue's floor and a useful
    // inspector side by side, the inspector stands to the right; otherwise
    // it lies under the catalogue (the plate).
    /// What the side inspector asks for until the user drags its edge.
    pub const INSPECTOR_SIDE_WIDTH: f32 = 400.0;
    /// Below this, a side inspector wraps its facts word by word: it goes
    /// under the catalogue instead.
    pub const INSPECTOR_SIDE_MIN_WIDTH: f32 = 340.0;
    /// The share of the band the side inspector may take.
    pub const INSPECTOR_SIDE_MAX_RATIO: f32 = 0.5;
    /// The aluminium between two planes.
    pub const PLANE_GAP: f32 = 8.0;

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
    pub const CONTEXT_HEADER_HEIGHT: f32 = 84.0;
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
    /// It is **measured**, not chosen. A row of the table spends 399 px
    /// before the name (the four fixed columns — version, source, size,
    /// status — and the row's padding), and the name needs its glyph, its
    /// gap, and — for 90 % of the real packages of this machine — 143 px of
    /// text. Without the ladder, the name would be whole from 564 px:
    /// 399 + 22 + 143. With the ladder (§4.7), the name keeps its floor of
    /// 165 px and the columns that say *where* give way, so the operator's
    /// target of 560 px holds: the p90's name is whole, and the two
    /// secondary cells give up 4 px of slack between them.
    ///
    /// The proof is `the_row_is_measured_at_and_around_the_catalogue_minimum`
    /// (components/package_table.rs). The 660 px and the 495 px of the
    /// earlier qualifications are kept in the record: they are what the row
    /// cost when the instrument had a column of its own, and §5.7 has ruled
    /// it into the size column — the record is the audit's, §5 item 7.
    pub const CATALOGUE_MIN_WIDTH: f32 = 560.0;

    // ── The row's own ladder ────────────────────────────────────────────────
    //
    // The row's fixed columns do not all have the same importance. The name
    // is the package's key; the version and the source say where it comes
    // from; the size and the status are the figure and the state, and the
    // size column carries the instrument's reading on top of its figure.
    // When the row is too narrow, the ones that say *where* give way first,
    // and the name is the last to be cut — and the row is clipped
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
    /// Le flexbox répartit le manque au prorata des facteurs de
    /// rétrécissement : ces deux nombres sont donc l'ordre lui-même.
    ///
    /// La source d'abord — c'est la plus jetable : le nom la dit presque
    /// toujours, et l'inspecteur la donne en entier. La version en dernier
    /// des secondaires. La lecture de l'instrument n'est plus une marche de
    /// l'échelle : depuis la décision §5.7 elle vit dans la colonne Taille,
    /// qui ne rétrécit jamais, et ne coûte plus un pixel à la ligne.
    ///
    /// La taille et l'état ne rétrécissent jamais : l'une est la figure,
    /// l'autre l'état du paquet. Et le nom ne rétrécit jamais non plus : c'est
    /// la clé.
    pub const ROW_SHRINK_SOURCE: f32 = 4.0;
    pub const ROW_SHRINK_VERSION: f32 = 1.0;

    /// The bar's track, inside the size column (§5.7).
    ///
    /// The size column never shrinks: it is 80 px wide and keeps a right
    /// padding of 12 px, so the track is 68 px — and because the column
    /// never gives way, the track is never narrowed by the ladder: the
    /// fill is `track × ratio` at every width of the row. The bar's length
    /// therefore *encodes* the size wherever the row is drawn, and the
    /// clip zone of the former column (the visible length
    /// `96 × ratio − 33` of the old 96 px cell) is gone with it.
    pub const SIZE_BAR_TRACK: f32 = 68.0;

    // ── The vertical budget, inside the station ─────────────────────────────
    //
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
