use std::{fs, path::PathBuf};

use clap::{Parser, Subcommand};
use rubrum_star_catalog::{CatalogStats, CsvStarCatalog, EquatorialBounds, StarCatalog};
use serde::Serialize;
use stellarium_skycultures::{
    CommonName, EquatorialPosition, RenderDiagnostics, RenderFigure, RenderFigureKind,
    RenderSkyCulture, SkyPositionCatalog, load_culture_dir, load_repository_dir,
    referenced_webp_assets,
};

#[derive(Debug, Parser)]
#[command(
    name = "stellarium-skycultures",
    version,
    about = "Inspect/import Stellarium sky-culture index.json data"
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// List culture directories found in a stellarium-skycultures checkout.
    List {
        /// Path to a local checkout of https://github.com/Stellarium/stellarium-skycultures.
        repo: PathBuf,
    },
    /// Print a compact summary for one culture directory.
    Summary {
        /// Path to a culture directory containing index.json.
        culture_dir: PathBuf,
    },
    /// Summarize a star catalog parse.
    CatalogSummary {
        /// Catalog path. Use a `normalized-*` format for rubrum-star-catalog data.
        catalog: PathBuf,
        /// Catalog format. Legacy `csv`/`tsv` use the Stellarium HIP loaders; `normalized-*` uses rubrum-star-catalog.
        #[arg(long, default_value = "normalized-csv")]
        catalog_format: CatalogFormat,
    },
    /// Report unresolved refs for a culture and catalog.
    MissingRefs {
        /// Path to a culture directory containing index.json.
        culture_dir: PathBuf,
        /// Catalog path. Use a `normalized-*` format for rubrum-star-catalog data.
        catalog: PathBuf,
        /// Catalog format. Legacy `csv`/`tsv` use the Stellarium HIP loaders; `normalized-*` uses rubrum-star-catalog.
        #[arg(long, default_value = "normalized-csv")]
        catalog_format: CatalogFormat,
    },
    /// Build render-ready JSON by resolving HIP/object refs against a star catalog.
    RenderJson {
        /// Path to a culture directory containing index.json.
        culture_dir: PathBuf,
        /// Catalog path. Use a `normalized-*` format for rubrum-star-catalog data.
        catalog: PathBuf,
        /// Catalog format. Legacy `csv`/`tsv` use the Stellarium HIP loaders; `normalized-*` uses rubrum-star-catalog.
        #[arg(long, default_value = "normalized-csv")]
        catalog_format: CatalogFormat,
        /// Pretty-print the JSON.
        #[arg(long)]
        pretty: bool,
    },
    /// Build a simple equirectangular SVG from one culture and a star catalog.
    RenderSvg {
        /// Path to a culture directory containing index.json.
        culture_dir: PathBuf,
        /// Catalog path. Use a `normalized-*` format for rubrum-star-catalog data.
        catalog: PathBuf,
        /// Output SVG path. Writes to stdout when omitted.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Catalog format. Legacy `csv`/`tsv` use the Stellarium HIP loaders; `normalized-*` uses rubrum-star-catalog.
        #[arg(long, default_value = "normalized-csv")]
        catalog_format: CatalogFormat,
        /// SVG width.
        #[arg(long, default_value_t = 1200.0)]
        width: f64,
        /// SVG height.
        #[arg(long, default_value_t = 600.0)]
        height: f64,
        /// Right ascension at the center of the view, in hours.
        #[arg(long, default_value_t = 12.0)]
        ra_center: f64,
        /// Visible right-ascension span, in hours.
        #[arg(long, default_value_t = 24.0)]
        ra_span: f64,
        /// Declination at the center of the view, in degrees.
        #[arg(long, default_value_t = 0.0)]
        dec_center: f64,
        /// Visible declination span, in degrees.
        #[arg(long, default_value_t = 180.0)]
        dec_span: f64,
        /// Mirror the horizontal axis.
        #[arg(long)]
        invert_x: bool,
        /// Mirror the vertical axis.
        #[arg(long)]
        invert_y: bool,
        /// Hide labels.
        #[arg(long)]
        no_labels: bool,
        /// Hide `is_ray_helper` asterisms.
        #[arg(long)]
        hide_ray_helpers: bool,
        /// Draw a dot at each rendered line point.
        #[arg(long)]
        show_star_dots: bool,
        /// Disable seam-aware path splitting.
        #[arg(long)]
        no_seam_split: bool,
        /// SVG line width.
        #[arg(long, default_value_t = 1.5)]
        line_width: f64,
        /// Label font size.
        #[arg(long, default_value_t = 12.0)]
        label_font_size: f64,
    },
    /// Build one SVG per constellation/figure plus a manifest.
    RenderSvgAll {
        /// Path to a culture directory containing index.json.
        #[arg(long)]
        culture_dir: PathBuf,
        /// Catalog path. Use a `normalized-*` format for rubrum-star-catalog data.
        #[arg(long)]
        catalog: PathBuf,
        /// Output directory for SVG files and index.json.
        #[arg(long)]
        output_dir: PathBuf,
        /// Catalog format. Legacy `csv`/`tsv` use the Stellarium HIP loaders; `normalized-*` uses rubrum-star-catalog.
        #[arg(long, default_value = "normalized-csv")]
        catalog_format: CatalogFormat,
        /// Include asterisms in addition to constellations.
        #[arg(long)]
        include_asterisms: bool,
        /// SVG width.
        #[arg(long, default_value_t = 1200.0)]
        width: f64,
        /// SVG height.
        #[arg(long, default_value_t = 600.0)]
        height: f64,
        /// Right ascension at the center of the view, in hours.
        #[arg(long, default_value_t = 12.0)]
        ra_center: f64,
        /// Visible right-ascension span, in hours.
        #[arg(long, default_value_t = 24.0)]
        ra_span: f64,
        /// Declination at the center of the view, in degrees.
        #[arg(long, default_value_t = 0.0)]
        dec_center: f64,
        /// Visible declination span, in degrees.
        #[arg(long, default_value_t = 180.0)]
        dec_span: f64,
        /// Mirror the horizontal axis.
        #[arg(long)]
        invert_x: bool,
        /// Mirror the vertical axis.
        #[arg(long)]
        invert_y: bool,
        /// Hide labels.
        #[arg(long)]
        no_labels: bool,
        /// Hide `is_ray_helper` asterisms.
        #[arg(long)]
        hide_ray_helpers: bool,
        /// Draw a dot at each rendered line point.
        #[arg(long)]
        show_star_dots: bool,
        /// Disable seam-aware path splitting.
        #[arg(long)]
        no_seam_split: bool,
        /// Fit each generated SVG projection to its figure instead of using the shared full-sky frame.
        #[arg(long)]
        fit_figures: bool,
        /// Padding multiplier used with --fit-figures.
        #[arg(long, default_value_t = 1.25)]
        fit_padding: f64,
        /// Minimum right-ascension span, in hours, used with --fit-figures.
        #[arg(long, default_value_t = 1.0)]
        min_ra_span: f64,
        /// Minimum declination span, in degrees, used with --fit-figures.
        #[arg(long, default_value_t = 10.0)]
        min_dec_span: f64,
        /// Copy figure illustration assets into the output directory and include them in the manifest.
        #[arg(long)]
        copy_illustrations: bool,
        /// Normalize copied raster illustrations to transparent PNGs by preserving existing alpha
        /// or removing likely black/dark edge-connected Stellarium color-key backgrounds.
        #[arg(long)]
        normalize_illustrations: bool,
        /// Maximum RGB channel value considered black/dark background during illustration normalization.
        #[arg(long, default_value_t = 32)]
        transparent_black_threshold: u8,
        /// Minimum fraction of sampled edge pixels that must be dark before background removal is applied.
        #[arg(long, default_value_t = 0.60)]
        transparent_edge_fraction: f64,
        /// SVG line width.
        #[arg(long, default_value_t = 1.5)]
        line_width: f64,
        /// Label font size.
        #[arg(long, default_value_t = 12.0)]
        label_font_size: f64,
    },
    /// Emit normalized JSON for one culture directory.
    ToJson {
        /// Path to a culture directory containing index.json.
        culture_dir: PathBuf,
        /// Pretty-print the JSON.
        #[arg(long)]
        pretty: bool,
    },
}

#[derive(Debug, Clone, Copy, clap::ValueEnum)]
enum CatalogFormat {
    /// Normalized CSV generated by rubrum-star-catalog-hipparcos or another backend.
    NormalizedCsv,
    /// Normalized TSV generated by rubrum-star-catalog or another backend.
    NormalizedTsv,
    /// Normalized pipe-delimited catalog generated by rubrum-star-catalog or another backend.
    NormalizedPipe,
    /// Legacy Stellarium helper CSV containing HIP + RA/Dec columns.
    Csv,
    /// Legacy Stellarium helper TSV containing HIP + RA/Dec columns.
    Tsv,
}

