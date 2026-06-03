use crate::projection::{PlanePoint, ProjectedFigure, ProjectedLine, ProjectedSkyCulture};

/// Options for serializing projected sky-culture data as SVG.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SvgOptions {
    pub show_labels: bool,
    pub include_styles: bool,
    pub hide_ray_helpers: bool,
    pub show_star_dots: bool,
    pub split_ra_seam: bool,
    pub line_width: f64,
    pub label_font_size: f64,
    pub star_dot_radius: f64,
}

impl Default for SvgOptions {
    fn default() -> Self {
        Self {
            show_labels: true,
            include_styles: true,
            hide_ray_helpers: false,
            show_star_dots: false,
            split_ra_seam: true,
            line_width: 1.5,
            label_font_size: 12.0,
            star_dot_radius: 2.0,
        }
    }
}

/// Serialize projected sky-culture lines and labels to a standalone SVG document.
pub fn projected_sky_culture_to_svg(culture: &ProjectedSkyCulture, options: SvgOptions) -> String {
    let mut svg = String::new();
    svg.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    svg.push_str(&format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {} {}\" width=\"{}\" height=\"{}\" role=\"img\" aria-labelledby=\"title\">\n",
        fmt_num(culture.width),
        fmt_num(culture.height),
        fmt_num(culture.width),
        fmt_num(culture.height)
    ));
    svg.push_str(&format!(
        "  <title id=\"title\">{}</title>\n",
        escape_xml(&culture.id)
    ));

    if options.include_styles {
        svg.push_str("  <style>\n");
        svg.push_str("    .sky-line { fill: none; stroke: currentColor; stroke-linecap: round; stroke-linejoin: round; opacity: 0.9; }\n");
        svg.push_str("    .sky-line--thin { opacity: 0.65; }\n");
        svg.push_str("    .sky-line--bold { stroke-width: 2.4; }\n");
        svg.push_str("    .sky-label { font-family: system-ui, sans-serif; fill: currentColor; paint-order: stroke; stroke: rgba(0,0,0,0.65); stroke-width: 2px; stroke-linejoin: round; }\n");
        svg.push_str("    .sky-star-dot { fill: currentColor; opacity: 0.8; }\n");
        svg.push_str("    .sky-figure--ray-helper { opacity: 0.45; }\n");
        svg.push_str("  </style>\n");
    }

    svg.push_str("  <g class=\"sky-culture\">\n");
    for figure in &culture.figures {
        if options.hide_ray_helpers && figure.is_ray_helper {
            continue;
        }
        push_figure_svg(&mut svg, figure, options, culture.width);
    }
    svg.push_str("  </g>\n");
    svg.push_str("</svg>\n");
    svg
}

fn push_figure_svg(
    svg: &mut String,
    figure: &ProjectedFigure,
    options: SvgOptions,
    culture_width: f64,
) {
    let mut class = String::from("sky-figure");
    if figure.is_ray_helper {
        class.push_str(" sky-figure--ray-helper");
    }

    svg.push_str(&format!(
        "    <g class=\"{}\" data-figure-id=\"{}\">\n",
        class,
        escape_xml_attr(&figure.id)
    ));

    for line in &figure.lines {
        for path in line_to_svg_paths(line, culture_width, options.split_ra_seam) {
            let mut line_class = String::from("sky-line");
            match line.style {
                Some(crate::LineStyle::Thin) => line_class.push_str(" sky-line--thin"),
                Some(crate::LineStyle::Bold) => line_class.push_str(" sky-line--bold"),
                None => {}
            }
            svg.push_str(&format!(
                "      <path class=\"{}\" d=\"{}\" stroke-width=\"{}\"/>\n",
                line_class,
                path,
                fmt_num(options.line_width)
            ));
        }

        if options.show_star_dots {
            for point in unique_line_points(line) {
                svg.push_str(&format!(
                    "      <circle class=\"sky-star-dot\" cx=\"{}\" cy=\"{}\" r=\"{}\"/>\n",
                    fmt_num(point.x),
                    fmt_num(point.y),
                    fmt_num(options.star_dot_radius)
                ));
            }
        }
    }

    if options.show_labels
        && let Some(label) = &figure.label
    {
        svg.push_str(&format!(
            "      <text class=\"sky-label\" x=\"{}\" y=\"{}\" font-size=\"{}\">{}</text>\n",
            fmt_num(label.point.x),
            fmt_num(label.point.y),
            fmt_num(options.label_font_size),
            escape_xml(&label.text)
        ));
    }

    svg.push_str("    </g>\n");
}

/// Convert a projected line to an SVG path `d` attribute.
pub fn line_to_svg_path(line: &ProjectedLine) -> Option<String> {
    let mut paths = line_to_svg_paths(line, f64::INFINITY, false);
    paths.pop()
}

/// Convert a projected line to one or more SVG path `d` attributes.
///
/// When `split_ra_seam` is true, segments with a horizontal jump greater than
/// half the SVG width are split to avoid drawing a long seam-crossing line.
pub fn line_to_svg_paths(line: &ProjectedLine, width: f64, split_ra_seam: bool) -> Vec<String> {
    let point_segments = split_line_points(&line.points, width, split_ra_seam);
    point_segments
        .into_iter()
        .filter_map(|segment| points_to_svg_path(&segment))
        .collect()
}

