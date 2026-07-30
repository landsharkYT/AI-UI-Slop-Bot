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
