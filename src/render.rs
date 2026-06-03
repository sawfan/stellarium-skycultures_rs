use rubrum_star_catalog::StarCatalog as RubrumStarCatalog;
use serde::{Deserialize, Serialize};

use crate::{
    Asterism, CommonName, Constellation, FigureRef, Illustration, Line, LineNode, LineStyle,
    ObjectRef, SkyCultureIndex,
    catalog::{EquatorialPosition, RubrumStarCatalogAdapter, SkyPositionCatalog},
};

/// Render-ready version of a sky culture.
///
/// This structure has the same culture/figure identity as the parsed Stellarium
/// source, but all resolvable line nodes have been converted to sky coordinates.
/// It is deliberately projection-agnostic: an SVG renderer can project the
/// `EquatorialPosition` values into whatever view/crop it needs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderSkyCulture {
    pub id: String,
    pub region: Option<String>,
    pub figures: Vec<RenderFigure>,
    pub diagnostics: RenderDiagnostics,
}

impl RenderSkyCulture {
    pub fn unresolved_items(&self) -> impl Iterator<Item = &UnresolvedLineNode> {
        self.diagnostics.unresolved.iter()
    }
}

/// Render-ready constellation or asterism.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderFigure {
    pub id: String,
    pub kind: RenderFigureKind,
    pub common_name: Option<CommonName>,
    pub lines: Vec<RenderLine>,
    pub image: Option<Illustration>,
    pub thumbnail: Option<String>,
    pub iau: Option<String>,
    pub description: Option<String>,
    pub label_offset: Option<[f64; 2]>,
    pub label_positions: Vec<EquatorialPosition>,
    pub single_star_radius: Option<f64>,
    pub is_ray_helper: bool,
}

/// Source-kind for a render-ready sky figure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RenderFigureKind {
    Constellation,
    Asterism,
}

/// Render-ready polyline/path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderLine {
    pub style: Option<LineStyle>,
    pub points: Vec<RenderPoint>,
}

impl RenderLine {
    pub fn is_empty(&self) -> bool {
        self.points.is_empty()
    }
}

/// A resolved point in a render-ready line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RenderPoint {
    pub position: EquatorialPosition,
    pub source: RenderPointSource,
}

/// Where a render point came from.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RenderPointSource {
    Hip(u32),
    DeepSkyObject(String),
    Literal,
}

/// Non-fatal issues encountered while creating render-ready data.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderDiagnostics {
    pub unresolved: Vec<UnresolvedLineNode>,
    pub skipped_empty_lines: usize,
}

impl RenderDiagnostics {
    pub fn is_clean(&self) -> bool {
        self.unresolved.is_empty() && self.skipped_empty_lines == 0
    }
}

/// A source line node that could not be resolved through the provided catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnresolvedLineNode {
    pub figure_id: String,
    pub line_index: usize,
    pub node_index: usize,
    pub node: LineNode,
}

/// Options for render-ready normalization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RenderOptions {
    /// If true, lines with unresolved nodes are split into continuous resolved
    /// segments. If false, unresolved nodes are simply omitted from the line.
    pub split_lines_at_unresolved_nodes: bool,

    /// If true, empty render lines are omitted.
    pub drop_empty_lines: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            split_lines_at_unresolved_nodes: true,
            drop_empty_lines: true,
        }
    }
}

/// Convert a parsed sky-culture index to projection-agnostic render data.
pub fn build_render_sky_culture(
    index: &SkyCultureIndex,
    catalog: &impl SkyPositionCatalog,
) -> RenderSkyCulture {
    build_render_sky_culture_with_options(index, catalog, RenderOptions::default())
}

/// Convert a parsed sky-culture index using any generic `rubrum-star-catalog`
/// implementation that can resolve HIP IDs.
pub fn build_render_sky_culture_from_star_catalog(
    index: &SkyCultureIndex,
    catalog: &impl RubrumStarCatalog,
) -> RenderSkyCulture {
    build_render_sky_culture_from_star_catalog_with_options(
        index,
        catalog,
        RenderOptions::default(),
    )
}