/// Convert a point list to an SVG path `d` attribute.
pub fn points_to_svg_path(points: &[PlanePoint]) -> Option<String> {
    let mut points = points.iter();
    let first = points.next()?;
    let mut path = format!("M {} {}", fmt_num(first.x), fmt_num(first.y));
    for point in points {
        path.push_str(&format!(" L {} {}", fmt_num(point.x), fmt_num(point.y)));
    }
    Some(path)
}

fn split_line_points(
    points: &[PlanePoint],
    width: f64,
    split_ra_seam: bool,
) -> Vec<Vec<PlanePoint>> {
    if points.is_empty() {
        return Vec::new();
    }

    if !split_ra_seam || !width.is_finite() || width <= 0.0 {
        return vec![points.to_vec()];
    }

    let mut segments: Vec<Vec<PlanePoint>> = Vec::new();
    let mut current = vec![points[0]];
    let threshold = width / 2.0;

    for pair in points.windows(2) {
        let previous = pair[0];
        let next = pair[1];
        if (next.x - previous.x).abs() > threshold && !current.is_empty() {
            segments.push(std::mem::take(&mut current));
        }
        current.push(next);
    }

    if !current.is_empty() {
        segments.push(current);
    }

    segments
}

fn unique_line_points(line: &ProjectedLine) -> Vec<PlanePoint> {
    let mut unique: Vec<PlanePoint> = Vec::new();
    for point in &line.points {
        if !unique
            .iter()
            .any(|seen| (seen.x - point.x).abs() < 0.001 && (seen.y - point.y).abs() < 0.001)
        {
            unique.push(*point);
        }
    }
    unique
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn escape_xml_attr(value: &str) -> String {
    escape_xml(value).replace('"', "&quot;")
}

fn fmt_num(value: f64) -> String {
    let formatted = format!("{value:.3}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{LineStyle, projection::ProjectedLabel, render::RenderFigureKind};

    #[test]
    fn converts_line_to_path() {
        let line = ProjectedLine {
            style: Some(LineStyle::Thin),
            points: vec![PlanePoint::new(1.0, 2.0), PlanePoint::new(3.5, 4.25)],
        };
        assert_eq!(
            line_to_svg_path(&line),
            Some("M 1 2 L 3.5 4.25".to_string())
        );
    }

    #[test]
    fn splits_large_x_jumps_when_requested() {
        let line = ProjectedLine {
            style: None,
            points: vec![
                PlanePoint::new(10.0, 10.0),
                PlanePoint::new(990.0, 20.0),
                PlanePoint::new(980.0, 30.0),
            ],
        };
        assert_eq!(
            line_to_svg_paths(&line, 1000.0, true),
            vec!["M 10 10".to_string(), "M 990 20 L 980 30".to_string()]
        );
        assert_eq!(line_to_svg_paths(&line, 1000.0, false).len(), 1);
    }

    #[test]
    fn hides_ray_helpers_and_renders_star_dots() {
        let culture = ProjectedSkyCulture {
            id: "sample".to_string(),
            region: None,
            width: 100.0,
            height: 50.0,
            figures: vec![
                ProjectedFigure {
                    id: "AST sample helper".to_string(),
                    kind: RenderFigureKind::Asterism,
                    lines: vec![ProjectedLine {
                        style: None,
                        points: vec![PlanePoint::new(1.0, 2.0)],
                    }],
                    label: None,
                    is_ray_helper: true,
                },
                ProjectedFigure {
                    id: "CON sample visible".to_string(),
                    kind: RenderFigureKind::Constellation,
                    lines: vec![ProjectedLine {
                        style: None,
                        points: vec![PlanePoint::new(3.0, 4.0)],
                    }],
                    label: None,
                    is_ray_helper: false,
                },
            ],
        };
        let svg = projected_sky_culture_to_svg(
            &culture,
            SvgOptions {
                hide_ray_helpers: true,
                show_star_dots: true,
                ..Default::default()
            },
        );
        assert!(!svg.contains("AST sample helper"));
        assert!(svg.contains("CON sample visible"));
        assert!(svg.contains("sky-star-dot"));
    }

    #[test]
    fn escapes_label_text() {
        let culture = ProjectedSkyCulture {
            id: "a&b".to_string(),
            region: None,
            width: 100.0,
            height: 50.0,
            figures: vec![ProjectedFigure {
                id: "CON sample <x>".to_string(),
                kind: RenderFigureKind::Constellation,
                lines: Vec::new(),
                label: Some(ProjectedLabel {
                    text: "A & <B>".to_string(),
                    point: PlanePoint::new(10.0, 20.0),
                }),
                is_ray_helper: false,
            }],
        };
        let svg = projected_sky_culture_to_svg(&culture, SvgOptions::default());
        assert!(svg.contains("a&amp;b"));
        assert!(svg.contains("A &amp; &lt;B&gt;"));
        assert!(svg.contains("CON sample &lt;x&gt;"));
    }
}
