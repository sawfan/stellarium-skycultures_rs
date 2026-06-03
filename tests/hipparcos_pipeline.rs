use rubrum_star_catalog::CsvStarCatalog;
use rubrum_star_catalog_hipparcos::read_hipparcos_catalog_reader;
use stellarium_skycultures::{
    Asterism, LineNode, SkyCultureIndex, build_render_sky_culture_from_star_catalog,
};

fn minimal_index() -> SkyCultureIndex {
    let mut index = SkyCultureIndex {
        id: "pipeline-test".to_string(),
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
    index.asterisms.push(Asterism {
        id: "hip-line".to_string(),
        common_name: None,
        lines: vec![vec![LineNode::Hip(1), LineNode::Hip(2)]],
        is_ray_helper: false,
    });
    index
}

#[test]
fn hipparcos_import_to_normalized_catalog_to_skyculture_render() {
    let records = read_hipparcos_catalog_reader(
        "hip,ra_deg,dec_deg,visual_mag,name\n\
         1,0.0,10.0,1.0,One\n\
         2,30.0,-5.0,2.0,Two\n"
            .as_bytes(),
    )
    .unwrap();

    let normalized_bytes =
        rubrum_star_catalog::write_normalized_catalog_writer(Vec::new(), &records).unwrap();
    let catalog = CsvStarCatalog::from_reader(normalized_bytes.as_slice()).unwrap();
    let rendered = build_render_sky_culture_from_star_catalog(&minimal_index(), &catalog);

    assert!(rendered.diagnostics.is_clean());
    let points = &rendered.figures[0].lines[0].points;
    assert_eq!(points.len(), 2);
    assert_eq!(points[0].position.ra_hours, 0.0);
    assert_eq!(points[0].position.dec_deg, 10.0);
    assert_eq!(points[1].position.ra_hours, 2.0);
    assert_eq!(points[1].position.dec_deg, -5.0);
}
