# Real-Repository Hardening Evidence

Status: local regression and read-only dogfood pass.

Date: 2026-07-29

Three repositories adjacent to this repository were copied into temporary directories and scanned through the public `init` and `scan` lifecycle. Their originals were not modified.

The initial pass exposed false wrapper diagnostics for ordinary callback values, TypeScript generic names misread as JSX render edges, external icon components counted as unresolved local components, non-UI workspace packages promoted to Analysis Scopes, an unsupported plain-CSS seam, and a same-role settings-menu Rhythm Homogenization false positive.

Seven public-seam regression tests now cover:

- browser-app discovery nested beyond conventional workspace layouts;
- exclusion of server and contracts packages from generated frontend scopes;
- separation of ordinary callbacks and TypeScript type arguments from React component evidence;
- exclusion of external component imports from local component-graph coverage;
- bounded signal extraction from statically referenced simple plain-CSS class selectors;
- exclusion of unreferenced stylesheets;
- explicit coverage loss, without impossible default-state Findings, for conditional or compound signal-bearing CSS; and
- protection of consistent same-role settings rows from Rhythm Homogenization.

The repeated dogfood pass reduced:

- ReactPDFRedactor from 90 diagnostics to three meaningful aggregate diagnostics, no Findings, and 71/71 component-graph coverage;
- OSM–GeoJSON to MarkdownMap from 36 diagnostics and one false Finding to six diagnostics, no Findings, and 108/110 component-graph coverage; and
- WebsiteHelper from 99 diagnostics across a whole-repository default scan to two meaningful aggregate diagnostics, no Findings, and 58/58 component-graph coverage in the single detected browser application.

The zero-Finding outcomes are not customer-validation labels. ReactPDFRedactor and WebsiteHelper remain incomplete because dynamic class expressions and unsupported conditional/compound CSS are visible coverage losses.

The release build also completed the deterministic 500-file, 500,500-line benchmark in 148 ms with a 14,460 KiB peak RSS. The 20-pair progress trial measured a -0.48% median overhead for progress enabled versus disabled and passed the 2% median gate. These local measurements are regression evidence, not cross-machine performance guarantees.

## Plain-CSS and graph follow-up

Date: 2026-07-30

A second read-only dogfood pass added EvacLogix from the adjacent `385/EvacLogix` repository. Fresh initialization correctly selected its `web` Vite application and excluded the `EvacLogixBuild` artifact directory.

EvacLogix exposed a false negative in the first plain-CSS adapter: `.page-card` combined tokenized large-shadow and padding declarations with a gradient `::before` treatment across ten distinct component owners, but raw `var(...)` values and the pseudo-element were not composed into the base-class evidence. A controlled temporary-copy probe showed that directly exposing those same values produced the expected recurrence cluster.

Rule pack `1.0.0-beta.2` now:

- resolves unique global static plain-CSS custom properties across statically referenced stylesheets with recursion-depth, cycle, fallback, ambiguity, and expanded-value bounds;
- composes only generated simple `.class::before` and `.class::after` signal declarations into the simple base class, while keeping stateful, descendant, compound, and conditional selectors unresolved;
- probes relative module extensions without destroying dotted basenames such as `unity.types.ts`; and
- excludes generated `_framework` runtime trees beneath `public` from the authored application module graph.

The repeated pass produced:

- ReactPDFRedactor: no Findings, three diagnostics, and 71/71 component-graph coverage;
- OSM–GeoJSON to MarkdownMap: no Findings, five diagnostics, and 89/90 component-graph coverage after generated .NET runtime removal;
- WebsiteHelper: no Findings, two diagnostics, and 58/58 component-graph coverage; and
- EvacLogix: ten `repeated-decorative-shell` Findings at score 52 across ten owners, seven diagnostics, and 203/203 component-graph coverage.

The remaining OSM graph miss is an authored worker runtime expression rather than generated framework noise. The original repositories were not modified; initialization and scans ran on disposable copies.

After the follow-up changes, the deterministic 500-file, 500,500-line release benchmark completed in 147 ms with a 14,400 KiB peak RSS. The 20-pair progress trial measured a -0.04% median overhead and passed the 2% median gate.

## Shot A detector recall follow-up

Date: 2026-07-30

Rule pack `1.0.0-beta.3` recognizes conventional root SPAs, restrained plain-CSS cards and structures, and bounded page composition through uniquely resolved local component owners. The four-repository pass now keeps ReactPDFRedactor and the OSM utility at zero Findings, raises WebsiteHelper from a false-negative zero to ten page/view Findings, and adds three page-pattern Findings to EvacLogix alongside its ten existing repeated-shell Findings.

The first Shot A calibration caught and removed structural double-counting for three-column work grids. Full fixtures, bounds, coverage, and result counts are recorded in [SHOT-A-DETECTOR-RECALL.md](SHOT-A-DETECTOR-RECALL.md).

## Shot B score calibration follow-up

Date: 2026-07-30

Rule pack `1.0.0-beta.4` fixes saturating pre-division arithmetic and exposes bounded severity, prevalence, recurrence, and density contributions. ReactPDFRedactor and the OSM utility remain at zero. WebsiteHelper moves from 28 low to 76 high, while EvacLogix moves from 35 low to 72 high. Incomplete scans are explicitly marked `coverage_limited`.

The formula, golden boundaries, and exact contribution breakdowns are recorded in [SHOT-B-SCORE-CALIBRATION.md](SHOT-B-SCORE-CALIBRATION.md).
