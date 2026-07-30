# Shot 4 — Static Style Adapter Candidate

Status: implemented locally; release and validation gates remain explicit.

Shot 4 closes a bounded part of the Full V1 style-analysis gap without executing repository code or target configuration.

## Implemented

- static `cva` base/variant definitions become separate, bounded reachable states rather than one impossible merged class list;
- semantic interpretation of supported Tailwind arbitrary radii, padding, gradient backgrounds, and large shadows;
- Tailwind major-version discovery from `package.json`;
- discovery of Tailwind v3 configuration filenames and v4 `@theme`, `@source`, `@utility`, `@custom-variant`, and Tailwind import sources;
- repository-local static CSS import checks, with missing imports recorded in the canonical adapter report, diagnostics, and scope completeness; and
- a schema-visible `styleAdapter` record containing detected version, sources, and unresolved constructs.

## Deliberate boundaries

This shot does not interpret Tailwind theme/configuration values, execute plugins, resolve lockfiles, provide an explicit version override, recursively compose CSS configuration, or model CVA defaults, call-site selections, compound variants, and full condition constraints. Unrecognized arbitrary syntax receives no guessed score. These gaps remain release gates in `requirements.md`.

This is scanner `0.4.0`, not Validated MVP or Full V1.