/// Convert a parsed sky-culture index using a generic `rubrum-star-catalog`
/// implementation and explicit render options.
pub fn build_render_sky_culture_from_star_catalog_with_options(
    index: &SkyCultureIndex,
    catalog: &impl RubrumStarCatalog,
    options: RenderOptions,
) -> RenderSkyCulture {
    let adapter = RubrumStarCatalogAdapter::new(catalog);
    build_render_sky_culture_with_options(index, &adapter, options)
}

/// Convert a parsed sky-culture index to projection-agnostic render data with options.
pub fn build_render_sky_culture_with_options(
    index: &SkyCultureIndex,
    catalog: &impl SkyPositionCatalog,
    options: RenderOptions,
) -> RenderSkyCulture {
    let mut diagnostics = RenderDiagnostics::default();
    let mut figures = Vec::with_capacity(index.constellations.len() + index.asterisms.len());

    for constellation in &index.constellations {
        figures.push(render_constellation(
            constellation,
            catalog,
            options,
            &mut diagnostics,
        ));
    }

    for asterism in &index.asterisms {
        figures.push(render_asterism(
            asterism,
            catalog,
            options,
            &mut diagnostics,
        ));
    }

    RenderSkyCulture {
        id: index.id.clone(),
        region: index.region.clone(),
        figures,
        diagnostics,
    }
}

/// Convert an arbitrary borrowed [`FigureRef`] to render-ready data.
pub fn build_render_figure(
    figure: FigureRef<'_>,
    catalog: &impl SkyPositionCatalog,
    options: RenderOptions,
    diagnostics: &mut RenderDiagnostics,
) -> RenderFigure {
    match figure {
        FigureRef::Constellation(value) => {
            render_constellation(value, catalog, options, diagnostics)
        }
        FigureRef::Asterism(value) => render_asterism(value, catalog, options, diagnostics),
    }
}

fn render_constellation(
    constellation: &Constellation,
    catalog: &impl SkyPositionCatalog,
    options: RenderOptions,
    diagnostics: &mut RenderDiagnostics,
) -> RenderFigure {
    RenderFigure {
        id: constellation.id.clone(),
        kind: RenderFigureKind::Constellation,
        common_name: constellation.common_name.clone(),
        lines: render_lines(
            &constellation.id,
            &constellation.lines,
            catalog,
            options,
            diagnostics,
        ),
        image: constellation.image.clone(),
        thumbnail: constellation.thumbnail.clone(),
        iau: constellation.iau.clone(),
        description: constellation.description_text().map(ToOwned::to_owned),
        label_offset: constellation.label_offset,
        label_positions: constellation
            .label_positions
            .iter()
            .map(|[ra_hours, dec_deg]| EquatorialPosition::new(*ra_hours, *dec_deg))
            .collect(),
        single_star_radius: constellation.single_star_radius,
        is_ray_helper: false,
    }
}

fn render_asterism(
    asterism: &Asterism,
    catalog: &impl SkyPositionCatalog,
    options: RenderOptions,
    diagnostics: &mut RenderDiagnostics,
) -> RenderFigure {
    RenderFigure {
        id: asterism.id.clone(),
        kind: RenderFigureKind::Asterism,
        common_name: asterism.common_name.clone(),
        lines: render_lines(&asterism.id, &asterism.lines, catalog, options, diagnostics),
        image: None,
        thumbnail: None,
        iau: None,
        description: None,
        label_offset: None,
        label_positions: Vec::new(),
        single_star_radius: None,
        is_ray_helper: asterism.is_ray_helper,
    }
}

fn render_lines(
    figure_id: &str,
    lines: &[Line],
    catalog: &impl SkyPositionCatalog,
    options: RenderOptions,
    diagnostics: &mut RenderDiagnostics,
) -> Vec<RenderLine> {
    let mut rendered = Vec::new();

    for (line_index, line) in lines.iter().enumerate() {
        let style = line.iter().find_map(LineNode::as_style);
        let mut current_points = Vec::new();

        for (node_index, node) in line.iter().enumerate() {
            if node.as_style().is_some() {
                continue;
            }

            match resolve_line_node(node, catalog) {
                Some(point) => current_points.push(point),
                None => {
                    diagnostics.unresolved.push(UnresolvedLineNode {
                        figure_id: figure_id.to_string(),
                        line_index,
                        node_index,
                        node: node.clone(),
                    });

                    if options.split_lines_at_unresolved_nodes {
                        flush_render_line(
                            &mut rendered,
                            &mut current_points,
                            style,
                            options,
                            diagnostics,
                        );
                    }
                }
            }
        }

        flush_render_line(
            &mut rendered,
            &mut current_points,
            style,
            options,
            diagnostics,
        );
    }

    rendered
}