impl CatalogFormat {
    fn normalized_delimiter(self) -> Option<u8> {
        match self {
            Self::NormalizedCsv => Some(b','),
            Self::NormalizedTsv => Some(b'\t'),
            Self::NormalizedPipe => Some(b'|'),
            Self::Csv | Self::Tsv => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct StarVisualStyle {
    radius: f64,
    color: &'static str,
}

enum LoadedCatalog {
    Normalized(CsvStarCatalog),
    Legacy(stellarium_skycultures::HipStarCatalog),
}

impl LoadedCatalog {
    fn len_hips(&self) -> usize {
        match self {
            Self::Normalized(catalog) => {
                catalog.iter().filter(|record| record.hip.is_some()).count()
            }
            Self::Legacy(catalog) => catalog.len_hips(),
        }
    }

    fn len_objects(&self) -> usize {
        match self {
            Self::Normalized(_) => 0,
            Self::Legacy(catalog) => catalog.len_objects(),
        }
    }

    fn star_visual_style(&self, hip: u32) -> Option<StarVisualStyle> {
        let Self::Normalized(catalog) = self else {
            return None;
        };
        let record = catalog.get(rubrum_star_catalog::StarId::Hip(hip))?;
        let color = record
            .color_index_bv
            .and_then(rubrum_star_catalog::display_color_from_bv)
            .or_else(|| {
                record
                    .spectral_type
                    .as_deref()
                    .and_then(rubrum_star_catalog::display_color_from_spectral_type)
            })
            .unwrap_or("currentColor");
        let radius = record
            .visual_mag
            .and_then(rubrum_star_catalog::display_radius_from_visual_mag)
            .unwrap_or(2.0);
        Some(StarVisualStyle { radius, color })
    }

    fn star_manifest_entry(&self, hip: u32) -> Option<SvgManifestStar> {
        let position = self.hip_position(hip)?;
        let visual = self.star_visual_style(hip);
        let rendered_radius = visual.map_or(2.0, |style| style.radius);
        let rendered_color = visual
            .map_or("currentColor", |style| style.color)
            .to_string();

        match self {
            Self::Normalized(catalog) => {
                let record = catalog.get(rubrum_star_catalog::StarId::Hip(hip))?;
                let proper_motion_ra_mas_per_year =
                    record.proper_motion.map(|pm| pm.pmra_mas_per_year);
                let proper_motion_dec_mas_per_year =
                    record.proper_motion.map(|pm| pm.pmdec_mas_per_year);
                Some(SvgManifestStar {
                    kind: "hip".to_string(),
                    id: hip.to_string(),
                    key: format!("hip-{hip}"),
                    label: record.name.clone().unwrap_or_else(|| format!("HIP {hip}")),
                    name: record.name.clone(),
                    ra_deg: record.position.ra_deg,
                    dec_deg: record.position.dec_deg,
                    visual_mag: record.visual_mag,
                    color_index_bv: record.color_index_bv,
                    spectral_type: record.spectral_type.clone(),
                    gaia_dr3: record.gaia_dr3,
                    tycho2: record
                        .tycho2
                        .map(|(tyc1, tyc2, tyc3)| format!("{tyc1}-{tyc2}-{tyc3}")),
                    parallax_mas: record.parallax_mas,
                    radial_velocity_km_s: record.radial_velocity_km_s,
                    proper_motion_ra_mas_per_year,
                    proper_motion_dec_mas_per_year,
                    rendered_radius,
                    rendered_color,
                })
            }
            Self::Legacy(_) => Some(SvgManifestStar {
                kind: "hip".to_string(),
                id: hip.to_string(),
                key: format!("hip-{hip}"),
                label: format!("HIP {hip}"),
                name: None,
                ra_deg: position.ra_hours * 15.0,
                dec_deg: position.dec_deg,
                visual_mag: None,
                color_index_bv: None,
                spectral_type: None,
                gaia_dr3: None,
                tycho2: None,
                parallax_mas: None,
                radial_velocity_km_s: None,
                proper_motion_ra_mas_per_year: None,
                proper_motion_dec_mas_per_year: None,
                rendered_radius,
                rendered_color,
            }),
        }
    }

    fn stats(&self) -> Option<CatalogStats> {
        match self {
            Self::Normalized(catalog) => Some(CatalogStats::from_catalog(catalog)),
            Self::Legacy(_) => None,
        }
    }

    fn bounds(&self) -> Option<stellarium_skycultures::CatalogBounds> {
        match self {
            Self::Normalized(catalog) => bounds_for_star_catalog(catalog),
            Self::Legacy(catalog) => catalog.bounds(),
        }
    }
}

impl SkyPositionCatalog for LoadedCatalog {
    fn hip_position(&self, hip: u32) -> Option<EquatorialPosition> {
        match self {
            Self::Normalized(catalog) => {
                catalog
                    .get(rubrum_star_catalog::StarId::Hip(hip))
                    .map(|record| {
                        EquatorialPosition::from_ra_deg(
                            record.position.ra_deg,
                            record.position.dec_deg,
                        )
                    })
            }
            Self::Legacy(catalog) => catalog.hip_position(hip),
        }
    }

    fn object_position(&self, object_id: &str) -> Option<EquatorialPosition> {
        match self {
            Self::Normalized(_) => None,
            Self::Legacy(catalog) => catalog.object_position(object_id),
        }
    }
}

fn load_catalog(
    path: PathBuf,
    catalog_format: CatalogFormat,
) -> Result<LoadedCatalog, Box<dyn std::error::Error>> {
    Ok(match catalog_format {
        format @ (CatalogFormat::NormalizedCsv
        | CatalogFormat::NormalizedTsv
        | CatalogFormat::NormalizedPipe) => {
            LoadedCatalog::Normalized(CsvStarCatalog::from_delimited_path(
                path,
                format
                    .normalized_delimiter()
                    .expect("matched normalized catalog format has delimiter"),
            )?)
        }
        CatalogFormat::Csv => {
            LoadedCatalog::Legacy(stellarium_skycultures::load_hip_catalog_csv(path)?)
        }
        CatalogFormat::Tsv => {
            LoadedCatalog::Legacy(stellarium_skycultures::load_hip_catalog_tsv(path)?)
        }
    })
}

fn bounds_for_star_catalog(
    catalog: &CsvStarCatalog,
) -> Option<stellarium_skycultures::CatalogBounds> {
    let bounds = CatalogStats::from_catalog(catalog).equatorial_bounds?;
    Some(bounds_to_stellarium(bounds))
}

fn bounds_to_stellarium(bounds: EquatorialBounds) -> stellarium_skycultures::CatalogBounds {
    let mut out = stellarium_skycultures::CatalogBounds::empty_accumulator();
    out.include_position(EquatorialPosition::from_ra_deg(
        bounds.min_ra_deg,
        bounds.min_dec_deg,
    ));
    out.include_position(EquatorialPosition::from_ra_deg(
        bounds.max_ra_deg,
        bounds.max_dec_deg,
    ));
    out.finish_accumulator()
        .expect("bounds converted from a non-empty bounds value")
}

fn print_catalog_summary(catalog: &LoadedCatalog) {
    if let Some(stats) = catalog.stats() {
        println!("Records: {}", stats.total_records);
        println!("HIP entries: {}", stats.hip_records);
        println!("Gaia DR3 entries: {}", stats.gaia_dr3_records);
        println!("Tycho-2 entries: {}", stats.tycho2_records);
        println!("Named entries: {}", stats.named_records);
        println!("Visual magnitude entries: {}", stats.visual_mag_records);
        println!("Proper-motion entries: {}", stats.proper_motion_records);
        println!("Object entries: 0");
    } else {
        println!("HIP entries: {}", catalog.len_hips());
        println!("Object entries: {}", catalog.len_objects());
    }
    if let Some(bounds) = catalog.bounds() {
        println!(
            "RA range: {:.6}h..{:.6}h",
            bounds.min_ra_hours, bounds.max_ra_hours
        );
        println!(
            "Dec range: {:.6}°..{:.6}°",
            bounds.min_dec_deg, bounds.max_dec_deg
        );
    } else {
        println!("RA range: n/a");
        println!("Dec range: n/a");
    }
}

#[derive(Debug, Serialize)]
struct SvgManifest {
    culture: String,
    region: Option<String>,
    figures: Vec<SvgManifestEntry>,
}

#[derive(Debug, Serialize)]
struct SvgManifestEntry {
    id: String,
    culture: String,
    slug: String,
    aliases: Vec<String>,
    canonical_name: String,
    iau: Option<String>,
    display_name: String,
    kind: String,
    svg_path: String,
    illustration: Option<SvgManifestIllustration>,
    illustrated_svg_path: Option<String>,
    line_count: usize,
    point_count: usize,
    unresolved_count: usize,
    skipped_empty_lines: usize,
    stars: Vec<SvgManifestStar>,
}

#[derive(Debug, Clone, Serialize)]
struct SvgManifestStar {
    kind: String,
    id: String,
    key: String,
    label: String,
    name: Option<String>,
    ra_deg: f64,
    dec_deg: f64,
    visual_mag: Option<f32>,
    color_index_bv: Option<f32>,
    spectral_type: Option<String>,
    gaia_dr3: Option<u64>,
    tycho2: Option<String>,
    parallax_mas: Option<f64>,
    radial_velocity_km_s: Option<f64>,
    proper_motion_ra_mas_per_year: Option<f64>,
    proper_motion_dec_mas_per_year: Option<f64>,
    rendered_radius: f64,
    rendered_color: String,
}

#[derive(Debug, Serialize)]
struct SvgManifestIllustration {
    source_path: String,
    asset_path: String,
    width: u32,
    height: u32,
    anchor_count: usize,
    transparency: Option<SvgManifestTransparency>,
    alignment: Option<SvgManifestAlignment>,
}

#[derive(Debug, Serialize)]
struct SvgManifestTransparency {
    mode: String,
    normalized: bool,
    output_format: String,
    source_had_alpha_channel: bool,
    source_used_alpha: bool,
    edge_dark_fraction: f64,
    transparent_pixels_written: usize,
    threshold: Option<u8>,
}

#[derive(Debug, Serialize)]
struct SvgManifestAlignment {
    kind: String,
    anchor_count: usize,
    rms_error_px: f64,
    matrix: [f64; 6],
}

fn render_svg_options(
    show_labels: bool,
    hide_ray_helpers: bool,
    show_star_dots: bool,
    split_ra_seam: bool,
    line_width: f64,
    label_font_size: f64,
) -> stellarium_skycultures::SvgOptions {
    stellarium_skycultures::SvgOptions {
        show_labels,
        hide_ray_helpers,
        show_star_dots,
        split_ra_seam,
        line_width,
        label_font_size,
        ..Default::default()
    }
}

#[allow(clippy::too_many_arguments)]
fn projection_options(
    width: f64,
    height: f64,
    ra_center: f64,
    ra_span: f64,
    dec_center: f64,
    dec_span: f64,
    invert_x: bool,
    invert_y: bool,
) -> stellarium_skycultures::ProjectionOptions {
    stellarium_skycultures::ProjectionOptions {
        width,
        height,
        center_ra_hours: ra_center,
        ra_span_hours: ra_span,
        center_dec_deg: dec_center,
        dec_span_deg: dec_span,
        invert_x,
        invert_y,
    }
}

fn common_name_label(common_name: &Option<CommonName>) -> Option<&str> {
    let common_name = common_name.as_ref()?;
    common_name
        .english
        .as_deref()
        .or(common_name.native.as_deref())
        .or(common_name.transliteration.as_deref())
}

fn figure_display_name(figure: &RenderFigure) -> String {
    common_name_label(&figure.common_name)
        .map(str::to_string)
        .unwrap_or_else(|| figure.id.clone())
}

fn figure_canonical_label(figure: &RenderFigure) -> String {
    figure
        .iau
        .as_deref()
        .and_then(|iau| iau_abbreviation_to_name(iau).map(str::to_string))
        .or_else(|| {
            constellation_abbreviation_from_id(&figure.id)
                .and_then(iau_abbreviation_to_name)
                .map(str::to_string)
        })
        .unwrap_or_else(|| figure_display_name(figure))
}

fn unique_aliases(primary_slug: &str, values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut aliases = Vec::new();
    for value in values {
        let alias = slugify(&value);
        if alias != primary_slug && !aliases.contains(&alias) {
            aliases.push(alias);
        }
    }
    aliases
}

fn constellation_abbreviation_from_id(id: &str) -> Option<&str> {
    let mut parts = id.split_whitespace();
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some("CON"), Some(_culture), Some(abbr), None) => Some(abbr),
        _ => None,
    }
}

fn iau_abbreviation_to_name(abbr: &str) -> Option<&'static str> {
    Some(match abbr {
        "And" => "Andromeda",
        "Ant" => "Antlia",
        "Aps" => "Apus",
        "Aql" => "Aquila",
        "Aqr" => "Aquarius",
        "Ara" => "Ara",
        "Ari" => "Aries",
        "Aur" => "Auriga",
        "Boo" => "Bootes",
        "CMa" => "Canis Major",
        "CMi" => "Canis Minor",
        "CVn" => "Canes Venatici",
        "Cae" => "Caelum",
        "Cam" => "Camelopardalis",
        "Cap" => "Capricornus",
        "Car" => "Carina",
        "Cas" => "Cassiopeia",
        "Cen" => "Centaurus",
        "Cep" => "Cepheus",
        "Cet" => "Cetus",
        "Cha" => "Chamaeleon",
        "Cir" => "Circinus",
        "Cnc" => "Cancer",
        "Col" => "Columba",
        "Com" => "Coma Berenices",
        "CrA" => "Corona Australis",
        "CrB" => "Corona Borealis",
        "Crt" => "Crater",
        "Cru" => "Crux",
        "Crv" => "Corvus",
        "Cyg" => "Cygnus",
        "Del" => "Delphinus",
        "Dor" => "Dorado",
        "Dra" => "Draco",
        "Equ" => "Equuleus",
        "Eri" => "Eridanus",
        "For" => "Fornax",
        "Gem" => "Gemini",
        "Gru" => "Grus",
        "Her" => "Hercules",
        "Hor" => "Horologium",
        "Hya" => "Hydra",
        "Hyi" => "Hydrus",
        "Ind" => "Indus",
        "LMi" => "Leo Minor",
        "Lac" => "Lacerta",
        "Leo" => "Leo",
        "Lep" => "Lepus",
        "Lib" => "Libra",
        "Lup" => "Lupus",
        "Lyn" => "Lynx",
        "Lyr" => "Lyra",
        "Men" => "Mensa",
        "Mic" => "Microscopium",
        "Mon" => "Monoceros",
        "Mus" => "Musca",
        "Nor" => "Norma",
        "Oct" => "Octans",
        "Oph" => "Ophiuchus",
        "Ori" => "Orion",
        "Pav" => "Pavo",
        "Peg" => "Pegasus",
        "Per" => "Perseus",
        "Phe" => "Phoenix",
        "Pic" => "Pictor",
        "PsA" => "Piscis Austrinus",
        "Psc" => "Pisces",
        "Pup" => "Puppis",
        "Pyx" => "Pyxis",
        "Ret" => "Reticulum",
        "Scl" => "Sculptor",
        "Sco" => "Scorpius",
        "Sct" => "Scutum",
        "Ser" => "Serpens",
        "Sex" => "Sextans",
        "Sge" => "Sagitta",
        "Sgr" => "Sagittarius",
        "Tau" => "Taurus",
        "Tel" => "Telescopium",
        "TrA" => "Triangulum Australe",
        "Tri" => "Triangulum",
        "Tuc" => "Tucana",
        "UMa" => "Ursa Major",
        "UMi" => "Ursa Minor",
        "Vel" => "Vela",
        "Vir" => "Virgo",
        "Vol" => "Volans",
        "Vul" => "Vulpecula",
        _ => return None,
    })
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;
    for ch in value.chars().flat_map(char::to_lowercase) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            previous_dash = false;
        } else if !previous_dash && !slug.is_empty() {
            slug.push('-');
            previous_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "figure".to_string()
    } else {
        slug
    }
}

