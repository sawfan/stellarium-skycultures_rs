//! Parser and data model for the Stellarium sky-cultures repository.
//!
//! The upstream data lives at <https://github.com/Stellarium/stellarium-skycultures>.
//! This crate intentionally does not vendor that data. Instead, point the loader or
//! CLI at a local checkout of the upstream repository and it will parse each
//! culture's `index.json`, preserve illustration (`.webp`) references, and expose
//! both source-like and render-ready representations for later SVG/export work.

pub mod catalog;
pub mod projection;
pub mod render;
pub mod svg;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Deserializer, Serialize};

pub use catalog::{
    CatalogBounds, CatalogLoadError, EquatorialPosition, HipStarCatalog, RubrumStarCatalogAdapter,
    SkyPositionCatalog, load_hip_catalog_csv, load_hip_catalog_delimited, load_hip_catalog_tsv,
};
pub use projection::{
    PlanePoint, ProjectedFigure, ProjectedLabel, ProjectedLine, ProjectedSkyCulture,
    ProjectionOptions, project_position, project_render_sky_culture,
};
pub use render::{
    RenderDiagnostics, RenderFigure, RenderFigureKind, RenderLine, RenderOptions, RenderPoint,
    RenderPointSource, RenderSkyCulture, UnresolvedLineNode, build_render_figure,
    build_render_sky_culture, build_render_sky_culture_from_star_catalog,
    build_render_sky_culture_from_star_catalog_with_options, build_render_sky_culture_with_options,
};
pub use svg::{
    SvgOptions, line_to_svg_path, line_to_svg_paths, points_to_svg_path,
    projected_sky_culture_to_svg,
};

/// Result type used by this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur while loading sky-culture data.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to read {path}: {source}")]
    Read {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse JSON in {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    #[error("{path} is not a sky-culture directory (missing index.json)")]
    MissingIndex { path: PathBuf },
}

/// Top-level parsed `index.json` plus local file-system context.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkyCulture {
    /// Directory containing this culture's `index.json`.
    pub dir: PathBuf,
    /// Parsed `index.json`.
    pub index: SkyCultureIndex,
    /// Optional markdown description (`description.md`).
    pub description_markdown: Option<String>,
}

impl SkyCulture {
    /// Returns a display name suitable for lists before translation support exists.
    pub fn display_name(&self) -> &str {
        &self.index.id
    }

    /// Resolves a culture-relative asset path (for example `illustrations/aquila.webp`).
    pub fn resolve_asset<P: AsRef<Path>>(&self, relative_path: P) -> PathBuf {
        self.dir.join(relative_path)
    }

    /// Returns constellation-like records from both `constellations` and `asterisms`.
    ///
    /// Stellarium stores most cultural star figures in `constellations`, while the
    /// western data also contains an `asterisms` section for helper figures such as
    /// ray helpers. This iterator gives downstream renderers a single stream.
    pub fn figures(&self) -> impl Iterator<Item = FigureRef<'_>> {
        self.index
            .constellations
            .iter()
            .map(FigureRef::Constellation)
            .chain(self.index.asterisms.iter().map(FigureRef::Asterism))
    }
}

/// Borrowed reference to a constellation-like sky figure.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FigureRef<'a> {
    Constellation(&'a Constellation),
    Asterism(&'a Asterism),
}

impl<'a> FigureRef<'a> {
    pub fn id(self) -> &'a str {
        match self {
            Self::Constellation(item) => &item.id,
            Self::Asterism(item) => &item.id,
        }
    }

    pub fn common_name(self) -> Option<&'a CommonName> {
        match self {
            Self::Constellation(item) => item.common_name.as_ref(),
            Self::Asterism(item) => item.common_name.as_ref(),
        }
    }

    pub fn lines(self) -> &'a [Line] {
        match self {
            Self::Constellation(item) => &item.lines,
            Self::Asterism(item) => &item.lines,
        }
    }
}

/// Parsed contents of a Stellarium sky-culture `index.json` file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SkyCultureIndex {
    pub id: String,
    pub region: Option<String>,
    #[serde(default)]
    pub classification: Vec<String>,
    pub thumbnail: Option<String>,
    pub thumbnail_bscale: Option<f64>,
    pub highlight: Option<String>,
    pub illustrations_bscale: Option<f64>,
    #[serde(default)]
    pub fallback_to_international_names: bool,
    #[serde(default)]
    pub langs_use_native_names: Vec<String>,
    pub native_lang: Option<String>,
    #[serde(default)]
    pub constellations: Vec<Constellation>,
    #[serde(default)]
    pub asterisms: Vec<Asterism>,
    #[serde(default, deserialize_with = "deserialize_common_names")]
    pub common_names: BTreeMap<String, Vec<CommonName>>,
    pub edges_source: Option<String>,
    pub edges_epoch: Option<String>,
    pub edges_type: Option<String>,
    #[serde(default)]
    pub edges: Vec<String>,
    pub lunar_system: Option<serde_json::Value>,
}

