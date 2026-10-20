use crate::render_lab::texture::{self, TextureKind};
use gpui::*;
use serde::{Deserialize, Serialize};

/// Schema version of the canonical Recipe Graph IR.
pub const RECIPE_GRAPH_SCHEMA_VERSION: u32 = 1;

// ── Color and Geometric Specifications (Engine-Independent) ───────────

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ColorSpec {
    Rgb { r: u8, g: u8, b: u8 },
    Rgba { r: u8, g: u8, b: u8, a: u8 },
    Hsla { h: f32, s: f32, l: f32, a: f32 },
}

impl ColorSpec {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::Rgb { r, g, b }
    }

    pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self::Rgba { r, g, b, a }
    }

    pub const fn hsla(h: f32, s: f32, l: f32, a: f32) -> Self {
        Self::Hsla { h, s, l, a }
    }

    pub fn to_gpui_hsla(self) -> gpui::Hsla {
        match self {
            Self::Rgb { r, g, b } => {
                let u = (r as u32) << 16 | (g as u32) << 8 | (b as u32);
                gpui::rgb(u).into()
            }
            Self::Rgba { r, g, b, a } => {
                let u = (r as u32) << 24 | (g as u32) << 16 | (b as u32) << 8 | (a as u32);
                gpui::rgba(u).into()
            }
            Self::Hsla { h, s, l, a } => gpui::hsla(h, s, l, a),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Dim {
    Full,
    Px(f32),
    Flex(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LayoutMode {
    None,
    Center,
    FlexRow,
    FlexCol,
    JustifyBetweenCol,
    AbsoluteFill,
    Relative,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BorderRadius {
    None,
    Md,
    Lg,
    Xl,
    Full,
    Px(f32),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BorderEdgeSpec {
    pub width: f32,
    pub color: ColorSpec,
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct BordersSpec {
    pub uniform: Option<BorderEdgeSpec>,
    pub top: Option<BorderEdgeSpec>,
    pub bottom: Option<BorderEdgeSpec>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ShadowSpec {
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur_radius: f32,
    pub spread_radius: f32,
    pub color: ColorSpec,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum FillSpec {
    Solid {
        color: ColorSpec,
    },
    LinearGradient2 {
        angle_deg: f32,
        start: ColorSpec,
        end: ColorSpec,
    },
    Texture {
        kind: TextureKind,
        seed: u64,
        width: u32,
        height: u32,
    },
}

// ── Graph Nodes ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum RecipeNode {
    Rect {
        w: Dim,
        h: Dim,
        layout: LayoutMode,
        padding: f32,
        fill: Option<FillSpec>,
        borders: BordersSpec,
        corner_radius: BorderRadius,
        shadows: Vec<ShadowSpec>,
        children: Vec<RecipeNode>,
    },
    Streamlines {
        lines: usize,
        stroke_width: f32,
        color: ColorSpec,
    },
    DirectionArrows {
        count: usize,
        stroke_width: f32,
        color: ColorSpec,
        shaft_length: f32,
        head_length: f32,
        head_width: f32,
    },
}

impl RecipeNode {
    pub fn rect() -> RectBuilder {
        RectBuilder::default()
    }

    pub fn streamlines(lines: usize, stroke_width: f32, color: ColorSpec) -> Self {
        Self::Streamlines {
            lines,
            stroke_width,
            color,
        }
    }

    pub fn direction_arrows(
        count: usize,
        stroke_width: f32,
        color: ColorSpec,
        shaft_length: f32,
        head_length: f32,
        head_width: f32,
    ) -> Self {
        Self::DirectionArrows {
            count,
            stroke_width,
            color,
            shaft_length,
            head_length,
            head_width,
        }
    }
}

#[derive(Default)]
pub struct RectBuilder {
    w: Option<Dim>,
    h: Option<Dim>,
    layout: Option<LayoutMode>,
    padding: f32,
    fill: Option<FillSpec>,
    borders: BordersSpec,
    corner_radius: Option<BorderRadius>,
    shadows: Vec<ShadowSpec>,
    children: Vec<RecipeNode>,
}

impl RectBuilder {
    pub fn w(mut self, dim: Dim) -> Self {
        self.w = Some(dim);
        self
    }

    pub fn h(mut self, dim: Dim) -> Self {
        self.h = Some(dim);
        self
    }

    pub fn size(mut self, dim: Dim) -> Self {
        self.w = Some(dim);
        self.h = Some(dim);
        self
    }

    pub fn size_full(self) -> Self {
        self.size(Dim::Full)
    }

    pub fn layout(mut self, mode: LayoutMode) -> Self {
        self.layout = Some(mode);
        self
    }

    pub fn center(self) -> Self {
        self.layout(LayoutMode::Center)
    }

    pub fn flex_row(self) -> Self {
        self.layout(LayoutMode::FlexRow)
    }

    pub fn flex_col(self) -> Self {
        self.layout(LayoutMode::FlexCol)
    }

    pub fn justify_between_col(self) -> Self {
        self.layout(LayoutMode::JustifyBetweenCol)
    }

    pub fn absolute_fill(self) -> Self {
        self.layout(LayoutMode::AbsoluteFill)
    }

    pub fn relative(self) -> Self {
        self.layout(LayoutMode::Relative)
    }

    pub fn padding(mut self, px: f32) -> Self {
        self.padding = px;
        self
    }

    pub fn fill(mut self, fill: FillSpec) -> Self {
        self.fill = Some(fill);
        self
    }

    pub fn solid(self, color: ColorSpec) -> Self {
        self.fill(FillSpec::Solid { color })
    }

    pub fn linear_gradient(self, angle_deg: f32, start: ColorSpec, end: ColorSpec) -> Self {
        self.fill(FillSpec::LinearGradient2 {
            angle_deg,
            start,
            end,
        })
    }

    pub fn texture(self, kind: TextureKind, seed: u64, width: u32, height: u32) -> Self {
        self.fill(FillSpec::Texture {
            kind,
            seed,
            width,
            height,
        })
    }

    pub fn border_uniform(mut self, width: f32, color: ColorSpec) -> Self {
        self.borders.uniform = Some(BorderEdgeSpec { width, color });
        self
    }

    pub fn border_t(mut self, width: f32, color: ColorSpec) -> Self {
        self.borders.top = Some(BorderEdgeSpec { width, color });
        self
    }

    pub fn border_b(mut self, width: f32, color: ColorSpec) -> Self {
        self.borders.bottom = Some(BorderEdgeSpec { width, color });
        self
    }

    pub fn rounded(mut self, radius: BorderRadius) -> Self {
        self.corner_radius = Some(radius);
        self
    }

    pub fn shadow(
        mut self,
        offset_x: f32,
        offset_y: f32,
        blur_radius: f32,
        spread_radius: f32,
        color: ColorSpec,
    ) -> Self {
        self.shadows.push(ShadowSpec {
            offset_x,
            offset_y,
            blur_radius,
            spread_radius,
            color,
        });
        self
    }

    pub fn child(mut self, child: RecipeNode) -> Self {
        self.children.push(child);
        self
    }

    pub fn children<I: IntoIterator<Item = RecipeNode>>(mut self, children: I) -> Self {
        self.children.extend(children);
        self
    }

    pub fn build(self) -> RecipeNode {
        RecipeNode::Rect {
            w: self.w.unwrap_or(Dim::Full),
            h: self.h.unwrap_or(Dim::Full),
            layout: self.layout.unwrap_or(LayoutMode::None),
            padding: self.padding,
            fill: self.fill,
            borders: self.borders,
            corner_radius: self.corner_radius.unwrap_or(BorderRadius::None),
            shadows: self.shadows,
            children: self.children,
        }
    }
}

// ── Recipe Plan & Context ──────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecipePlan {
    pub schema_version: u32,
    pub recipe_id: String,
    pub root: RecipeNode,
}

impl RecipePlan {
    pub fn new(recipe_id: impl Into<String>, root: RecipeNode) -> Self {
        Self {
            schema_version: RECIPE_GRAPH_SCHEMA_VERSION,
            recipe_id: recipe_id.into(),
            root,
        }
    }

    pub fn structural_metrics(&self) -> RecipeStructuralMetrics {
        let mut metrics = RecipeStructuralMetrics::default();
        measure_node(&self.root, 1, &mut metrics);
        metrics
    }
}

// ── Structural Telemetry Metrics ───────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct RecipeStructuralMetrics {
    pub graph_nodes: usize,
    pub expanded_ops: usize,
    pub max_depth: usize,
    pub fill_ops: usize,
    pub gradient_ops: usize,
    pub border_ops: usize,
    pub shadow_lobes: usize,
    pub texture_count: usize,
    pub texture_pixels: usize,
    pub texture_rgba_bytes: usize,
    pub painted_paths: usize,
    pub path_segments: usize,
    pub sampled_points: usize,
}

fn measure_node(node: &RecipeNode, depth: usize, metrics: &mut RecipeStructuralMetrics) {
    metrics.graph_nodes += 1;
    metrics.max_depth = metrics.max_depth.max(depth);

    match node {
        RecipeNode::Rect {
            fill,
            borders,
            shadows,
            children,
            ..
        } => {
            if let Some(f) = fill {
                match f {
                    FillSpec::Solid { .. } => {
                        metrics.fill_ops += 1;
                        metrics.expanded_ops += 1;
                    }
                    FillSpec::LinearGradient2 { .. } => {
                        metrics.gradient_ops += 1;
                        metrics.expanded_ops += 1;
                    }
                    FillSpec::Texture { width, height, .. } => {
                        metrics.texture_count += 1;
                        let px_count = (*width as usize) * (*height as usize);
                        metrics.texture_pixels += px_count;
                        metrics.texture_rgba_bytes += px_count * 4;
                        metrics.expanded_ops += 1;
                    }
                }
            }

            if borders.uniform.is_some() {
                metrics.border_ops += 1;
                metrics.expanded_ops += 1;
            }
            if borders.top.is_some() {
                metrics.border_ops += 1;
                metrics.expanded_ops += 1;
            }
            if borders.bottom.is_some() {
                metrics.border_ops += 1;
                metrics.expanded_ops += 1;
            }

            metrics.shadow_lobes += shadows.len();
            metrics.expanded_ops += shadows.len();

            for child in children {
                measure_node(child, depth + 1, metrics);
            }
        }
        RecipeNode::Streamlines { lines, .. } => {
            metrics.expanded_ops += lines;
            metrics.painted_paths += lines;
            metrics.path_segments += lines * 119;
            metrics.sampled_points += lines * 120;
        }
        RecipeNode::DirectionArrows { count, .. } => {
            metrics.expanded_ops += count * 3;
            metrics.painted_paths += count * 3;
            metrics.path_segments += count * 3;
            metrics.sampled_points += count * 6;
        }
    }
}

// ── Stock GPUI Compiler ────────────────────────────────────────────────

pub fn compile_stock_gpui(plan: &RecipePlan) -> AnyElement {
    enum Action<'a> {
        Expand(&'a RecipeNode),
        BuildRect(&'a RecipeNode, usize),
    }

    let mut actions = vec![Action::Expand(&plan.root)];
    let mut results: Vec<AnyElement> = Vec::new();

    while let Some(action) = actions.pop() {
        match action {
            Action::Expand(node) => match node {
                RecipeNode::Streamlines { .. } => {
                    results.push(compile_streamlines(node));
                }
                RecipeNode::DirectionArrows { .. } => {
                    results.push(compile_direction_arrows(node));
                }
                RecipeNode::Rect {
                    fill: Some(FillSpec::Texture { .. }),
                    ..
                } => {
                    results.push(compile_texture_rect(node));
                }
                RecipeNode::Rect { children, .. } => {
                    actions.push(Action::BuildRect(node, children.len()));
                    for child in children.iter().rev() {
                        actions.push(Action::Expand(child));
                    }
                }
            },
            Action::BuildRect(node, child_count) => {
                let start_idx = results.len() - child_count;
                let child_elements: Vec<AnyElement> = results.drain(start_idx..).collect();
                results.push(compile_rect_element(node, child_elements));
            }
        }
    }

    results.pop().expect("root element must exist")
}

fn compile_texture_rect(node: &RecipeNode) -> AnyElement {
    if let RecipeNode::Rect {
        w,
        h,
        fill:
            Some(FillSpec::Texture {
                kind,
                seed,
                width,
                height,
            }),
        ..
    } = node
    {
        let rendered_img = texture::texture_for(*kind, *seed, *width, *height);
        let mut el = img(rendered_img);
        match w {
            Dim::Full => el = el.w_full(),
            Dim::Px(val) => el = el.w(px(*val)),
            Dim::Flex(_) => el = el.w_full(),
        }
        match h {
            Dim::Full => el = el.h_full(),
            Dim::Px(val) => el = el.h(px(*val)),
            Dim::Flex(_) => el = el.h_full(),
        }
        el.object_fit(ObjectFit::Fill).into_any_element()
    } else {
        unreachable!()
    }
}

fn compile_rect_element(node: &RecipeNode, child_elements: Vec<AnyElement>) -> AnyElement {
    if let RecipeNode::Rect {
        w,
        h,
        layout,
        padding,
        fill,
        borders,
        corner_radius,
        shadows,
        ..
    } = node
    {
        let mut div = div();

        // Dimensions
        match w {
            Dim::Full => div = div.w_full(),
            Dim::Px(val) => div = div.w(px(*val)),
            Dim::Flex(_) => div = div.flex_1(),
        }
        match h {
            Dim::Full => div = div.h_full(),
            Dim::Px(val) => div = div.h(px(*val)),
            Dim::Flex(_) => {}
        }

        // Layout
        match layout {
            LayoutMode::None => {}
            LayoutMode::Center => {
                div = div.flex().items_center().justify_center();
            }
            LayoutMode::FlexRow => {
                div = div.flex().flex_row();
            }
            LayoutMode::FlexCol => {
                div = div.flex().flex_col();
            }
            LayoutMode::JustifyBetweenCol => {
                div = div.flex().flex_col().justify_between();
            }
            LayoutMode::AbsoluteFill => {
                div = div.size_full().absolute();
            }
            LayoutMode::Relative => {
                div = div.relative();
            }
        }

        // Padding
        if *padding > 0.0 {
            div = div.p(px(*padding));
        }

        // Fill
        if let Some(f) = fill {
            match f {
                FillSpec::Solid { color } => {
                    div = div.bg(color.to_gpui_hsla());
                }
                FillSpec::LinearGradient2 {
                    angle_deg,
                    start,
                    end,
                } => {
                    div = div.bg(linear_gradient(
                        *angle_deg,
                        linear_color_stop(start.to_gpui_hsla(), 0.0),
                        linear_color_stop(end.to_gpui_hsla(), 1.0),
                    ));
                }
                FillSpec::Texture { .. } => unreachable!(),
            }
        }

        // Corner radius
        match corner_radius {
            BorderRadius::None => {}
            BorderRadius::Md => div = div.rounded_md(),
            BorderRadius::Lg => div = div.rounded_lg(),
            BorderRadius::Xl => div = div.rounded_xl(),
            BorderRadius::Full => div = div.rounded_full(),
            BorderRadius::Px(val) => div = div.rounded(px(*val)),
        }

        // Uniform border
        if let Some(b) = borders.uniform {
            if b.width >= 2.0 {
                div = div.border_2();
            } else {
                div = div.border_1();
            }
            div = div.border_color(b.color.to_gpui_hsla());
        }

        // Directional borders
        if let Some(b) = borders.top {
            if b.width >= 2.0 {
                div = div.border_t_2();
            } else {
                div = div.border_t_1();
            }
            div = div.border_color(b.color.to_gpui_hsla());
        }

        if let Some(b) = borders.bottom {
            div = div.border_b_1().border_color(b.color.to_gpui_hsla());
        }

        // Shadows
        if !shadows.is_empty() {
            let gpui_shadows: Vec<BoxShadow> = shadows
                .iter()
                .map(|s| BoxShadow {
                    color: s.color.to_gpui_hsla(),
                    offset: point(px(s.offset_x), px(s.offset_y)),
                    blur_radius: px(s.blur_radius),
                    spread_radius: px(s.spread_radius),
                })
                .collect();
            div = div.shadow(gpui_shadows);
        }

        // Children
        for child in child_elements {
            div = div.child(child);
        }

        div.into_any_element()
    } else {
        unreachable!()
    }
}

fn compile_streamlines(node: &RecipeNode) -> AnyElement {
    if let RecipeNode::Streamlines {
        lines,
        stroke_width,
        color,
    } = node
    {
        let line_count = *lines;
        let sw = *stroke_width;
        let c = color.to_gpui_hsla();

        canvas(
            |_bounds, _window, _cx| {},
            move |bounds, _, window, _cx| {
                let w: f32 = bounds.size.width.into();
                let h: f32 = bounds.size.height.into();
                let ox: f32 = bounds.origin.x.into();
                let oy: f32 = bounds.origin.y.into();
                for line in 0..line_count {
                    let mut u = -0.44f32;
                    let mut v = (line as f32 / (line_count - 1).max(1) as f32 - 0.5) * 0.76;
                    let mut pts = Vec::with_capacity(120);
                    for _ in 0..120 {
                        pts.push((ox + w * (u * 0.44 + 0.5), oy + h * (v * 0.60 + 0.5)));
                        let r1 = ((u + 0.52).powi(2) + v * v + 0.015).powf(1.5);
                        let r2 = ((u - 0.52).powi(2) + v * v + 0.015).powf(1.5);
                        let ex = (u + 0.52) / r1 - (u - 0.52) / r2;
                        let ey = v / r1 - v / r2;
                        let len = (ex * ex + ey * ey).sqrt().max(0.01);
                        u += 0.018 * ex / len;
                        v += 0.018 * ey / len;
                        if u.abs() > 0.95 || v.abs() > 0.72 {
                            break;
                        }
                    }
                    let mut p = PathBuilder::stroke(px(sw));
                    if let Some(&(x, y)) = pts.first() {
                        p.move_to(point(px(x), px(y)));
                    }
                    for &(x, y) in pts.iter().skip(1) {
                        p.line_to(point(px(x), px(y)));
                    }
                    if let Ok(built) = p.build() {
                        window.paint_path(built, c);
                    }
                }
            },
        )
        .size_full()
        .into_any_element()
    } else {
        unreachable!()
    }
}

fn compile_direction_arrows(node: &RecipeNode) -> AnyElement {
    if let RecipeNode::DirectionArrows {
        count,
        stroke_width,
        color,
        shaft_length,
        head_length,
        head_width,
    } = node
    {
        let count = *count;
        let sw = *stroke_width;
        let c = color.to_gpui_hsla();
        let shaft = *shaft_length;
        let head = *head_length;
        let head_width = *head_width;
        canvas(
            |_bounds, _window, _cx| {},
            move |bounds, _, window, _cx| {
                let w: f32 = bounds.size.width.into();
                let h: f32 = bounds.size.height.into();
                let ox: f32 = bounds.origin.x.into();
                let oy: f32 = bounds.origin.y.into();
                let spacing = w / (count.max(1) as f32 + 1.0);
                let center_y = oy + h * 0.5;
                for index in 0..count {
                    let center_x = ox + spacing * (index as f32 + 1.0);
                    let tip_x = center_x + shaft * 0.5;
                    let base_x = tip_x - head;
                    let shaft_start = center_x - shaft * 0.5;
                    let mut paths = [
                        (
                            point(px(shaft_start), px(center_y)),
                            point(px(base_x), px(center_y)),
                        ),
                        (
                            point(px(base_x), px(center_y - head_width)),
                            point(px(tip_x), px(center_y)),
                        ),
                        (
                            point(px(base_x), px(center_y + head_width)),
                            point(px(tip_x), px(center_y)),
                        ),
                    ];
                    for (start, end) in &mut paths {
                        let mut path = PathBuilder::stroke(px(sw));
                        path.move_to(*start);
                        path.line_to(*end);
                        if let Ok(built) = path.build() {
                            window.paint_path(built, c);
                        }
                    }
                }
            },
        )
        .size_full()
        .into_any_element()
    } else {
        unreachable!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn test_recipe_plan_serialization_round_trip() {
        let node = RecipeNode::rect()
            .size_full()
            .linear_gradient(
                90.0,
                ColorSpec::rgb(0x0e, 0x1b, 0x24),
                ColorSpec::rgb(0x20, 0xa8, 0x94),
            )
            .build();
        let plan = RecipePlan::new("g01-linear-scalar-gradient/r1", node);

        let json = serde_json::to_string_pretty(&plan).expect("plan serializes");
        let deserialized: RecipePlan = serde_json::from_str(&json).expect("plan deserializes");

        assert_eq!(plan, deserialized);
        let metrics = plan.structural_metrics();
        assert_eq!(metrics.graph_nodes, 1);
        assert_eq!(metrics.gradient_ops, 1);
        assert_eq!(metrics.max_depth, 1);
    }
}