fn figure_kind_str(kind: RenderFigureKind) -> &'static str {
    match kind {
        RenderFigureKind::Constellation => "constellation",
        RenderFigureKind::Asterism => "asterism",
    }
}

fn diagnostics_for_figure(rendered: &RenderSkyCulture, figure_id: &str) -> RenderDiagnostics {
    RenderDiagnostics {
        unresolved: rendered
            .diagnostics
            .unresolved
            .iter()
            .filter(|item| item.figure_id == figure_id)
            .cloned()
            .collect(),
        skipped_empty_lines: 0,
    }
}

#[derive(Debug, Clone, Copy)]
struct FitOptions {
    enabled: bool,
    padding: f64,
    min_ra_span_hours: f64,
    min_dec_span_deg: f64,
}

impl FitOptions {
    fn projection_for_figure(
        self,
        figure: &RenderFigure,
        fallback: stellarium_skycultures::ProjectionOptions,
    ) -> stellarium_skycultures::ProjectionOptions {
        if !self.enabled {
            return fallback;
        }
        let Some(bounds) = figure_position_bounds(figure) else {
            return fallback;
        };

        let padding = if self.padding.is_finite() && self.padding > 0.0 {
            self.padding
        } else {
            1.25
        };
        let min_ra_span_hours = self.min_ra_span_hours.max(0.05);
        let min_dec_span_deg = self.min_dec_span_deg.max(0.5);

        let mut ra_span_hours = (bounds.ra_span_hours * padding)
            .max(min_ra_span_hours)
            .min(24.0);
        let mut dec_span_deg = (bounds.dec_span_deg * padding)
            .max(min_dec_span_deg)
            .min(180.0);

        let target_ratio = fallback.width / fallback.height.max(1.0);
        let ra_span_deg = ra_span_hours * 15.0;
        let current_ratio = ra_span_deg / dec_span_deg.max(0.001);
        if current_ratio < target_ratio {
            ra_span_hours = (dec_span_deg * target_ratio / 15.0).min(24.0);
        } else if current_ratio > target_ratio {
            dec_span_deg = (ra_span_deg / target_ratio).min(180.0);
        }

        stellarium_skycultures::ProjectionOptions {
            center_ra_hours: bounds.center_ra_hours,
            ra_span_hours,
            center_dec_deg: bounds.center_dec_deg,
            dec_span_deg,
            ..fallback
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct FigurePositionBounds {
    center_ra_hours: f64,
    ra_span_hours: f64,
    center_dec_deg: f64,
    dec_span_deg: f64,
}

fn figure_position_bounds(figure: &RenderFigure) -> Option<FigurePositionBounds> {
    let positions = figure
        .lines
        .iter()
        .flat_map(|line| line.points.iter().map(|point| point.position))
        .chain(figure.label_positions.iter().copied())
        .collect::<Vec<_>>();
    if positions.is_empty() {
        return None;
    }

    let dec_min = positions
        .iter()
        .map(|position| position.dec_deg)
        .fold(f64::INFINITY, f64::min);
    let dec_max = positions
        .iter()
        .map(|position| position.dec_deg)
        .fold(f64::NEG_INFINITY, f64::max);

    let mut ra_values = positions
        .iter()
        .map(|position| position.ra_hours.rem_euclid(24.0))
        .collect::<Vec<_>>();
    ra_values.sort_by(f64::total_cmp);

    let mut largest_gap = -1.0;
    let mut gap_start_index = 0;
    for index in 0..ra_values.len() {
        let current = ra_values[index];
        let next = if index + 1 < ra_values.len() {
            ra_values[index + 1]
        } else {
            ra_values[0] + 24.0
        };
        let gap = next - current;
        if gap > largest_gap {
            largest_gap = gap;
            gap_start_index = index;
        }
    }

    let arc_start = if gap_start_index + 1 < ra_values.len() {
        ra_values[gap_start_index + 1]
    } else {
        ra_values[0]
    };
    let arc_end = if gap_start_index + 1 < ra_values.len() {
        ra_values[gap_start_index] + 24.0
    } else {
        ra_values[gap_start_index]
    };
    let ra_span_hours = (24.0 - largest_gap).max(0.0);
    let center_ra_hours = ((arc_start + arc_end) / 2.0).rem_euclid(24.0);

    Some(FigurePositionBounds {
        center_ra_hours,
        ra_span_hours,
        center_dec_deg: (dec_min + dec_max) / 2.0,
        dec_span_deg: (dec_max - dec_min).max(0.0),
    })
}

fn line_paths_markup(
    projected: &stellarium_skycultures::ProjectedSkyCulture,
    svg_options: stellarium_skycultures::SvgOptions,
) -> String {
    let mut out = String::new();
    for figure in &projected.figures {
        if svg_options.hide_ray_helpers && figure.is_ray_helper {
            continue;
        }
        for line in &figure.lines {
            for path in stellarium_skycultures::line_to_svg_paths(
                line,
                projected.width,
                svg_options.split_ra_seam,
            ) {
                let mut class = String::from("sky-line");
                match line.style {
                    Some(stellarium_skycultures::LineStyle::Thin) => {
                        class.push_str(" sky-line--thin")
                    }
                    Some(stellarium_skycultures::LineStyle::Bold) => {
                        class.push_str(" sky-line--bold")
                    }
                    None => {}
                }
                out.push_str(&format!(
                    "      <path class=\"{}\" d=\"{}\" stroke-width=\"{}\"/>\n",
                    class,
                    path,
                    fmt_num(svg_options.line_width)
                ));
            }
        }
    }
    out
}

fn star_key_from_source(source: &stellarium_skycultures::RenderPointSource) -> Option<String> {
    match source {
        stellarium_skycultures::RenderPointSource::Hip(hip) => Some(format!("hip-{hip}")),
        _ => None,
    }
}

fn stars_for_figure(figure: &RenderFigure, catalog: &LoadedCatalog) -> Vec<SvgManifestStar> {
    let mut seen = std::collections::BTreeSet::new();
    let mut stars = Vec::new();

    for point in figure.lines.iter().flat_map(|line| &line.points) {
        let stellarium_skycultures::RenderPointSource::Hip(hip) = &point.source else {
            continue;
        };
        if !seen.insert(*hip) {
            continue;
        }
        if let Some(star) = catalog.star_manifest_entry(*hip) {
            stars.push(star);
        }
    }

    stars.sort_by(|a, b| a.label.cmp(&b.label).then(a.key.cmp(&b.key)));
    stars
}

fn star_dots_markup(
    figure: &RenderFigure,
    projection_options: stellarium_skycultures::ProjectionOptions,
    svg_options: stellarium_skycultures::SvgOptions,
    catalog: &LoadedCatalog,
) -> String {
    if !svg_options.show_star_dots {
        return String::new();
    }

    let mut out = String::new();
    let mut seen = Vec::<(f64, f64)>::new();
    for point in figure.lines.iter().flat_map(|line| &line.points) {
        let projected =
            stellarium_skycultures::project_position(point.position, projection_options);
        if seen
            .iter()
            .any(|(x, y)| (projected.x - *x).abs() < 0.001 && (projected.y - *y).abs() < 0.001)
        {
            continue;
        }
        seen.push((projected.x, projected.y));

        let style = match &point.source {
            stellarium_skycultures::RenderPointSource::Hip(hip) => catalog.star_visual_style(*hip),
            _ => None,
        };
        let radius = style.map_or(svg_options.star_dot_radius, |style| style.radius);
        let fill = style.map_or("currentColor", |style| style.color);
        let star_attrs = match &point.source {
            stellarium_skycultures::RenderPointSource::Hip(hip) => {
                let key =
                    star_key_from_source(&point.source).unwrap_or_else(|| format!("hip-{hip}"));
                let label = match catalog.star_manifest_entry(*hip) {
                    Some(star) => star.label,
                    None => format!("HIP {hip}"),
                };
                format!(
                    " data-star-kind=\"hip\" data-star-id=\"{hip}\" data-star-key=\"{}\" aria-label=\"{}\" role=\"link\" tabindex=\"0\"",
                    escape_xml_attr(&key),
                    escape_xml_attr(&label),
                )
            }
            _ => String::new(),
        };
        out.push_str(&format!(
            "      <circle class=\"sky-star-dot\"{star_attrs} cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\"/>\n",
            fmt_num(projected.x),
            fmt_num(projected.y),
            fmt_num(radius),
            fill,
        ));
    }
    out
}

fn projected_figure_svg(
    projected: &stellarium_skycultures::ProjectedSkyCulture,
    figure: &RenderFigure,
    projection_options: stellarium_skycultures::ProjectionOptions,
    svg_options: stellarium_skycultures::SvgOptions,
    catalog: &LoadedCatalog,
) -> String {
    let line_markup = line_paths_markup(projected, svg_options);
    let star_markup = star_dots_markup(figure, projection_options, svg_options, catalog);
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w} {h}\" width=\"{w}\" height=\"{h}\" role=\"img\">\n  <style>\n    .sky-line {{ fill: none; stroke: currentColor; stroke-linecap: round; stroke-linejoin: round; opacity: 0.95; }}\n    .sky-star-dot {{ opacity: 0.95; }}\n  </style>\n  <g class=\"sky-lines-layer\">\n{line_markup}  </g>\n  <g class=\"sky-stars-layer\">\n{star_markup}  </g>\n</svg>\n",
        w = fmt_num(projected.width),
        h = fmt_num(projected.height),
    )
}

fn escape_xml_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn fmt_num(value: f64) -> String {
    let formatted = format!("{value:.3}");
    formatted
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_string()
}

fn solve_affine(
    anchor_pairs: &[([f64; 2], stellarium_skycultures::PlanePoint)],
) -> Option<[f64; 6]> {
    if anchor_pairs.len() < 3 {
        return None;
    }

    let mut normal = [[0.0_f64; 6]; 6];
    let mut rhs = [0.0_f64; 6];

    for ([x, y], target) in anchor_pairs {
        let row_u = [*x, 0.0, *y, 0.0, 1.0, 0.0];
        let row_v = [0.0, *x, 0.0, *y, 0.0, 1.0];
        accumulate_normal_equation(&mut normal, &mut rhs, row_u, target.x);
        accumulate_normal_equation(&mut normal, &mut rhs, row_v, target.y);
    }

    solve_linear_6(normal, rhs)
}

fn accumulate_normal_equation(
    normal: &mut [[f64; 6]; 6],
    rhs: &mut [f64; 6],
    row: [f64; 6],
    value: f64,
) {
    for i in 0..6 {
        rhs[i] += row[i] * value;
        for j in 0..6 {
            normal[i][j] += row[i] * row[j];
        }
    }
}

#[allow(clippy::needless_range_loop)]
fn solve_linear_6(mut a: [[f64; 6]; 6], mut b: [f64; 6]) -> Option<[f64; 6]> {
    for pivot in 0..6 {
        let mut best = pivot;
        for row in (pivot + 1)..6 {
            if a[row][pivot].abs() > a[best][pivot].abs() {
                best = row;
            }
        }
        if a[best][pivot].abs() < 1e-9 {
            return None;
        }
        if best != pivot {
            a.swap(best, pivot);
            b.swap(best, pivot);
        }

        let divisor = a[pivot][pivot];
        for col in pivot..6 {
            a[pivot][col] /= divisor;
        }
        b[pivot] /= divisor;

        for row in 0..6 {
            if row == pivot {
                continue;
            }
            let factor = a[row][pivot];
            for col in pivot..6 {
                a[row][col] -= factor * a[pivot][col];
            }
            b[row] -= factor * b[pivot];
        }
    }
    Some(b)
}

fn apply_affine(matrix: [f64; 6], point: [f64; 2]) -> stellarium_skycultures::PlanePoint {
    let [a, b, c, d, e, f] = matrix;
    let [x, y] = point;
    stellarium_skycultures::PlanePoint::new(a * x + c * y + e, b * x + d * y + f)
}

fn affine_rms_error(
    matrix: [f64; 6],
    anchor_pairs: &[([f64; 2], stellarium_skycultures::PlanePoint)],
) -> f64 {
    let sum_sq = anchor_pairs
        .iter()
        .map(|(source, target)| {
            let actual = apply_affine(matrix, *source);
            let dx = actual.x - target.x;
            let dy = actual.y - target.y;
            dx * dx + dy * dy
        })
        .sum::<f64>();
    (sum_sq / anchor_pairs.len().max(1) as f64).sqrt()
}

#[derive(Debug, Clone, Copy)]
struct TransformedImageBounds {
    min_x: f64,
    max_x: f64,
    min_y: f64,
    max_y: f64,
}

impl TransformedImageBounds {
    fn width(self) -> f64 {
        self.max_x - self.min_x
    }

