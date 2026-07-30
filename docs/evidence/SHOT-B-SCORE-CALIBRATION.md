# Shot B Score Calibration Evidence

Status: local regression and read-only dogfood pass.

Date: 2026-07-30

Shot B corrects and explains Component and Repository AI Slop Score aggregation. It does not change Finding activation thresholds.

## Defect found

The prior Repository Profile attempted to retain 70% of its strongest Component Profile with `u8` saturating multiplication. Multiplication saturated at 255 before division, so a strongest component score of 90 contributed 25 points instead of 63. This caused WebsiteHelper to report 28 low and EvacLogix 35 low despite high-severity, recurring Findings.

## Implemented score contract

Rule pack `1.0.0-beta.4` and report schema 7 provide:

- compatible reachable-state selection for Component Profiles;
- strongest Finding plus capped breadth from distinct additional Slop Patterns;
- no breadth for duplicate archetype explanations of the same rule;
- widened arithmetic;
- named Repository Profile terms for severity, prevalence, recurrence, and density;
- contribution points, caps, evidence counts, and explanations in canonical JSON, terminal output, and Markdown; and
- `qualified` versus `coverage_limited` score interpretation without silently reweighting incomplete scans.

The public score seam includes golden, boundary, and monotonic cases. Three dominant recurring components score 97; adding a fourth compatible recurring owner increases the bounded score to 100. Duplicate Template Convergence Findings for two Page Archetypes leave the component at the single-rule score of 55.

## Real-repository calibration

Fresh scans ran against the same disposable mirrors used by Shot A:

| Repository | Shot A | Shot B | Interpretation | Findings |
| --- | ---: | ---: | --- | ---: |
| ReactPDFRedactor | 0 minimal | 0 minimal | coverage-limited | 0 |
| OSM–GeoJSON to MarkdownMap | 0 minimal | 0 minimal | qualified | 0 |
| WebsiteHelper | 28 low | 76 high | coverage-limited | 10 |
| EvacLogix | 35 low | 72 high | coverage-limited | 13 |

WebsiteHelper's 76 consists of 54 severity, 2 prevalence, 15 recurrence, and 5 density points. EvacLogix's 72 consists of 46 severity, 7 prevalence, 15 recurrence, and 4 density points. The clean controls remain at zero because Shot B does not invent Findings.

Local verification completed with:

```text
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
```

These results are calibration evidence, not cross-repository rankings or customer validation.
