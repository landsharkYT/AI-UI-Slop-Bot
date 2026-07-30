# V1 Alpha Support Matrix

Status: Shot 4 implementation candidate; hosted release qualification pending.

| Surface | Alpha support |
| --- | --- |
| Scanner target | Native Rust binary |
| Verified development target | Linux x86-64, glibc |
| Rust toolchain used for Shot 4 | 1.96.0 |
| React source | `.jsx` and `.tsx` named function/arrow components |
| Syntax mechanics | Oxc 0.142.0 |
| Tailwind | Literal utilities, selected semantic arbitrary values, manifest major-version discovery, config-source discovery, and v4 CSS-first source discovery |
| Inline styles | Literal and repository-local shared static object values for supported decorative categories |
| Configuration | `ai-ui-slop.config.jsonc`, schema 1 |
| Canonical report | JSON schema 3 and Markdown projection |
| Baseline | JSON schema 2 with semantic migration preview |
| Rule pack | `1.0.0-beta.1`, nine executable rule paths |
| Page Archetypes | Fourteen built-in IDs, `unknown`, and declarative custom definitions |
| Static class composition | Literals, repository-local constants, static templates, finite conditionals, standard combinators, configured wrapper names, and bounded static `cva` variant definitions |
| Repository discovery | Checkout/Trusted Policy `.gitignore`, built-in exclusions, internal symlink deduplication, external-symlink diagnostics, UTF-8 BOM and CRLF |
| Repository graph | Relative imports, tsconfig path aliases, barrels, literal dynamic/lazy imports, rendered components, routes, archetypes, approved primitives |
| Route adapters | Next App Router, Next Pages Router, static React Router declarations, configured overrides |
| GitHub integration | Composite Action supporting protected Trusted Policy and requiring a separately installed, integrity-verified native binary |
| Release automation | Five native target jobs, digest manifest, SPDX SBOM, GitHub attestation |

Not yet qualified:

- completed Windows, macOS, and Linux ARM hosted smoke runs;
- musl or other libc/ABI targets;
- React `createElement` and automatic-runtime call extraction;
- static CVA selections, defaults, compound variants, and complete symbolic variant constraints;
- interpretation of Tailwind v3 configuration values, v4 theme/custom utility semantics, lockfile inference, explicit version override, and recursive/cyclic CSS-import resolution;
- complete workspace-export, primitive-impact, wrapper, and lazy-route graphs;
- signal cancellation and allocator-accounted memory enforcement;
- authenticated release assets verified from an actual hosted tag workflow; and
- customer calibration or V1 release acceptance.

Inputs outside this matrix are unsupported rather than silently treated as analyzed.