    fn height(self) -> f64 {
        self.max_y - self.min_y
    }
}

fn transformed_image_bounds(matrix: [f64; 6], width: f64, height: f64) -> TransformedImageBounds {
    let corners = [[0.0, 0.0], [width, 0.0], [0.0, height], [width, height]];
    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    let mut min_y = f64::INFINITY;
    let mut max_y = f64::NEG_INFINITY;

    for corner in corners {
        let point = apply_affine(matrix, corner);
        min_x = min_x.min(point.x);
        max_x = max_x.max(point.x);
        min_y = min_y.min(point.y);
        max_y = max_y.max(point.y);
    }

    TransformedImageBounds {
        min_x,
        max_x,
        min_y,
        max_y,
    }
}

fn transformed_image_is_plausible(
    bounds: TransformedImageBounds,
    projection_options: stellarium_skycultures::ProjectionOptions,
    require_contained: bool,
) -> bool {
    let max_width = projection_options.width * 1.10;
    let max_height = projection_options.height * 1.10;
    if !bounds.width().is_finite()
        || !bounds.height().is_finite()
        || bounds.width() > max_width
        || bounds.height() > max_height
    {
        return false;
    }

    if require_contained {
        let tolerance = 2.0;
        return bounds.min_x >= -tolerance
            && bounds.max_x <= projection_options.width + tolerance
            && bounds.min_y >= -tolerance
            && bounds.max_y <= projection_options.height + tolerance;
    }

    true
}

fn aligned_illustration_image_markup(
    figure: &RenderFigure,
    projection_options: stellarium_skycultures::ProjectionOptions,
    illustration_asset_path: &str,
    require_contained: bool,
) -> Option<(String, SvgManifestAlignment)> {
    let image = figure.image.as_ref()?;
    let anchor_pairs = image
        .anchors
        .iter()
        .filter_map(|anchor| {
            figure
                .lines
                .iter()
                .flat_map(|line| &line.points)
                .find_map(|point| match point.source {
                    stellarium_skycultures::RenderPointSource::Hip(hip) if hip == anchor.hip => {
                        Some(point.position)
                    }
                    _ => None,
                })
                .map(|position| {
                    (
                        anchor.pos,
                        stellarium_skycultures::project_position(position, projection_options),
                    )
                })
        })
        .collect::<Vec<_>>();

    let matrix = solve_affine(&anchor_pairs)?;
    let image_width = f64::from(image.size[0]);
    let image_height = f64::from(image.size[1]);
    let bounds = transformed_image_bounds(matrix, image_width, image_height);
    if !transformed_image_is_plausible(bounds, projection_options, require_contained) {
        eprintln!(
            "warning: skipping implausible illustration alignment for {}: transformed bbox x={:.1}..{:.1}, y={:.1}..{:.1} ({:.1}×{:.1}px) in {:.1}×{:.1}px projection",
            figure.id,
            bounds.min_x,
            bounds.max_x,
            bounds.min_y,
            bounds.max_y,
            bounds.width(),
            bounds.height(),
            projection_options.width,
            projection_options.height,
        );
        return None;
    }
    let rms_error_px = affine_rms_error(matrix, &anchor_pairs);
    let [a, b, c, d, e, f] = matrix;
    let markup = format!(
        "    <image href=\"{href}\" width=\"{iw}\" height=\"{ih}\" transform=\"matrix({a} {b} {c} {d} {e} {f})\" opacity=\"0.78\"/>\n",
        href = escape_xml_attr(illustration_asset_path),
        iw = image.size[0],
        ih = image.size[1],
        a = fmt_num(a),
        b = fmt_num(b),
        c = fmt_num(c),
        d = fmt_num(d),
        e = fmt_num(e),
        f = fmt_num(f),
    );

    Some((
        markup,
        SvgManifestAlignment {
            kind: "affine".to_string(),
            anchor_count: anchor_pairs.len(),
            rms_error_px,
            matrix,
        },
    ))
}

fn generate_illustrated_svg(
    _culture: &stellarium_skycultures::SkyCulture,
    figure: &RenderFigure,
    projected: &stellarium_skycultures::ProjectedSkyCulture,
    projection_options: stellarium_skycultures::ProjectionOptions,
    svg_options: stellarium_skycultures::SvgOptions,
    catalog: &LoadedCatalog,
    illustration_asset_path: &str,
) -> Option<(String, SvgManifestAlignment)> {
    let (image_markup, alignment) = aligned_illustration_image_markup(
        figure,
        projection_options,
        illustration_asset_path,
        false,
    )?;
    let line_markup = line_paths_markup(projected, svg_options);
    let star_markup = star_dots_markup(figure, projection_options, svg_options, catalog);
    let svg = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w} {h}\" width=\"{w}\" height=\"{h}\" role=\"img\">\n  <style>\n    .sky-line {{ fill: none; stroke: currentColor; stroke-linecap: round; stroke-linejoin: round; opacity: 0.95; }}\n    .sky-star-dot {{ opacity: 0.95; }}\n  </style>\n  <rect width=\"100%\" height=\"100%\" fill=\"transparent\"/>\n  <g class=\"sky-illustration-layer\">\n{image_markup}  </g>\n  <g class=\"sky-lines-layer\">\n{line_markup}  </g>\n  <g class=\"sky-stars-layer\">\n{star_markup}  </g>\n</svg>\n",
        w = fmt_num(projected.width),
        h = fmt_num(projected.height),
    );

    Some((svg, alignment))
}

