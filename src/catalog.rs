use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use rubrum_star_catalog::{StarCatalog, StarId};
use serde::{Deserialize, Serialize};

/// Equatorial sky position used by the render-ready layer.
///
/// Right ascension is stored in sidereal hours (`0..24`) because Stellarium line
/// literals use `[ra_hours, dec_deg]`. Declination is stored in degrees.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EquatorialPosition {
    pub ra_hours: f64,
    pub dec_deg: f64,
}

impl EquatorialPosition {
    pub fn new(ra_hours: f64, dec_deg: f64) -> Self {
        Self { ra_hours, dec_deg }
    }

    pub fn from_ra_deg(ra_deg: f64, dec_deg: f64) -> Self {
        Self {
            ra_hours: ra_deg / 15.0,
            dec_deg,
        }
    }

    pub fn ra_deg(self) -> f64 {
        self.ra_hours * 15.0
    }
}

/// Minimal catalog interface needed to normalize Stellarium line nodes for rendering.
///
/// Implement this trait for whichever star/object catalog the app eventually uses.
/// The built-in [`HipStarCatalog`] is intentionally small and convenient for import
/// tools and tests.
pub trait SkyPositionCatalog {
    fn hip_position(&self, hip: u32) -> Option<EquatorialPosition>;

    fn object_position(&self, _object_id: &str) -> Option<EquatorialPosition> {
        None
    }
}

/// In-memory catalog keyed by HIP number and optional object IDs.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HipStarCatalog {
    hip_positions: BTreeMap<u32, EquatorialPosition>,
    object_positions: BTreeMap<String, EquatorialPosition>,
}

impl HipStarCatalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_hip(
        &mut self,
        hip: u32,
        position: EquatorialPosition,
    ) -> Option<EquatorialPosition> {
        self.hip_positions.insert(hip, position)
    }

    pub fn insert_object(
        &mut self,
        object_id: impl Into<String>,
        position: EquatorialPosition,
    ) -> Option<EquatorialPosition> {
        self.object_positions.insert(object_id.into(), position)
    }

    pub fn len_hips(&self) -> usize {
        self.hip_positions.len()
    }

    pub fn len_objects(&self) -> usize {
        self.object_positions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.hip_positions.is_empty() && self.object_positions.is_empty()
    }

    pub fn hip_positions(&self) -> impl Iterator<Item = (u32, EquatorialPosition)> + '_ {
        self.hip_positions
            .iter()
            .map(|(hip, position)| (*hip, *position))
    }

    pub fn object_positions(&self) -> impl Iterator<Item = (&str, EquatorialPosition)> + '_ {
        self.object_positions
            .iter()
            .map(|(id, position)| (id.as_str(), *position))
    }

    pub fn bounds(&self) -> Option<CatalogBounds> {
        let mut bounds = CatalogBounds::empty();
        for (_, position) in self.hip_positions() {
            bounds.include(position);
        }
        for (_, position) in self.object_positions() {
            bounds.include(position);
        }
        bounds.finish()
    }
}

/// RA/Dec bounds for loaded catalog positions.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CatalogBounds {
    pub min_ra_hours: f64,
    pub max_ra_hours: f64,
    pub min_dec_deg: f64,
    pub max_dec_deg: f64,
    count: u32,
}

impl CatalogBounds {
    fn empty() -> Self {
        Self {
            min_ra_hours: f64::INFINITY,
            max_ra_hours: f64::NEG_INFINITY,
            min_dec_deg: f64::INFINITY,
            max_dec_deg: f64::NEG_INFINITY,
            count: 0,
        }
    }

    /// Creates an empty bounds accumulator for downstream catalog adapters.
    pub fn empty_accumulator() -> Self {
        Self::empty()
    }

    fn include(&mut self, position: EquatorialPosition) {
        self.min_ra_hours = self.min_ra_hours.min(position.ra_hours);
        self.max_ra_hours = self.max_ra_hours.max(position.ra_hours);
        self.min_dec_deg = self.min_dec_deg.min(position.dec_deg);
        self.max_dec_deg = self.max_dec_deg.max(position.dec_deg);
        self.count += 1;
    }

    /// Includes a position in this bounds accumulator.
    pub fn include_position(&mut self, position: EquatorialPosition) {
        self.include(position);
    }

    fn finish(self) -> Option<Self> {
        (self.count > 0).then_some(self)
    }

    /// Finishes bounds accumulation, returning `None` if no positions were seen.
    pub fn finish_accumulator(self) -> Option<Self> {
        self.finish()
    }
}

impl SkyPositionCatalog for HipStarCatalog {
    fn hip_position(&self, hip: u32) -> Option<EquatorialPosition> {
        self.hip_positions.get(&hip).copied()
    }

    fn object_position(&self, object_id: &str) -> Option<EquatorialPosition> {
        self.object_positions.get(object_id).copied()
    }
}

