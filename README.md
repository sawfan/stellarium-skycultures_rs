# stellarium-skycultures

Rust data structures and import helpers for the upstream [Stellarium sky-cultures](https://github.com/Stellarium/stellarium-skycultures) repository.

This crate expects the upstream data checkout under `crates/stellarium-skycultures/vendor/stellarium-skycultures/` for local development examples. From inside this crate directory, that same path is `vendor/stellarium-skycultures/`. The data stays contained with the Rust importer crate and may later be converted to a crate-local git submodule.

## What is parsed

- `index.json` metadata for each culture.
- `constellations` and `asterisms` line definitions.
- HIP star references, `DSO:*` references, style markers (`thin`/`bold`), and explicit `[ra_hours, dec_deg]` points.
- Common-name metadata.
- Illustration metadata and culture-relative `.webp` asset paths.
- Optional `description.md` text when loading a culture directory.

## CLI examples

```sh
cargo run -p stellarium-skycultures -- list crates/stellarium-skycultures/vendor/stellarium-skycultures
cargo run -p stellarium-skycultures -- summary crates/stellarium-skycultures/vendor/stellarium-skycultures/western
cargo run -p stellarium-skycultures -- to-json crates/stellarium-skycultures/vendor/stellarium-skycultures/western --pretty
cargo run -p stellarium-skycultures -- render-json crates/stellarium-skycultures/vendor/stellarium-skycultures/western ./hip_catalog.csv --pretty
cargo run -p stellarium-skycultures -- catalog-summary ./hip_catalog.csv
cargo run -p stellarium-skycultures -- missing-refs crates/stellarium-skycultures/vendor/stellarium-skycultures/western ./hip_catalog.csv
cargo run -p stellarium-skycultures -- render-svg crates/stellarium-skycultures/vendor/stellarium-skycultures/western ./hip_catalog.csv --output western.svg
```

The `render-json` command expects a delimited catalog with headers. Recognized HIP columns include `hip`, `hip_id`, `hip_number`, and `hipparcos`; recognized RA columns include `ra_hours`, `ra`, `ra_deg`, `RAJ2000`, and `ra_icrs`; recognized Dec columns include `dec_deg`, `DEJ2000`, `dec`, and `de_icrs`. Use `--catalog-format tsv` for tab-separated inputs.

## Library examples

```rust
use stellarium_skycultures::{
    EquatorialPosition, HipStarCatalog, build_render_sky_culture, load_repository_dir,
    referenced_webp_assets,
};

let cultures = load_repository_dir("crates/stellarium-skycultures/vendor/stellarium-skycultures")?;
let mut catalog = HipStarCatalog::new();
catalog.insert_hip(98036, EquatorialPosition::new(20.6905, 45.2803));

for culture in &cultures {
    println!("{}", culture.index.id);
    let renderable = build_render_sky_culture(&culture.index, &catalog);
    println!(
        "  {} figures, {} unresolved nodes",
        renderable.figures.len(),
        renderable.diagnostics.unresolved.len()
    );
    for figure in culture.figures() {
        println!("  {}", figure.id());
    }
    for webp in referenced_webp_assets(&culture.index) {
        println!("  asset: {}", culture.resolve_asset(webp).display());
    }
}
# Ok::<(), Box<dyn std::error::Error>>(())
```

## Render-ready layer

`build_render_sky_culture(...)` converts Stellarium line nodes into projection-agnostic `RenderPoint`s:

- HIP numbers resolve through the `SkyPositionCatalog` trait.
- `DSO:*` object references resolve through the optional `object_position(...)` catalog method.
- Literal `[ra_hours, dec_deg]` points are already positioned and pass through directly.
- Style markers (`thin`/`bold`) are preserved on `RenderLine::style`.
- Unresolved nodes are non-fatal and collected in `RenderDiagnostics`.

This keeps catalog resolution separate from projection. An SVG renderer can take `RenderSkyCulture`, choose a projection/crop, then map each `EquatorialPosition` to SVG coordinates.

## Projection and SVG

`project_render_sky_culture(...)` converts `RenderSkyCulture` to `ProjectedSkyCulture` with a simple whole-sky equirectangular projection by default. `projected_sky_culture_to_svg(...)` then serializes projected lines and labels into a standalone SVG document.

The CLI exposes this via `render-svg`:

```sh
cargo run -p stellarium-skycultures -- render-svg crates/stellarium-skycultures/vendor/stellarium-skycultures/western ./hip_catalog.csv \
  --width 1200 --height 600 \
  --ra-center 12 --ra-span 24 \
  --dec-center 0 --dec-span 180 \
  --hide-ray-helpers --show-star-dots \
  --output western.svg
```

Useful diagnostics before rendering:

```sh
cargo run -p stellarium-skycultures -- catalog-summary ./hip_catalog.csv
cargo run -p stellarium-skycultures -- missing-refs crates/stellarium-skycultures/vendor/stellarium-skycultures/western ./hip_catalog.csv
```

SVG export splits large horizontal jumps by default to avoid drawing long RA-seam-crossing lines. Use `--no-seam-split` to disable that behavior.

## Future rendering notes

The current SVG path is intentionally simple and debuggable. Next improvements could include richer projection choices and illustration fitting. Illustration rendering can use each `Illustration`'s `anchors` to fit the culture-relative WebP image to projected star positions.