#[derive(Debug, Clone)]
struct OverviewFigure {
    figure: RenderFigure,
    slug: String,
    title: String,
    illustration_markup: Option<String>,
}

fn overview_figure_svg_markup(
    overview_figure: &OverviewFigure,
    projection_options: stellarium_skycultures::ProjectionOptions,
    svg_options: stellarium_skycultures::SvgOptions,
    catalog: &LoadedCatalog,
) -> String {
    let single = RenderSkyCulture {
        id: overview_figure.figure.id.clone(),
        region: None,
        figures: vec![overview_figure.figure.clone()],
        diagnostics: RenderDiagnostics::default(),
    };
    let projected = stellarium_skycultures::project_render_sky_culture(&single, projection_options);
    let Some(projected_figure) = projected.figures.first() else {
        return String::new();
    };

    let mut out = format!(
        "  <g class=\"sky-figure\" data-slug=\"{}\" data-figure-id=\"{}\" data-title=\"{}\">\n",
        escape_xml_attr(&overview_figure.slug),
        escape_xml_attr(&overview_figure.figure.id),
        escape_xml_attr(&overview_figure.title),
    );

    if let Some(markup) = &overview_figure.illustration_markup {
        out.push_str("    <g class=\"sky-illustration-layer\">\n");
        out.push_str(markup);
        out.push_str("    </g>\n");
    }

    out.push_str("    <g class=\"sky-hit-layer\">\n");
    for line in &projected_figure.lines {
        for path in stellarium_skycultures::line_to_svg_paths(
            line,
            projected.width,
            svg_options.split_ra_seam,
        ) {
            out.push_str(&format!(
                "      <path class=\"sky-hit-line\" d=\"{}\" stroke-width=\"18\"/>\n",
                path
            ));
        }
    }
    out.push_str("    </g>\n");

    out.push_str("    <g class=\"sky-lines-layer\">\n");
    for line in &projected_figure.lines {
        for path in stellarium_skycultures::line_to_svg_paths(
            line,
            projected.width,
            svg_options.split_ra_seam,
        ) {
            let mut class = String::from("sky-line");
            match line.style {
                Some(stellarium_skycultures::LineStyle::Thin) => class.push_str(" sky-line--thin"),
                Some(stellarium_skycultures::LineStyle::Bold) => class.push_str(" sky-line--bold"),
                None => {}
            }
            out.push_str(&format!(
                "      <path class=\"{}\" d=\"{}\" stroke-width=\"{}\"/>\n",
                class,
                path,
                fmt_num(svg_options.line_width)
            ));
        }
    }
    out.push_str("    </g>\n");

    out.push_str("    <g class=\"sky-stars-layer\">\n");
    out.push_str(&star_dots_markup(
        &overview_figure.figure,
        projection_options,
        svg_options,
        catalog,
    ));
    out.push_str("    </g>\n");
    out.push_str("  </g>\n");
    out
}

