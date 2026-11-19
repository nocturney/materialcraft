# Atomic node corrections

These changes correct filtering, channel selection and empty-content compositing in
MaterialCraft. Catalog membership and bundled SBS assets are unchanged.

## Behavior changes

- Blur uses a separable box filter with fractional endpoint coverage and bounded running
  sums. Non-finite samples stop affecting a window after they leave it.
- Directional Blur uses the full radius, weighted endpoints and image-space rotation.
  Its sampling budget is bounded for large strengths.
- Sharpen uses a four-neighbor Laplacian, processes alpha, clamps negative lobes and retains
  HDR output. Explicit Float32 operation order preserves cancellation behavior on ramps.
- All three filters scale from the shorter image dimension and honor horizontal and
  vertical tiling independently. Tiling changes invalidate the cook cache.
- Channels Shuffle supplies a grayscale scalar through all ordinary channel selections,
  including alpha. Color inputs retain their alpha. Signed and oversized saved selectors
  remain intact until input kinds determine channel selection.
- Empty Text/SVG content preserves background pixels, including hidden RGB under zero
  alpha. Incompatible background kinds produce empty pixels; matching gray backgrounds
  retain their scalar values.

Channels Shuffle now offers eight input-channel choices. Graphs using the former
Black/White choices follow the corrected selector behavior. Use a Uniform Color input
for constant channels.

## Regression coverage

Local tests cover fractional filter footprints, directional extent, sharpen gain and
Float32 cancellation, rectangular images, independent tiling and cache invalidation,
gray/color channel selection, signed-selector round trips and transparent backgrounds:

- [Filter and channel tests](../crates/nodes/tests/atomic_filter_parity.rs)
- [Tiling and cache tests](../crates/eval/tests/filter_tiling.rs)
- [SBS round-trip tests](../crates/formats/src/sbs/tests.rs)

Save/load fixtures use explicit blur strengths and compatible background kinds so their
binding and connection-preservation assertions retain their purpose.

## Remaining limits

Full numerical compatibility remains unestablished. Open work includes oblique sampling,
floating-point rounding and quantization, mixed input/output resolutions, non-default
Sharpen alpha behavior and nonempty Text/SVG rasterization. Processor and map execution
limits are summarized in the [catalog status](node-catalog-parity.md).

## Validation

Recorded release tests for nodes, formats, eval and engine pass, including
`every_node_cooks` and `panic_hunt`. Formatting,
workspace Clippy with `--all-targets -- -D warnings`, dependency layers and all ten wasm
checks pass.

These checks cover the listed crates and targets; other build and runtime environments
have not been validated by this local run.

The optional `filter_timing` example benchmarks MaterialCraft's Blur, Directional Blur
and Sharpen on generated images at 512 and 1024 pixels. It uses no external application:

```sh
cargo run -p materialcraft-nodes --release --example filter_timing
```