/// Adapter that lets Stellarium sky-culture rendering resolve HIP line nodes from
/// any generic `rubrum-star-catalog` implementation.
///
/// Stellarium line data references HIP IDs, while downstream catalog ownership now
/// lives in `rubrum-star-catalog`. This adapter preserves the local render API's
/// hour-based [`EquatorialPosition`] type without reintroducing a separate star
/// catalog model here.
#[derive(Debug, Clone, Copy)]
pub struct RubrumStarCatalogAdapter<'a, C: StarCatalog + ?Sized> {
    catalog: &'a C,
}

impl<'a, C: StarCatalog + ?Sized> RubrumStarCatalogAdapter<'a, C> {
    pub fn new(catalog: &'a C) -> Self {
        Self { catalog }
    }

    pub fn catalog(&self) -> &'a C {
        self.catalog
    }
}

impl<C: StarCatalog + ?Sized> SkyPositionCatalog for RubrumStarCatalogAdapter<'_, C> {
    fn hip_position(&self, hip: u32) -> Option<EquatorialPosition> {
        self.catalog.get(StarId::Hip(hip)).map(|record| {
            EquatorialPosition::from_ra_deg(record.position.ra_deg, record.position.dec_deg)
        })
    }
}

impl<C: StarCatalog + ?Sized> SkyPositionCatalog for &C {
    fn hip_position(&self, hip: u32) -> Option<EquatorialPosition> {
        self.get(StarId::Hip(hip)).map(|record| {
            EquatorialPosition::from_ra_deg(record.position.ra_deg, record.position.dec_deg)
        })
    }
}

/// Errors raised while loading a small HIP catalog from delimited text.
#[derive(Debug, thiserror::Error)]
pub enum CatalogLoadError {
    #[error("failed to open/read catalog {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to parse delimited catalog {path}: {source}")]
    Csv {
        path: PathBuf,
        #[source]
        source: csv::Error,
    },