fn generate_overview_svg(
    culture_id: &str,
    figures: &[OverviewFigure],
    projection_options: stellarium_skycultures::ProjectionOptions,
    svg_options: stellarium_skycultures::SvgOptions,
    catalog: &LoadedCatalog,
) -> String {
    let figure_markup = figures
        .iter()
        .map(|figure| overview_figure_svg_markup(figure, projection_options, svg_options, catalog))
        .collect::<String>();

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {w} {h}\" width=\"{w}\" height=\"{h}\" role=\"img\">\n  <title>{title}</title>\n  <style>\n    .sky-line {{ fill: none; stroke: currentColor; stroke-linecap: round; stroke-linejoin: round; opacity: 0.95; }}\n    .sky-star-dot {{ opacity: 0.95; }}\n    .sky-hit-line {{ fill: none; stroke: transparent; pointer-events: stroke; }}\n    .sky-figure {{ cursor: pointer; }}\n    .sky-figure:hover .sky-line {{ stroke-width: 2.6; }}\n  </style>\n  <rect width=\"100%\" height=\"100%\" fill=\"transparent\"/>\n{figure_markup}</svg>\n",
        w = fmt_num(projection_options.width),
        h = fmt_num(projection_options.height),
        title = escape_xml_attr(culture_id),
    )
}

fn copy_figure_illustration(
    culture: &stellarium_skycultures::SkyCulture,
    figure: &RenderFigure,
    output_dir: &std::path::Path,
    slug: &str,
    enabled: bool,
    normalization: IllustrationNormalizationOptions,
) -> Result<Option<SvgManifestIllustration>, Box<dyn std::error::Error>> {
    if !enabled {
        return Ok(None);
    }
    let Some(image) = &figure.image else {
        return Ok(None);
    };

    let source_path = culture.resolve_asset(&image.file);
    if !source_path.exists() {
        eprintln!(
            "warning: illustration file not found for {}: {}",
            figure.id,
            source_path.display()
        );
        return Ok(None);
    }

    let illustrations_dir = output_dir.join("illustrations");
    fs::create_dir_all(&illustrations_dir)?;

    let asset = copy_or_normalize_illustration_asset(
        &source_path,
        &illustrations_dir,
        slug,
        normalization,
    )?;

    Ok(Some(SvgManifestIllustration {
        source_path: image.file.clone(),
        asset_path: asset.asset_path,
        width: asset.width,
        height: asset.height,
        anchor_count: image.anchors.len(),
        transparency: Some(asset.transparency),
        alignment: None,
    }))
}

#[derive(Debug, Clone, Copy)]
struct IllustrationNormalizationOptions {
    enabled: bool,
    black_threshold: u8,
    min_edge_dark_fraction: f64,
}

#[derive(Debug)]
struct CopiedIllustrationAsset {
    asset_path: String,
    width: u32,
    height: u32,
    transparency: SvgManifestTransparency,
}

fn alpha_channel_is_meaningful(image: &image::RgbaImage) -> bool {
    image.pixels().any(|pixel| pixel.0[3] < 250)
}

fn image_color_type_has_alpha(color_type: image::ColorType) -> bool {
    color_type.has_alpha()
}

fn pixel_is_dark_background(pixel: image::Rgba<u8>, threshold: u8) -> bool {
    let [r, g, b, a] = pixel.0;
    a >= 250 && r <= threshold && g <= threshold && b <= threshold
}

fn pixel_dark_score(pixel: image::Rgba<u8>) -> u8 {
    let [r, g, b, a] = pixel.0;
    if a < 250 {
        return 0;
    }
    r.max(g).max(b)
}

fn infer_dark_background_threshold(image: &image::RgbaImage, configured_threshold: u8) -> u8 {
    let width = image.width();
    let height = image.height();
    if width == 0 || height == 0 {
        return configured_threshold;
    }

    let mut edge_scores =
        Vec::with_capacity((width.saturating_mul(2) + height.saturating_mul(2)) as usize);
    for x in 0..width {
        edge_scores.push(pixel_dark_score(*image.get_pixel(x, 0)));
        edge_scores.push(pixel_dark_score(*image.get_pixel(x, height - 1)));
    }
    if height > 2 {
        for y in 1..(height - 1) {
            edge_scores.push(pixel_dark_score(*image.get_pixel(0, y)));
            edge_scores.push(pixel_dark_score(*image.get_pixel(width - 1, y)));
        }
    }

    edge_scores.sort_unstable();
    let median = edge_scores[edge_scores.len() / 2];
    let p90_index = edge_scores.len().saturating_mul(90) / 100;
    let p90 = edge_scores[p90_index.min(edge_scores.len() - 1)];

    configured_threshold
        .max(median.saturating_add(18))
        .max(p90.saturating_add(6))
}

fn edge_dark_fraction(image: &image::RgbaImage, threshold: u8) -> f64 {
    let width = image.width();
    let height = image.height();
    if width == 0 || height == 0 {
        return 0.0;
    }

    let mut dark = 0_usize;
    let mut total = 0_usize;
    for x in 0..width {
        for y in [0, height - 1] {
            total += 1;
            if pixel_is_dark_background(*image.get_pixel(x, y), threshold) {
                dark += 1;
            }
        }
    }
    if height > 2 {
        for y in 1..(height - 1) {
            for x in [0, width - 1] {
                total += 1;
                if pixel_is_dark_background(*image.get_pixel(x, y), threshold) {
                    dark += 1;
                }
            }
        }
    }

    dark as f64 / total.max(1) as f64
}

