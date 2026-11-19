# Node catalog status

MaterialCraft is a clean-room implementation. This page summarizes available catalogs,
implemented corrections and remaining compatibility work.

## Inventory

| Catalog | Entries |
|---|---:|
| Main graph | 29: 25 primitives and four input/output items |
| Function graph | 85 operations |
| FX-Map graph | Quadrant, Switch, Iterate and Dot |
| Loaded package functions | 954, including 689 hidden entries |

The earlier count of 26 primitives included the removed `motion_blur` wrapper around
Directional Blur. The current count of 25 includes Dot/reroute. These corrections change
no catalog membership.

Package counts describe availability. Function-body behavior and redistribution rights
require separate verification; the 954 loaded package function bodies remain untested.

## Implemented corrections

- Missing function operands use their input types, including vectors, casts, Boolean
  operations and sampler coordinates. Partially connected vectors retain connected lanes.
- Missing textures produce appropriate zero pixels and image kinds in Channels Shuffle,
  HSL, Emboss and Dynamic Gradient.
- Fresh-node defaults are corrected for Uniform Color, function constants, loop limits
  and several image and pattern nodes.
- Grayscale image-input defaults serialize as scalars. Function loop limits use the scalar
  integer tag. Channel selector values survive signed SBS save/load.
- Filter kernels, rectangular scaling, independent tiling, channel selection and empty
  Text/SVG backgrounds are covered in [atomic node corrections](atomic-parity-improvements.md).

Regression coverage lives in the nodes, formats and eval test suites. These tests execute
MaterialCraft code and use original procedural fixtures.

## Remaining work

Full behavioral parity is unestablished. Outstanding areas include:

- Filter sampling, image boundaries and output sizing.
- Scalar/vector conversions, invalid-input handling, overflow and precision policy.
- Function execution order, shared state, loop behavior and variable scope.
- Random sequences, sampler coordinates and explicit addressing modes.
- FX-Map shape rasterization, roughness/depth weighting and assignment scope.
- Nonempty text/vector rendering, arbitrary SBS round trips and version-header compatibility.

Exhaustive parameter combinations, deeply nested processors/maps, arbitrary non-finite
payloads, dynamic resource selection and every resolution/aspect ratio remain untested.
Local safety tests cover cycles, allocation limits and execution budgets.

## Validation limits

Release suites for nodes, formats, eval and engine pass locally, including the
SBS resource-substitution test. Formatting, workspace Clippy, dependency layers
and wasm checks pass. Other targets and runtime environments remain unverified.