    #[error("catalog {path} is missing a required column: {column}")]
    MissingColumn { path: PathBuf, column: &'static str },

    #[error("catalog {path} row {row}: invalid {column} value {value:?}")]
    InvalidValue {
        path: PathBuf,
        row: usize,
        column: &'static str,
        value: String,
    },
}

/// Load a comma-separated HIP catalog with headers.
///
/// Required columns are flexible:
/// - HIP: `hip`, `hip_id`, `hip_number`, `hipparcos`
/// - RA: either hours (`ra_hours`, `rah`, `ra`) or degrees (`ra_deg`, `RAJ2000`, `ra_icrs`)
/// - Dec: `dec_deg`, `DEJ2000`, `dec`, `de_icrs`
///
/// Numeric values may be decimal, or simple sexagesimal values separated by `:` or whitespace.
pub fn load_hip_catalog_csv(path: impl AsRef<Path>) -> Result<HipStarCatalog, CatalogLoadError> {
    load_hip_catalog_delimited(path, b',')
}

/// Load a tab-separated HIP catalog with headers.
pub fn load_hip_catalog_tsv(path: impl AsRef<Path>) -> Result<HipStarCatalog, CatalogLoadError> {
    load_hip_catalog_delimited(path, b'\t')
}

/// Load a delimited HIP catalog with headers.
pub fn load_hip_catalog_delimited(
    path: impl AsRef<Path>,
    delimiter: u8,
) -> Result<HipStarCatalog, CatalogLoadError> {
    let path = path.as_ref();
    let mut reader = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .flexible(true)
        .from_path(path)
        .map_err(|source| CatalogLoadError::Io {
            path: path.to_path_buf(),
            source: source.into(),
        })?;

    let headers = reader
        .headers()
        .map_err(|source| CatalogLoadError::Csv {
            path: path.to_path_buf(),
            source,
        })?
        .clone();

    let columns = CatalogColumns::from_headers(path, &headers)?;
    let mut catalog = HipStarCatalog::new();

    for (row_index, record) in reader.records().enumerate() {
        let row = row_index + 2;
        let record = record.map_err(|source| CatalogLoadError::Csv {
            path: path.to_path_buf(),
            source,
        })?;

        let hip_text = get_field(&record, columns.hip).trim();
        if hip_text.is_empty() {
            continue;
        }
        let hip = parse_hip(hip_text).ok_or_else(|| CatalogLoadError::InvalidValue {
            path: path.to_path_buf(),
            row,
            column: "hip",
            value: hip_text.to_string(),
        })?;

        let ra_text = get_field(&record, columns.ra).trim();
        let dec_text = get_field(&record, columns.dec).trim();
        if ra_text.is_empty() || dec_text.is_empty() {
            continue;
        }

        let raw_ra = parse_angle(ra_text).ok_or_else(|| CatalogLoadError::InvalidValue {
            path: path.to_path_buf(),
            row,
            column: columns.ra_column_label,
            value: ra_text.to_string(),
        })?;
        let dec_deg = parse_angle(dec_text).ok_or_else(|| CatalogLoadError::InvalidValue {
            path: path.to_path_buf(),
            row,
            column: "dec_deg",
            value: dec_text.to_string(),
        })?;

        let ra_hours = match columns.ra_unit {
            RaUnit::Hours => raw_ra,
            RaUnit::Degrees => raw_ra / 15.0,
            RaUnit::Infer => {
                if raw_ra.abs() <= 24.0 {
                    raw_ra
                } else {
                    raw_ra / 15.0
                }
            }
        };

        catalog.insert_hip(hip, EquatorialPosition::new(ra_hours, dec_deg));
    }

    Ok(catalog)
}

#[derive(Debug, Clone, Copy)]
struct CatalogColumns {
    hip: usize,
    ra: usize,
    dec: usize,
    ra_unit: RaUnit,
    ra_column_label: &'static str,
}

impl CatalogColumns {
    fn from_headers(path: &Path, headers: &csv::StringRecord) -> Result<Self, CatalogLoadError> {
        let normalized: Vec<String> = headers.iter().map(normalize_header).collect();
        let hip = find_column(
            &normalized,
            &["hip", "hipid", "hipnumber", "hipparcos", "hipparcosid"],
        )
        .ok_or_else(|| CatalogLoadError::MissingColumn {
            path: path.to_path_buf(),
            column: "hip",
        })?;

        let ra_hours = find_column(
            &normalized,
            &["rahours", "rahour", "rah", "rahr", "raj2000hours"],
        );
        let ra_degrees = find_column(
            &normalized,
            &[
                "radeg",
                "radegrees",
                "raj2000",
                "raj2000deg",
                "raicrs",
                "raicrsdeg",
            ],
        );
        let ra_plain = find_column(&normalized, &["ra"]);

        let (ra, ra_unit, ra_column_label) = if let Some(index) = ra_hours {
            (index, RaUnit::Hours, "ra_hours")
        } else if let Some(index) = ra_degrees {
            (index, RaUnit::Degrees, "ra_deg")
        } else if let Some(index) = ra_plain {
            (index, RaUnit::Infer, "ra")
        } else {
            return Err(CatalogLoadError::MissingColumn {
                path: path.to_path_buf(),
                column: "ra_hours or ra_deg",
            });
        };

        let dec = find_column(
            &normalized,
            &[
                "decdeg",
                "decdegrees",
                "dej2000",
                "dej2000deg",
                "decj2000",
                "deicrs",
                "deicrsdeg",
                "dec",
                "de",
            ],
        )
        .ok_or_else(|| CatalogLoadError::MissingColumn {
            path: path.to_path_buf(),
            column: "dec_deg",
        })?;

        Ok(Self {
            hip,
            ra,
            dec,
            ra_unit,
            ra_column_label,
        })
    }
}

#[derive(Debug, Clone, Copy)]
enum RaUnit {
    Hours,
    Degrees,
    Infer,
}

fn get_field(record: &csv::StringRecord, index: usize) -> &str {
    record.get(index).unwrap_or("")
}

fn normalize_header(header: &str) -> String {
    header
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn find_column(normalized_headers: &[String], candidates: &[&str]) -> Option<usize> {
    normalized_headers
        .iter()
        .position(|header| candidates.iter().any(|candidate| header == candidate))
}

fn parse_hip(value: &str) -> Option<u32> {
    let trimmed = value.trim();
    let numeric = trimmed
        .strip_prefix("HIP")
        .or_else(|| trimmed.strip_prefix("hip"))
        .unwrap_or(trimmed)
        .trim();
    numeric.parse().ok()
}

fn parse_angle(value: &str) -> Option<f64> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    if let Ok(value) = trimmed.parse::<f64>() {
        return Some(value);
    }

    parse_sexagesimal(trimmed)
}

fn parse_sexagesimal(value: &str) -> Option<f64> {
    let parts: Vec<_> = value
        .split(|ch: char| ch == ':' || ch.is_ascii_whitespace())
        .filter(|part| !part.is_empty())
        .collect();
    if parts.is_empty() || parts.len() > 3 {
        return None;
    }

    let first = parts[0].parse::<f64>().ok()?;
    let sign = if first.is_sign_negative() { -1.0 } else { 1.0 };
    let mut value = first.abs();

    if let Some(minutes) = parts.get(1) {
        value += minutes.parse::<f64>().ok()? / 60.0;
    }
    if let Some(seconds) = parts.get(2) {
        value += seconds.parse::<f64>().ok()? / 3600.0;
    }

    Some(sign * value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_sexagesimal_angles() {
        assert_eq!(parse_angle("12:30:00"), Some(12.5));
        assert_eq!(parse_angle("-10 30 00"), Some(-10.5));
    }

    #[test]
    fn catalog_trait_resolves_inserted_hips() {
        let mut catalog = HipStarCatalog::new();
        catalog.insert_hip(123, EquatorialPosition::new(1.5, -2.0));
        assert_eq!(
            catalog.hip_position(123),
            Some(EquatorialPosition::new(1.5, -2.0))
        );
        assert_eq!(catalog.hip_position(999), None);
    }
}
