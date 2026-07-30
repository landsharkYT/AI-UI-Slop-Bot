# EventCardSite Calibration

Status: local read-only dogfood complete; customer judgment pending.

Date: 2026-07-30

EventCardSite was copied to a disposable directory and scanned through the public CLI. Its original checkout was not modified. Test and end-to-end fixture directories were excluded from the authored-application pass.

## Initial result

| Measure | Result |
| --- | ---: |
| Repository score | 81, high, coverage-limited |
| Findings | 31 |
| Framework Default Convergence | 19 |
| Control Surface Homogenization | 6 |
| Template Convergence | 5 |
| Decoration Saturation | 1 |

Manual evidence review found systematic false positives in framework recipe coherence, page ownership, generic grid classification, dialog treatment, evidence-source selection, and static Tailwind v3 plugin-array handling.

## Rule pack `1.0.0-beta.6`

Nine public-seam regression tests reproduce those gaps and protect the corrected boundaries:

- coherent per-element framework recipes;
- intrinsic Finding severity independent of owner recurrence;
- signal-specific evidence snippets;
- helper components inside page modules remaining non-pages;
- static empty Tailwind plugin arrays resolving completely;
- dialog elevation excluded from persistent decoration saturation;
- dialog surfaces excluded from framework-default convergence; and
- functional command grids excluded from bento/feature template evidence while explicit spans remain bento evidence.

The repeated disposable-copy scan produced:

| Measure | Result |
| --- | ---: |
| Repository score | 64, high, qualified |
| Findings | 6 |
| Control Surface Homogenization | 6 |
| All other rules | 0 |
| Parse coverage | 61/61 |
| Style-resolution coverage | 1689/1844 |
| Component-graph coverage | 561/564 |
| Route coverage | 6/6 |

The six remaining findings span `App`, `CardSupportPanel`, `MassEditPanel`, `CharacterCreationPanel`, `RoleplayCardBrowser`, and `RoleplayPage`. Each reports compact typography and outlined chrome plus recurring compact/neutral surface traits across at least three roles. This is a plausible review prompt: the product repeatedly gives controls, content, and structural regions the same dense bordered treatment.

The score is not a verdict that EventCardSite is “64% slop.” It is a bounded prioritization signal derived from the strongest component and cross-owner recurrence. A product reviewer should still decide whether the dense treatment is a defect, a deliberate house style, or a mix requiring narrower component-level decisions.