/// A cultural constellation/star figure from the `constellations` section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Constellation {
    pub id: String,
    #[serde(default)]
    pub lines: Vec<Line>,
    pub image: Option<Illustration>,
    pub thumbnail: Option<String>,
    pub common_name: Option<CommonName>,
    pub iau: Option<String>,
    pub description: Option<String>,
    /// Upstream contains one historical typo. Preserve it so data is not lost.
    #[serde(rename = "descritpion")]
    pub misspelled_description: Option<String>,
    pub label_offset: Option<[f64; 2]>,
    #[serde(default)]
    pub label_positions: Vec<[f64; 2]>,
    pub single_star_radius: Option<f64>,
}

impl Constellation {
    pub fn description_text(&self) -> Option<&str> {
        self.description
            .as_deref()
            .or(self.misspelled_description.as_deref())
    }
}

/// A figure from the optional `asterisms` section.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Asterism {
    pub id: String,
    #[serde(default)]
    pub lines: Vec<Line>,
    pub common_name: Option<CommonName>,
    #[serde(default)]
    pub is_ray_helper: bool,
}

/// A polyline/path definition from Stellarium's `lines` arrays.
///
/// Items can be style markers (`thin`, `bold`), HIP star numbers, DSO object
/// references such as `DSO:M45`, or literal equatorial points `[ra_hours, dec_deg]`.
pub type Line = Vec<LineNode>;

/// A single item inside a line path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LineNode {
    Hip(u32),
    Text(String),
    Equatorial([f64; 2]),
}

impl LineNode {
    pub fn as_style(&self) -> Option<LineStyle> {
        match self {
            Self::Text(text) if text == "thin" => Some(LineStyle::Thin),
            Self::Text(text) if text == "bold" => Some(LineStyle::Bold),
            _ => None,
        }
    }

    pub fn as_object_ref(&self) -> Option<ObjectRef<'_>> {
        match self {
            Self::Hip(hip) => Some(ObjectRef::Hip(*hip)),
            Self::Text(text) => text.strip_prefix("DSO:").map(ObjectRef::DeepSkyObject),
            Self::Equatorial([ra_hours, dec_deg]) => Some(ObjectRef::Equatorial {
                ra_hours: *ra_hours,
                dec_deg: *dec_deg,
            }),
        }
    }
}

/// Line-weight markers supported by upstream `lines` arrays.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LineStyle {
    Thin,
    Bold,
}

/// Normalized object reference extracted from a [`LineNode`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObjectRef<'a> {
    Hip(u32),
    DeepSkyObject(&'a str),
    Equatorial { ra_hours: f64, dec_deg: f64 },
}

/// A Stellarium illustration entry. The `file` path is culture-directory relative.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Illustration {
    pub file: String,
    pub size: [u32; 2],
    #[serde(default)]
    pub anchors: Vec<IllustrationAnchor>,
}

/// Maps an image pixel position to a HIP star anchor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IllustrationAnchor {
    pub pos: [f64; 2],
    pub hip: u32,
}

/// Cultural/common-name metadata used for constellations and sky objects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommonName {
    pub english: Option<String>,
    pub native: Option<String>,
    pub pronounce: Option<String>,
    pub transliteration: Option<String>,
    #[serde(rename = "IPA")]
    pub ipa: Option<String>,
    #[serde(default)]
    pub references: Vec<u32>,
    pub context: Option<String>,
    pub translators_comments: Option<String>,
    pub comment: Option<String>,
    pub description: Option<String>,
    pub visible: Option<serde_json::Value>,
}

/// Load a single `index.json` file.
pub fn load_index_file(path: impl AsRef<Path>) -> Result<SkyCultureIndex> {
    let path = path.as_ref();
    let text = fs::read_to_string(path).map_err(|source| Error::Read {
        path: path.to_path_buf(),
        source,
    })?;
    serde_json::from_str(&text).map_err(|source| Error::Json {
        path: path.to_path_buf(),
        source,
    })
}

