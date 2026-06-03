use serde::{Deserialize, Serialize};

use crate::{
    CommonName, LineStyle,
    catalog::EquatorialPosition,
    render::{RenderFigure, RenderFigureKind, RenderLine, RenderSkyCulture},
};

/// Two-dimensional point in an output/projected coordinate system.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PlanePoint {
    pub x: f64,
    pub y: f64,
}

impl PlanePoint {
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }
}

/// Simple projection options for debugging and SVG export.
///
/// The default is a whole-sky equirectangular view with RA `0h..24h` mapped left
/// to right and declination `+90°..-90°` mapped top to bottom.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ProjectionOptions {
    pub width: f64,
    pub height: f64,
    pub center_ra_hours: f64,
    pub ra_span_hours: f64,
    pub center_dec_deg: f64,
    pub dec_span_deg: f64,
    pub invert_x: bool,
    pub invert_y: bool,
}

impl Default for ProjectionOptions {
    fn default() -> Self {
        Self {
            width: 1200.0,
            height: 600.0,
            center_ra_hours: 12.0,
            ra_span_hours: 24.0,
            center_dec_deg: 0.0,
            dec_span_deg: 180.0,
            invert_x: false,
            invert_y: false,
        }
    }
}

/// A sky culture after projection into two-dimensional coordinates.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectedSkyCulture {
    pub id: String,
    pub region: Option<String>,
    pub width: f64,
    pub height: f64,
    pub figures: Vec<ProjectedFigure>,
}

/// A projected constellation or asterism.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectedFigure {
    pub id: String,
    pub kind: RenderFigureKind,
    pub lines: Vec<ProjectedLine>,
    pub label: Option<ProjectedLabel>,
    pub is_ray_helper: bool,
}

/// A projected polyline/path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectedLine {
    pub style: Option<LineStyle>,
    pub points: Vec<PlanePoint>,
}

/// Projected label placement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectedLabel {
    pub text: String,
    pub point: PlanePoint,
}

/// Project a render-ready sky culture using an equirectangular projection.
pub fn project_render_sky_culture(
    culture: &RenderSkyCulture,
    options: ProjectionOptions,
) -> ProjectedSkyCulture {
    ProjectedSkyCulture {
        id: culture.id.clone(),
        region: culture.region.clone(),
        width: options.width,
        height: options.height,
        figures: culture
            .figures
            .iter()
            .map(|figure| project_figure(figure, options))
            .collect(),
    }
}

/// Project a single equatorial point using the configured equirectangular view.
pub fn project_position(position: EquatorialPosition, options: ProjectionOptions) -> PlanePoint {
    let delta_ra = wrapped_ra_delta_hours(position.ra_hours, options.center_ra_hours);
    let mut x_norm = 0.5 + delta_ra / options.ra_span_hours;
    let mut y_norm = 0.5 - (position.dec_deg - options.center_dec_deg) / options.dec_span_deg;

    if options.invert_x {
        x_norm = 1.0 - x_norm;
    }
    if options.invert_y {
        y_norm = 1.0 - y_norm;
    }

    PlanePoint::new(x_norm * options.width, y_norm * options.height)
}

fn project_figure(figure: &RenderFigure, options: ProjectionOptions) -> ProjectedFigure {
    let lines: Vec<_> = figure
        .lines
        .iter()
        .map(|line| project_line(line, options))
        .collect();

    let label = figure_label_text(figure)
        .and_then(|text| figure_label_position(figure, &lines, options).map(|point| (text, point)))
        .map(|(text, point)| ProjectedLabel { text, point });

    ProjectedFigure {
        id: figure.id.clone(),
        kind: figure.kind,
        lines,
        label,
        is_ray_helper: figure.is_ray_helper,
    }
}

fn project_line(line: &RenderLine, options: ProjectionOptions) -> ProjectedLine {
    ProjectedLine {
        style: line.style,
        points: line
            .points
            .iter()
            .map(|point| project_position(point.position, options))
            .collect(),
    }
}

fn figure_label_text(figure: &RenderFigure) -> Option<String> {
    figure
        .common_name
        .as_ref()
        .and_then(common_name_label)
        .or_else(|| Some(figure.id.clone()))
}

fn common_name_label(common_name: &CommonName) -> Option<String> {
    common_name
        .english
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            common_name
                .native
                .as_deref()
                .filter(|value| !value.trim().is_empty())
        })
        .map(ToOwned::to_owned)
}

fn figure_label_position(
    figure: &RenderFigure,
    lines: &[ProjectedLine],
    options: ProjectionOptions,
) -> Option<PlanePoint> {
    if let Some(position) = figure.label_positions.first() {
        return Some(project_position(*position, options));
    }

    projected_centroid(lines)
}

fn projected_centroid(lines: &[ProjectedLine]) -> Option<PlanePoint> {
    let mut count = 0_u32;
    let mut x_sum = 0.0;
    let mut y_sum = 0.0;

    for point in lines.iter().flat_map(|line| &line.points) {
        count += 1;
        x_sum += point.x;
        y_sum += point.y;
    }

    (count > 0).then(|| PlanePoint::new(x_sum / f64::from(count), y_sum / f64::from(count)))
}

fn wrapped_ra_delta_hours(ra_hours: f64, center_ra_hours: f64) -> f64 {
    let mut delta = (ra_hours - center_ra_hours).rem_euclid(24.0);
    if delta >= 12.0 {
        delta -= 24.0;
    }
    delta
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projects_whole_sky_defaults() {
        let options = ProjectionOptions::default();
        assert_eq!(
            project_position(EquatorialPosition::new(12.0, 0.0), options),
            PlanePoint::new(600.0, 300.0)
        );
        assert_eq!(
            project_position(EquatorialPosition::new(0.0, 90.0), options),
            PlanePoint::new(0.0, 0.0)
        );
        assert_eq!(
            project_position(EquatorialPosition::new(0.0, -90.0), options),
            PlanePoint::new(0.0, 600.0)
        );
    }

    #[test]
    fn wraps_ra_near_center() {
        assert_eq!(wrapped_ra_delta_hours(23.0, 0.0), -1.0);
        assert_eq!(wrapped_ra_delta_hours(1.0, 0.0), 1.0);
    }
}
