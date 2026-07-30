# V1 Alpha Support Matrix

Status: Shot 1 breadth implementation; release qualification pending.

| Surface | Alpha support |
| --- | --- |
| Scanner target | Native Rust binary |
| Verified development target | Linux x86-64, glibc |
| Rust toolchain used for Shot 1 | 1.96.0 |
| React source | `.jsx` and `.tsx` named function/arrow components |
| Syntax mechanics | Oxc 0.142.0 |
| Tailwind | Literal default-state utilities shaped like Tailwind v3/v4 classes |
| Inline styles | Static object values for supported decorative categories |
| Configuration | `ai-ui-slop.config.jsonc`, schema 1 |
| Canonical report | JSON schema 1 and Markdown projection |
| Rule pack | `1.0.0-alpha.1`, nine executable rule paths |
| Page Archetypes | Fourteen built-in IDs, `unknown`, and declarative custom definitions |
| GitHub integration | Composite Action requiring a separately installed, integrity-verified native binary |

Not yet qualified in Shot 1:

- Windows and macOS native targets;
- musl or other libc/ABI targets;
- React `createElement` and automatic-runtime call extraction;
- finite dynamic class composition, CVA, or configured wrapper aliases;
- CSS import and Tailwind configuration resolution;
- complete component/import/route graphs;
- cancellation, parallel scheduling, resource ceilings, and benchmark gates;
- authenticated release manifests, SBOMs, provenance, and immutable release assets; and
- customer calibration or V1 release acceptance.

Inputs outside this matrix are unsupported rather than silently treated as analyzed.
