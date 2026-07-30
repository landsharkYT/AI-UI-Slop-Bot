# Shot 5 — Reachable Style Semantics Candidate

Status: implemented locally; release and validation gates remain explicit.

Shot 5 deepens Tailwind configuration analysis behind one crate-private module and closes the most important static reachability gaps from Shot 4 without executing repository code.

## Implemented

- `auto`, Tailwind 3, and Tailwind 4 policy selection with configured, manifest, lockfile, and CSS provenance plus conflict diagnostics;
- bounded recursive repository-local CSS configuration imports, cycle and external-import diagnostics, and auxiliary-file, aggregate-byte, and import-edge ceilings;
- canonical configuration sources, byte/edge counts, statically observed theme-value counts, and custom-variant names;
- supported static v3 radius, shadow, gradient, and spacing values map configured utilities into the same Slop Signal model as built-in utilities;
- CVA base variants, defaults, statically selected call-site values, compound variants, and conservative branching for runtime selections;
- variant-prefixed utilities remain attached to reachable-state condition families instead of being merged into the default state;
- incompatible built-in `dark` and `light` states cannot form an impossible Finding;
- the configurable reachable-state ceiling stops combinatorial expansion and records explicit Analysis Coverage loss; and
- canonical report schema 4 and scanner `0.5.0`.

## Deliberate boundaries

Only supported static radius, shadow, gradient, and spacing declarations currently define new Slop Signal mappings; other Tailwind theme and custom-utility semantics remain unresolved or informational. CVA array-valued selectors and runtime values remain conservative or unresolved. Condition simplification does not yet prove exclusivity for arbitrary data, ARIA, container, feature-query, or repository-defined custom variants. CSS Modules, CSS-in-JS, the browser cascade, and target plugins remain outside V1's current implemented surface.

React ownership breadth, complete workspace/export and primitive-impact graphs, cancellation, allocator-accounted memory, hosted platform qualification, and customer calibration remain later gates.

This is scanner `0.5.0`, not Validated MVP or Full V1.