/// Load a sky-culture directory containing `index.json` and optionally `description.md`.
pub fn load_culture_dir(path: impl AsRef<Path>) -> Result<SkyCulture> {
    let dir = path.as_ref();
    let index_path = dir.join("index.json");
    if !index_path.exists() {
        return Err(Error::MissingIndex {
            path: dir.to_path_buf(),
        });
    }

    let index = load_index_file(&index_path)?;
    let description_path = dir.join("description.md");
    let description_markdown = match fs::read_to_string(&description_path) {
        Ok(text) => Some(text),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => None,
        Err(source) => {
            return Err(Error::Read {
                path: description_path,
                source,
            });
        }
    };

    Ok(SkyCulture {
        dir: dir.to_path_buf(),
        index,
        description_markdown,
    })
}

/// Load all immediate child directories of an upstream repository checkout that contain `index.json`.
pub fn load_repository_dir(path: impl AsRef<Path>) -> Result<Vec<SkyCulture>> {
    let root = path.as_ref();
    let mut cultures = Vec::new();

    let entries = fs::read_dir(root).map_err(|source| Error::Read {
        path: root.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| Error::Read {
            path: root.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() && path.join("index.json").is_file() {
            cultures.push(load_culture_dir(path)?);
        }
    }

    cultures.sort_by(|a, b| a.index.id.cmp(&b.index.id));
    Ok(cultures)
}

/// Returns all culture-relative WebP paths referenced by a culture.
pub fn referenced_webp_assets(index: &SkyCultureIndex) -> Vec<&str> {
    let mut assets = Vec::new();

    if let Some(thumbnail) = index
        .thumbnail
        .as_deref()
        .filter(|path| path.ends_with(".webp"))
    {
        assets.push(thumbnail);
    }

    for constellation in &index.constellations {
        if let Some(image) = &constellation.image
            && image.file.ends_with(".webp")
        {
            assets.push(image.file.as_str());
        }
        if let Some(thumbnail) = constellation
            .thumbnail
            .as_deref()
            .filter(|path| path.ends_with(".webp"))
        {
            assets.push(thumbnail);
        }
    }

    assets.sort_unstable();
    assets.dedup();
    assets
}

fn deserialize_common_names<'de, D>(
    deserializer: D,
) -> std::result::Result<BTreeMap<String, Vec<CommonName>>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = BTreeMap::<String, serde_json::Value>::deserialize(deserializer)?;
    let mut names = BTreeMap::new();

    for (key, value) in raw {
        if value.is_array() {
            let parsed = Vec::<CommonName>::deserialize(value).map_err(serde::de::Error::custom)?;
            names.insert(key, parsed);
        }
    }

    Ok(names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_constellation_with_illustration() {
        let json = r#"
        {
          "id": "western",
          "region": "Europe",
          "thumbnail": "illustrations/capricornus.webp",
          "constellations": [{
            "id": "CON western Aql",
            "lines": [[98036, 97649, 97278], ["thin", 95501, 93747], ["DSO:M45", [2.75, 26.75]]],
            "image": {
              "file": "illustrations/aquila.webp",
              "size": [512, 512],
              "anchors": [{"pos": [163, 232], "hip": 97649}]
            },
            "common_name": {"english": "Eagle", "native": "Aquila"},
            "iau": "Aql"
          }]
        }"#;

        let parsed: SkyCultureIndex = serde_json::from_str(json).unwrap();
        let con = &parsed.constellations[0];
        assert_eq!(con.id, "CON western Aql");
        assert_eq!(
            con.common_name.as_ref().unwrap().english.as_deref(),
            Some("Eagle")
        );
        assert_eq!(con.lines[1][0].as_style(), Some(LineStyle::Thin));
        assert_eq!(
            con.lines[2][0].as_object_ref(),
            Some(ObjectRef::DeepSkyObject("M45"))
        );
        assert_eq!(con.image.as_ref().unwrap().anchors[0].hip, 97649);
        assert_eq!(
            referenced_webp_assets(&parsed),
            vec![
                "illustrations/aquila.webp",
                "illustrations/capricornus.webp"
            ]
        );
    }

    #[test]
    fn filters_common_names_comment_entry() {
        let json = r#"
        {
          "id": "sample",
          "common_names": {
            "comment": "metadata",
            "HIP 91262": [{"english": "The Old Woman"}]
          },
          "constellations": []
        }"#;

        let parsed: SkyCultureIndex = serde_json::from_str(json).unwrap();
        assert_eq!(parsed.common_names.len(), 1);
        assert!(parsed.common_names.contains_key("HIP 91262"));
    }
}
