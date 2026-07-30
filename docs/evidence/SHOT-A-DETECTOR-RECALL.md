# Shot A Detector Recall Evidence

Status: local regression and read-only dogfood pass.

Date: 2026-07-30

Shot A addresses the detector's high-precision/low-recall gap without changing the report schema. Rule pack `1.0.0-beta.3` adds:

- conventional React root-mount recognition as `root-spa:/`;
- semantic plain-CSS recognition for restrained card surfaces, repeated grids, and eyebrow typography without using class names as evidence;
- `App` as a supported root page owner; and
- page-level Cardification and Template Convergence evaluation through uniquely resolved local render owners, bounded to depth 8, 64 owners, and 512 element facts.

One early calibration build double-counted a three-column grid as both a repeated grid and a summary structure. That mislabeled ReactPDFRedactor as Template Convergence. A public workstation regression now proves that a task-oriented three-column detector grid plus two actions remains clean, and one grid contributes only one structural fact.

The canonical public-seam tests cover:

- a Vite-style root mount nested behind `StrictMode`;
- a restrained plain-CSS dashboard with arbitrary class names;
- a PDF workstation negative control with card-like work panes, a three-column detector grid, and multiple actions; and
- cross-file composition of five restrained cards through two locally rendered components.

Fresh scans ran against disposable mirrors of four adjacent repositories; their originals were not modified:

| Repository | Status | Score | Findings | Route result |
| --- | --- | ---: | --- | --- |
| ReactPDFRedactor | incomplete | 0 minimal | none | root SPA, 1/1 |
| OSM–GeoJSON to MarkdownMap | successful | 0 minimal | none | 3/3 |
| WebsiteHelper | incomplete | 28 low | 6 Cardification, 4 Template Convergence | root SPA, 1/1 |
| EvacLogix | incomplete | 35 low | 10 Repeated Decorative Shell, 2 Cardification, 1 Template Convergence | 11/11 |

The useful calibration outcome is directional: ReactPDFRedactor and the sparse OSM utility remain clean, WebsiteHelper is no longer a false negative, and EvacLogix gains page-pattern evidence without losing its existing repeated-shell evidence. Incomplete status still means a clean or low score is not proof of absence.

Local verification completed with:

```text
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

Customer calibration, score-weight calibration, and broader framework/style-system recall remain outside this pass.
