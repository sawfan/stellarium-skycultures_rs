use rubrum_star_catalog::{
    EquatorialPosition as CatalogPosition, InMemoryStarCatalog, StarId, StarRecord,
};
use stellarium_skycultures::{
    LineNode, RubrumStarCatalogAdapter, SkyCultureIndex, SkyPositionCatalog,
    build_render_sky_culture_from_star_catalog,
};

fn record(hip: u32, ra_deg: f64, dec_deg: f64) -> StarRecord {
    StarRecord {
        id: StarId::Hip(hip),
        hip: Some(hip),
        gaia_dr3: None,
        tycho2: None,
        position: CatalogPosition::new(ra_deg, dec_deg),
        epoch_julian_year: 2000.0,
        proper_motion: None,
        parallax_mas: None,
        radial_velocity_km_s: None,
        visual_mag: None,
        color_index_bv: None,
        spectral_type: None,
        name: None,
    }
}

#[test]
fn adapter_resolves_hip_positions_from_generic_catalog() {
    let catalog = InMemoryStarCatalog::new(vec![record(123, 30.0, -5.0)]).unwrap();
    let adapter = RubrumStarCatalogAdapter::new(&catalog);

    let position = adapter.hip_position(123).unwrap();
    assert_eq!(position.ra_hours, 2.0);
    assert_eq!(position.dec_deg, -5.0);
    assert_eq!(adapter.hip_position(999), None);
}

#[test]
fn generic_catalog_render_helper_resolves_hip_lines() {
    let catalog = InMemoryStarCatalog::new(vec![record(123, 30.0, -5.0)]).unwrap();
    let mut index = SkyCultureIndex {
        id: "test".to_string(),
        region: None,
        classification: Vec::new(),
        thumbnail: None,
        thumbnail_bscale: None,
        highlight: None,
        illustrations_bscale: None,
        fallback_to_international_names: false,
        langs_use_native_names: Vec::new(),
        native_lang: None,
        constellations: Vec::new(),
        asterisms: Vec::new(),
        common_names: Default::default(),
        edges_source: None,
        edges_epoch: None,
        edges_type: None,
        edges: Vec::new(),
        lunar_system: None,
    };
    index.asterisms.push(stellarium_skycultures::Asterism {
        id: "line".to_string(),
        common_name: None,
        lines: vec![vec![LineNode::Hip(123)]],
        is_ray_helper: false,
    });

    let rendered = build_render_sky_culture_from_star_catalog(&index, &catalog);
    assert!(rendered.diagnostics.is_clean());
    assert_eq!(
        rendered.figures[0].lines[0].points[0].position.ra_hours,
        2.0
    );
}