fn remove_edge_connected_dark_background(image: &mut image::RgbaImage, threshold: u8) -> usize {
    let width = image.width();
    let height = image.height();
    if width == 0 || height == 0 {
        return 0;
    }

    let len = (width as usize).saturating_mul(height as usize);
    let mut visited = vec![false; len];
    let mut queue = std::collections::VecDeque::<(u32, u32)>::new();

    let push_if_dark = |x: u32,
                        y: u32,
                        image: &image::RgbaImage,
                        visited: &mut [bool],
                        queue: &mut std::collections::VecDeque<(u32, u32)>| {
        let index = y as usize * width as usize + x as usize;
        if visited[index] {
            return;
        }
        visited[index] = true;
        if pixel_is_dark_background(*image.get_pixel(x, y), threshold) {
            queue.push_back((x, y));
        }
    };

    for x in 0..width {
        push_if_dark(x, 0, image, &mut visited, &mut queue);
        push_if_dark(x, height - 1, image, &mut visited, &mut queue);
    }
    for y in 0..height {
        push_if_dark(0, y, image, &mut visited, &mut queue);
        push_if_dark(width - 1, y, image, &mut visited, &mut queue);
    }

    let mut transparent = 0_usize;
    while let Some((x, y)) = queue.pop_front() {
        let pixel = image.get_pixel_mut(x, y);
        if pixel.0[3] != 0 {
            pixel.0[3] = 0;
            transparent += 1;
        }

        let neighbors = [
            x.checked_sub(1).map(|nx| (nx, y)),
            (x + 1 < width).then_some((x + 1, y)),
            y.checked_sub(1).map(|ny| (x, ny)),
            (y + 1 < height).then_some((x, y + 1)),
        ];
        for neighbor in neighbors.into_iter().flatten() {
            let (nx, ny) = neighbor;
            let index = ny as usize * width as usize + nx as usize;
            if visited[index] {
                continue;
            }
            visited[index] = true;
            if pixel_is_dark_background(*image.get_pixel(nx, ny), threshold) {
                queue.push_back((nx, ny));
            }
        }
    }

    transparent
}

fn copy_or_normalize_illustration_asset(
    source_path: &std::path::Path,
    illustrations_dir: &std::path::Path,
    slug: &str,
    options: IllustrationNormalizationOptions,
) -> Result<CopiedIllustrationAsset, Box<dyn std::error::Error>> {
    if !options.enabled {
        let extension = source_path
            .extension()
            .and_then(|ext| ext.to_str())
            .filter(|ext| !ext.trim().is_empty())
            .unwrap_or("webp");
        let asset_filename = format!("{slug}.{extension}");
        let relative_path = format!("illustrations/{asset_filename}");
        fs::copy(source_path, illustrations_dir.join(&asset_filename))?;
        let image = image::open(source_path)?;
        let color_type = image.color();
        let rgba = image.to_rgba8();
        return Ok(CopiedIllustrationAsset {
            asset_path: relative_path,
            width: rgba.width(),
            height: rgba.height(),
            transparency: SvgManifestTransparency {
                mode: "copied".to_string(),
                normalized: false,
                output_format: extension.to_ascii_lowercase(),
                source_had_alpha_channel: image_color_type_has_alpha(color_type),
                source_used_alpha: alpha_channel_is_meaningful(&rgba),
                edge_dark_fraction: edge_dark_fraction(&rgba, options.black_threshold),
                transparent_pixels_written: 0,
                threshold: None,
            },
        });
    }

    let image = image::open(source_path)?;
    let color_type = image.color();
    let source_had_alpha_channel = image_color_type_has_alpha(color_type);
    let mut rgba = image.to_rgba8();
    let source_used_alpha = alpha_channel_is_meaningful(&rgba);
    let dark_fraction = edge_dark_fraction(&rgba, options.black_threshold);
    let inferred_threshold = infer_dark_background_threshold(&rgba, options.black_threshold);
    let inferred_dark_fraction = edge_dark_fraction(&rgba, inferred_threshold);
    let removal_threshold = if dark_fraction >= options.min_edge_dark_fraction {
        options.black_threshold
    } else if inferred_dark_fraction >= options.min_edge_dark_fraction {
        inferred_threshold
    } else {
        options.black_threshold
    };
    let removal_dark_fraction = edge_dark_fraction(&rgba, removal_threshold);
    let mut mode = if source_used_alpha {
        "alpha".to_string()
    } else if removal_dark_fraction >= options.min_edge_dark_fraction {
        "color-key-black-edge".to_string()
    } else {
        "opaque".to_string()
    };
    let transparent_pixels_written = if source_used_alpha {
        0
    } else if removal_dark_fraction >= options.min_edge_dark_fraction {
        remove_edge_connected_dark_background(&mut rgba, removal_threshold)
    } else {
        0
    };
    if mode == "color-key-black-edge" && transparent_pixels_written == 0 {
        mode = "opaque".to_string();
    }

    let asset_filename = format!("{slug}.png");
    let relative_path = format!("illustrations/{asset_filename}");
    rgba.save_with_format(
        illustrations_dir.join(&asset_filename),
        image::ImageFormat::Png,
    )?;

    Ok(CopiedIllustrationAsset {
        asset_path: relative_path,
        width: rgba.width(),
        height: rgba.height(),
        transparency: SvgManifestTransparency {
            mode,
            normalized: true,
            output_format: "png".to_string(),
            source_had_alpha_channel,
            source_used_alpha,
            edge_dark_fraction: removal_dark_fraction,
            transparent_pixels_written,
            threshold: Some(removal_threshold),
        },
    })
}

#[allow(clippy::too_many_arguments)]
fn render_svg_all(
    culture: &stellarium_skycultures::SkyCulture,
    rendered: &RenderSkyCulture,
    output_dir: PathBuf,
    include_asterisms: bool,
    fit_options: FitOptions,
    copy_illustrations: bool,
    illustration_normalization: IllustrationNormalizationOptions,
    projection_options: stellarium_skycultures::ProjectionOptions,
    svg_options: stellarium_skycultures::SvgOptions,
    catalog: &LoadedCatalog,
) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(&output_dir)?;
    for entry in fs::read_dir(&output_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file()
            && matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("svg" | "json")
            )
        {
            fs::remove_file(path)?;
        }
    }
    let illustrations_dir = output_dir.join("illustrations");
    if copy_illustrations && illustrations_dir.exists() {
        fs::remove_dir_all(&illustrations_dir)?;
    }
    let mut entries = Vec::new();
    let mut overview_figures = Vec::new();
    let mut used_slugs = std::collections::BTreeSet::new();

    for figure in &rendered.figures {
        if figure.kind == RenderFigureKind::Asterism && !include_asterisms {
            continue;
        }
        if svg_options.hide_ray_helpers && figure.is_ray_helper {
            continue;
        }

        let display_name = figure_display_name(figure);
        let canonical_label = figure_canonical_label(figure);
        let aliases = unique_aliases(
            &slugify(&canonical_label),
            [display_name.clone(), figure.id.clone()]
                .into_iter()
                .chain(figure.iau.clone()),
        );
        let base_slug = slugify(&canonical_label);
        let mut slug = base_slug.clone();
        let mut duplicate_index = 2;
        while !used_slugs.insert(slug.clone()) {
            slug = format!("{base_slug}-{duplicate_index}");
            duplicate_index += 1;
        }

        let single = RenderSkyCulture {
            id: format!("{}:{}", rendered.id, figure.id),
            region: rendered.region.clone(),
            figures: vec![figure.clone()],
            diagnostics: diagnostics_for_figure(rendered, &figure.id),
        };
        let projected = stellarium_skycultures::project_render_sky_culture(
            &single,
            fit_options.projection_for_figure(figure, projection_options),
        );
        let projection_for_figure = fit_options.projection_for_figure(figure, projection_options);
        let svg = projected_figure_svg(
            &projected,
            figure,
            projection_for_figure,
            svg_options,
            catalog,
        );
        let svg_filename = format!("{slug}.svg");
        fs::write(output_dir.join(&svg_filename), svg)?;
        let mut illustration = copy_figure_illustration(
            culture,
            figure,
            &output_dir,
            &slug,
            copy_illustrations,
            illustration_normalization,
        )?;
        let illustrated_svg_path = if let Some(illustration_meta) = illustration.as_mut() {
            let overview_illustration_markup = aligned_illustration_image_markup(
                figure,
                projection_options,
                &illustration_meta.asset_path,
                true,
            )
            .map(|(markup, _alignment)| markup);

            overview_figures.push(OverviewFigure {
                figure: figure.clone(),
                slug: slug.clone(),
                title: canonical_label.clone(),
                illustration_markup: overview_illustration_markup,
            });

            generate_illustrated_svg(
                culture,
                figure,
                &projected,
                projection_for_figure,
                svg_options,
                catalog,
                &illustration_meta.asset_path,
            )
            .map(|(svg, alignment)| {
                illustration_meta.alignment = Some(alignment);
                let filename = format!("{slug}-illustrated.svg");
                fs::write(output_dir.join(&filename), svg).map(|_| filename)
            })
            .transpose()?
        } else {
            overview_figures.push(OverviewFigure {
                figure: figure.clone(),
                slug: slug.clone(),
                title: canonical_label.clone(),
                illustration_markup: None,
            });
            None
        };

        let line_count = figure.lines.len();
        let point_count = figure
            .lines
            .iter()
            .map(|line| line.points.len())
            .sum::<usize>();
        let unresolved_count = rendered
            .diagnostics
            .unresolved
            .iter()
            .filter(|item| item.figure_id == figure.id)
            .count();

        entries.push(SvgManifestEntry {
            id: figure.id.clone(),
            culture: culture.index.id.clone(),
            slug,
            aliases,
            canonical_name: canonical_label,
            iau: figure
                .iau
                .clone()
                .or_else(|| constellation_abbreviation_from_id(&figure.id).map(str::to_string)),
            display_name,
            kind: figure_kind_str(figure.kind).to_string(),
            svg_path: svg_filename,
            illustration,
            illustrated_svg_path,
            line_count,
            point_count,
            unresolved_count,
            skipped_empty_lines: 0,
            stars: stars_for_figure(figure, catalog),
        });
    }

    entries.sort_by(|a, b| a.display_name.cmp(&b.display_name).then(a.id.cmp(&b.id)));
    let overview_svg = generate_overview_svg(
        &culture.index.id,
        &overview_figures,
        projection_options,
        svg_options,
        catalog,
    );
    fs::write(output_dir.join("overview.svg"), overview_svg)?;

    let manifest = SvgManifest {
        culture: culture.index.id.clone(),
        region: culture.index.region.clone(),
        figures: entries,
    };
    fs::write(
        output_dir.join("index.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    println!(
        "wrote {} SVGs and index.json to {}",
        manifest.figures.len(),
        output_dir.display()
    );
    Ok(())
}

