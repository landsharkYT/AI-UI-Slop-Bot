# V1 Alpha Support Matrix

Status: V1 Implementation Candidate; bounded feature surface locally verified and external release qualification pending.

| Surface | Alpha support |
| --- | --- |
| Scanner target | Native Rust binary |
| Verified development target | Linux x86-64, glibc |
| Rust toolchain used for Shot 7 | 1.96.0 |
| React source | Configured `.js`, `.jsx`, `.ts`, and `.tsx`; named functions/arrows, transparent wrapper aliases, and `Component`/`PureComponent` classes |
| Syntax mechanics | Oxc 0.142.0 |
| Tailwind | Literal utilities, selected semantic arbitrary values, configured/manifest/lockfile major-version resolution, safe static v3 theme semantics alongside visible dynamic coverage loss, and bounded recursive v4 `@theme`/`@utility` resolution across static imports |
| Inline styles | Literal and repository-local shared static object values for supported decorative categories |
| Plain CSS | Statically referenced scope-local stylesheets; source-ordered declarations within one stylesheet for simple class selectors (including later overrides), bounded unique global static custom properties, generated simple `::before`/`::after` decoration, restrained card surfaces, repeated grids, eyebrow typography, and compact control-surface traits share the signal model; duplicate semantic classes across stylesheets, scoped/ambiguous/cyclic variables, and conditional or compound signal-bearing selectors become attributable limitations or blocking loss according to blast radius |
| Configuration | `ai-ui-slop.config.jsonc`, schema 1 |
| Canonical report | JSON schema 9 and Markdown projection, including typed per-scope applicability/status, structured limitation versus blocking-loss diagnostics, canonical resource usage, exact Finding evidence, score contributions, selected component state, and score-interpretation qualification |
| Baseline | JSON schema 2 with semantic migration preview |
| Rule pack | `1.0.0-beta.9`, eleven executable rule paths, semantic structural-region safeguards, bounded static class/component registries, and bounded component/repository aggregation |
| Page Archetypes | Fourteen built-in IDs, `unknown`, and declarative custom definitions |
| Static class composition | Literals, repository-local constants, static templates, finite conditionals, standard combinators, configured class helpers, CVA defaults/static selections/scalar-or-array compound variants, and bounded variant-condition families |
| Repository discovery | Checkout/Trusted Policy `.gitignore`, built-in exclusions, internal symlink deduplication, external-symlink diagnostics, UTF-8 BOM and CRLF; `init` uses React plus browser-entrypoint evidence and bounded nested discovery |
| Source roles | Application modules by default; tests, specs, mocks, fixtures, and stories are excluded, with stories explicitly enabled through `includeStories` |
| Repository graph | Oxc-contextual component render edges, dotted-basename relative imports, inherited tsconfig path aliases, workspace package exports, symbol-aware wildcard/named barrels and cycles, literal dynamic/lazy imports, generated public `_framework` exclusion, routes, archetypes, approved primitives, component-level `path#owner` Finding impact sites, and bounded page composition through resolved local owners |
| Route adapters | Next App Router and Next Pages Router at repository root or beneath `src/`, static React Router declarations, conventional React root mounts, configured overrides; test modules never become application routes |
| Resource control | Source/file/graph/AST/reachable-state/diagnostic/output ceilings, accounted parser memory, cooperative wall time, and cancellation |
| GitHub integration | Composite Action supporting protected Trusted Policy, cooperative wall time, Linux outer-memory control, and a separately installed integrity-verified native binary |
| Release automation | Five native target jobs, digest manifest, SPDX SBOM, GitHub attestation |

Explicitly outside the V1 feature boundary:

- musl or other libc/ABI targets not named by the supported native-target matrix;
- aliased or transformed automatic-runtime factory calls and arbitrary render-transforming HOCs;
- runtime CVA selections;
- Tailwind plugin-provided semantics beyond bounded non-executing static interpretation;
- specificity, `!important`, conditional/media-query cascade resolution, CSS Modules, and CSS-in-JS analysis beyond the supported source-ordered simple-class subset.

Former V1 Implementation Blockers, implemented and locally verified in `0.13.0`:

- semantic extraction from relevant static Tailwind v3 theme values and v4 theme/custom-utility declarations;
- bounded reachability reasoning for supported container, feature-query, and repository-defined custom variants;
- symbol-aware named and wildcard re-export provenance; and
- component-level impact attribution for supported shared primitives.

Mandatory Full V1 release qualification still pending:

- completed Windows, macOS, and Linux ARM hosted smoke runs;
- committed reference-runner performance evidence;
- required fuzz and mutation evidence;
- authenticated release assets verified from an actual hosted tag workflow; and
- customer calibration or V1 release acceptance.

Inputs outside this matrix are unsupported rather than silently treated as analyzed.

The classification of blockers, release gates, post-V1 candidates, and product non-goals is normative in the [V1 scope ledger](../requirements.md#32-v1-scope-ledger). Post-V1 candidates are not roadmap commitments.
