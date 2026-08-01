# Completion Audit: Adjacent Repositories

Status: local read-only dogfood complete; external validation still pending.

Date: 2026-07-31

Nine adjacent frontend repositories were copied to disposable directories and scanned through the public `init` and `scan` CLI lifecycle. Dependency, build-output, VCS, virtual-environment, and scanner-report directories were excluded from the copies. Original repositories were not modified.

## Rule pack `1.0.0-beta.7` results

| Repository | Outcome | Repository Profile | Findings | Rule distribution |
| --- | --- | ---: | ---: | --- |
| ReactPDFRedactor | insufficient analysis | 59, coverage-limited | 1 | Control Surface Homogenization: 1 |
| OSM–GeoJSON to MarkdownMap | insufficient analysis | 0, coverage-limited | 0 | none |
| WebsiteHelper | insufficient analysis | 68, coverage-limited | 6 | Cardification: 6 |
| EvacLogix | insufficient analysis | 69, coverage-limited | 12 | Cardification: 2; Repeated Decorative Shell: 10 |
| EventCardSite | success | 64, qualified | 6 | Control Surface Homogenization: 6 |
| CourseOfTemptation-Modded | insufficient analysis | 83, coverage-limited | 65 | Cardification: 2; Control Surface Homogenization: 7; Decoration Saturation: 11; Effect Stacking: 3; Generic Container Depth: 36; Shape Homogenization: 3; Template Convergence: 3 |
| MHGU Autopilot | insufficient analysis | 0, coverage-limited | 0 | none |
| Neural Network Card Game web | insufficient analysis | 0, coverage-limited | 0 | none |
| RAGChatWebsite | insufficient analysis | 66, coverage-limited | 2 | Cardification: 1; Decoration Saturation: 1 |

The zeros are not clean labels. OSM and MHGU expose unresolved dynamic styling or stylesheet semantics; Neural Network Card Game exposes a runtime-selected component registry the static graph cannot resolve. The heavily decorated CourseOfTemptation result is directionally credible, but its deep-wrapper and composed-page findings still need Design Authority labels rather than automatic acceptance.

## Regressions found and fixed

Five public-seam tests now cover:

- Next.js `src/app/page.tsx` adapter recognition;
- exact route owners such as `Home` participating in Template Convergence without promoting helpers in the same module;
- named identifiers referenced by `export default` retaining route ownership;
- action-control column spans not becoming bento evidence; and
- test modules not becoming application routes.

The CourseOfTemptation route inventory fell from 25 to 15 after removing test modules. MHGU now reports its `src/app/page.tsx#Home` boundary as a high-confidence Next App Router route rather than a medium-confidence generic filesystem route.

## Coverage not represented by these repositories

The set is dominated by React, TypeScript, Vite, Tailwind, and global plain CSS. It contains one Next.js application, no CSS Modules, no supported CSS-in-JS adapter, and no real-repository CVA usage. CSS Modules and CSS-in-JS remain explicitly outside the V1 support matrix; CVA behavior is covered by deterministic fixtures. These absences are not grounds for broader support claims.

## Completion judgment

The locally implementable V1 scanner is feature-complete enough for structured use-case testing. It is not release-validated. The critical path is now labeled customer calibration, committed licensed reference cases, rendered before/after review, reference-runner performance evidence, hosted platform smoke tests, and authenticated release verification—not additional heuristic breadth.