fn print_missing_refs(rendered: &stellarium_skycultures::RenderSkyCulture) {
    let mut missing_hips = std::collections::BTreeSet::new();
    let mut missing_dso = std::collections::BTreeSet::new();
    let mut other = 0_usize;

    for item in rendered.unresolved_items() {
        match &item.node {
            stellarium_skycultures::LineNode::Hip(hip) => {
                missing_hips.insert(*hip);
            }
            stellarium_skycultures::LineNode::Text(text) => {
                if let Some(dso) = text.strip_prefix("DSO:") {
                    missing_dso.insert(dso.to_string());
                } else {
                    other += 1;
                }
            }
            stellarium_skycultures::LineNode::Equatorial(_) => {
                other += 1;
            }
        }
    }

    println!(
        "Unresolved total: {}",
        rendered.diagnostics.unresolved.len()
    );
    println!("Missing HIP refs: {}", missing_hips.len());
    if !missing_hips.is_empty() {
        let values = missing_hips
            .iter()
            .map(u32::to_string)
            .collect::<Vec<_>>()
            .join(", ");
        println!("  {values}");
    }
    println!("Missing DSO refs: {}", missing_dso.len());
    if !missing_dso.is_empty() {
        let values = missing_dso.into_iter().collect::<Vec<_>>().join(", ");
        println!("  {values}");
    }
    println!("Other unresolved nodes: {other}");
    println!(
        "Skipped empty lines: {}",
        rendered.diagnostics.skipped_empty_lines
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.command {
        Command::List { repo } => {
            let cultures = load_repository_dir(repo)?;
            for culture in cultures {
                println!(
                    "{}\t{}\t{} constellations\t{} asterisms\t{} webp assets",
                    culture.index.id,
                    culture.index.region.as_deref().unwrap_or(""),
                    culture.index.constellations.len(),
                    culture.index.asterisms.len(),
                    referenced_webp_assets(&culture.index).len(),
                );
            }
        }
        Command::Summary { culture_dir } => {
            let culture = load_culture_dir(culture_dir)?;
            println!("id: {}", culture.index.id);
            println!("region: {}", culture.index.region.as_deref().unwrap_or(""));
            println!(
                "classification: {}",
                culture.index.classification.join(", ")
            );
            println!("constellations: {}", culture.index.constellations.len());
            println!("asterisms: {}", culture.index.asterisms.len());
            println!("common-name objects: {}", culture.index.common_names.len());
            println!("webp assets:");
            for asset in referenced_webp_assets(&culture.index) {
                println!("  - {}", asset);
            }
        }
        Command::CatalogSummary {
            catalog,
            catalog_format,
        } => {
            let catalog = load_catalog(catalog, catalog_format)?;
            print_catalog_summary(&catalog);
        }
        Command::MissingRefs {
            culture_dir,
            catalog,
            catalog_format,
        } => {
            let culture = load_culture_dir(culture_dir)?;
            let catalog = load_catalog(catalog, catalog_format)?;
            let rendered =
                stellarium_skycultures::build_render_sky_culture(&culture.index, &catalog);
            print_missing_refs(&rendered);
        }
        Command::RenderJson {
            culture_dir,
            catalog,
            catalog_format,
            pretty,
        } => {
            let culture = load_culture_dir(culture_dir)?;
            let catalog = load_catalog(catalog, catalog_format)?;
            let rendered =
                stellarium_skycultures::build_render_sky_culture(&culture.index, &catalog);
            if pretty {
                println!("{}", serde_json::to_string_pretty(&rendered)?);
            } else {
                println!("{}", serde_json::to_string(&rendered)?);
            }
        }
        Command::RenderSvg {
            culture_dir,
            catalog,
            output,
            catalog_format,
            width,
            height,
            ra_center,
            ra_span,
            dec_center,
            dec_span,
            invert_x,
            invert_y,
            no_labels,
            hide_ray_helpers,
            show_star_dots,
            no_seam_split,
            line_width,
            label_font_size,
        } => {
            let culture = load_culture_dir(culture_dir)?;
            let catalog = load_catalog(catalog, catalog_format)?;
            let rendered =
                stellarium_skycultures::build_render_sky_culture(&culture.index, &catalog);
            let projected = stellarium_skycultures::project_render_sky_culture(
                &rendered,
                projection_options(
                    width, height, ra_center, ra_span, dec_center, dec_span, invert_x, invert_y,
                ),
            );
            let svg = stellarium_skycultures::projected_sky_culture_to_svg(
                &projected,
                render_svg_options(
                    !no_labels,
                    hide_ray_helpers,
                    show_star_dots,
                    !no_seam_split,
                    line_width,
                    label_font_size,
                ),
            );
            if let Some(output) = output {
                std::fs::write(output, svg)?;
            } else {
                print!("{svg}");
            }
        }
        Command::RenderSvgAll {
            culture_dir,
            catalog,
            output_dir,
            catalog_format,
            include_asterisms,
            width,
            height,
            ra_center,
            ra_span,
            dec_center,
            dec_span,
            invert_x,
            invert_y,
            no_labels,
            hide_ray_helpers,
            show_star_dots,
            no_seam_split,
            fit_figures,
            fit_padding,
            min_ra_span,
            min_dec_span,
            copy_illustrations,
            normalize_illustrations,
            transparent_black_threshold,
            transparent_edge_fraction,
            line_width,
            label_font_size,
        } => {
            let culture = load_culture_dir(culture_dir)?;
            let catalog = load_catalog(catalog, catalog_format)?;
            let rendered =
                stellarium_skycultures::build_render_sky_culture(&culture.index, &catalog);
            render_svg_all(
                &culture,
                &rendered,
                output_dir,
                include_asterisms,
                FitOptions {
                    enabled: fit_figures,
                    padding: fit_padding,
                    min_ra_span_hours: min_ra_span,
                    min_dec_span_deg: min_dec_span,
                },
                copy_illustrations,
                IllustrationNormalizationOptions {
                    enabled: normalize_illustrations,
                    black_threshold: transparent_black_threshold,
                    min_edge_dark_fraction: transparent_edge_fraction.clamp(0.0, 1.0),
                },
                projection_options(
                    width, height, ra_center, ra_span, dec_center, dec_span, invert_x, invert_y,
                ),
                render_svg_options(
                    !no_labels,
                    hide_ray_helpers,
                    show_star_dots,
                    !no_seam_split,
                    line_width,
                    label_font_size,
                ),
                &catalog,
            )?;
        }
        Command::ToJson {
            culture_dir,
            pretty,
        } => {
            let culture = load_culture_dir(culture_dir)?;
            if pretty {
                println!("{}", serde_json::to_string_pretty(&culture)?);
            } else {
                println!("{}", serde_json::to_string(&culture)?);
            }
        }
    }

    Ok(())
}