fn flush_render_line(
    rendered: &mut Vec<RenderLine>,
    current_points: &mut Vec<RenderPoint>,
    style: Option<LineStyle>,
    options: RenderOptions,
    diagnostics: &mut RenderDiagnostics,
) {
    if current_points.is_empty() && options.drop_empty_lines {
        diagnostics.skipped_empty_lines += 1;
        return;
    }

    rendered.push(RenderLine {
        style,
        points: std::mem::take(current_points),
    });
}

fn resolve_line_node(node: &LineNode, catalog: &impl SkyPositionCatalog) -> Option<RenderPoint> {
    match node.as_object_ref()? {
        ObjectRef::Hip(hip) => catalog.hip_position(hip).map(|position| RenderPoint {
            position,
            source: RenderPointSource::Hip(hip),
        }),
        ObjectRef::DeepSkyObject(object_id) => {
            catalog
                .object_position(object_id)
                .map(|position| RenderPoint {
                    position,
                    source: RenderPointSource::DeepSkyObject(object_id.to_string()),
                })
        }
        ObjectRef::Equatorial { ra_hours, dec_deg } => Some(RenderPoint {
            position: EquatorialPosition::new(ra_hours, dec_deg),
            source: RenderPointSource::Literal,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LineNode, catalog::HipStarCatalog};

    #[test]
    fn resolves_hip_lines_and_preserves_style() {
        let index = SkyCultureIndex {
            id: "sample".to_string(),
            region: None,
            classification: Vec::new(),
            thumbnail: None,
            thumbnail_bscale: None,
            highlight: None,
            illustrations_bscale: None,
            fallback_to_international_names: false,
            langs_use_native_names: Vec::new(),
            native_lang: None,
            constellations: vec![Constellation {
                id: "CON sample Foo".to_string(),
                lines: vec![vec![
                    LineNode::Text("thin".to_string()),
                    LineNode::Hip(1),
                    LineNode::Hip(2),
                ]],
                image: None,
                thumbnail: None,
                common_name: None,
                iau: None,
                description: None,
                misspelled_description: None,
                label_offset: None,
                label_positions: Vec::new(),
                single_star_radius: None,
            }],
            asterisms: Vec::new(),
            common_names: Default::default(),
            edges_source: None,
            edges_epoch: None,
            edges_type: None,
            edges: Vec::new(),
            lunar_system: None,
        };

        let mut catalog = HipStarCatalog::new();
        catalog.insert_hip(1, EquatorialPosition::new(1.0, 10.0));
        catalog.insert_hip(2, EquatorialPosition::new(2.0, 20.0));

        let rendered = build_render_sky_culture(&index, &catalog);
        assert!(rendered.diagnostics.is_clean());
        assert_eq!(rendered.figures.len(), 1);
        assert_eq!(rendered.figures[0].lines.len(), 1);
        assert_eq!(rendered.figures[0].lines[0].style, Some(LineStyle::Thin));
        assert_eq!(rendered.figures[0].lines[0].points.len(), 2);
    }

    #[test]
    fn splits_lines_at_unresolved_nodes() {
        let line = vec![LineNode::Hip(1), LineNode::Hip(999), LineNode::Hip(2)];
        let mut catalog = HipStarCatalog::new();
        catalog.insert_hip(1, EquatorialPosition::new(1.0, 10.0));
        catalog.insert_hip(2, EquatorialPosition::new(2.0, 20.0));
        let mut diagnostics = RenderDiagnostics::default();

        let rendered = render_lines(
            "figure",
            &[line],
            &catalog,
            RenderOptions::default(),
            &mut diagnostics,
        );

        assert_eq!(rendered.len(), 2);
        assert_eq!(diagnostics.unresolved.len(), 1);
        assert_eq!(diagnostics.skipped_empty_lines, 0);
    }
}
