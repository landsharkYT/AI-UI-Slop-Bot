# AI UI Slop Bot

An evidence-first Rust analyzer for detecting repeated, context-insensitive frontend aesthetics—the “everything is a floating gradient card” family of patterns common in generated interfaces.

The repository is at the **Shot 5 implementation candidate** (`0.5.0`): product breadth, principal hardening seams, and bounded reachable-style semantics are represented end to end, but external calibration, hosted platform qualification, and remaining implementation gates are incomplete. It must not be presented as Validated MVP or Full V1.

## Run it

```sh
cargo run -- scan ./path/to/react-repository --format json --progress auto
```

The scan:

- loads independent monorepository Analysis Scopes from JSONC configuration;
- analyzes named function and arrow-function components in `.jsx` and `.tsx` with Oxc;
- resolves literal and repository-local constant Tailwind `className` strings, bounded CVA defaults/static selections/compound variants, compatible theme states, safely interpretable arbitrary values, and literal/shared static inline-style objects;
- resolves configured, manifest, and lockfile Tailwind versions plus bounded recursive v4 CSS-first sources without importing or executing target code;
- honors the applicable checkout or Trusted Policy `.gitignore`, deduplicates internal source symlinks, and reports external symlinks as coverage loss;
- resolves relative imports, `tsconfig` path aliases, barrels, and static lazy imports into the typed graph;
- identifies Next App Router, Next Pages Router, configured, and static React Router boundaries;
- evaluates the complete nine-rule alpha pack and all fourteen built-in Page Archetypes;
- supports explicit House Style values/primitives, narrow Suppressions, dispositions, configured routes, safe `unknown`, and declarative custom archetypes;
- produces Finding, Component, and Repository AI Slop Scores without blending scopes;
- writes `.ai-ui-slop/reports/report.json` and `refactoring-brief.md`;
- sends progress and diagnostics to stderr, leaving requested JSON stdout machine-readable; and
- defaults to advisory operation, with Reviewed Baseline regression enforcement available only after explicit configuration and acceptance.

Use `--progress always` to exercise the candidate progress display or `--progress never` for silent stderr. Use `--format terminal|json|markdown` to select stdout presentation.

## Command surface

```sh
ai-ui-slop init ./repo
ai-ui-slop config validate ./repo --effective default
ai-ui-slop scan ./repo --format json
ai-ui-slop scan ./pull-request --trusted-policy-root ./protected-base --jobs 4
ai-ui-slop explain effect-stacking
ai-ui-slop baseline create ./repo
ai-ui-slop baseline accept ./repo --approver maintainer --rationale "Reviewed debt"
ai-ui-slop baseline check ./repo --format json
ai-ui-slop feedback bundle ./repo
ai-ui-slop schema report
ai-ui-slop update check
ai-ui-slop version
```

The root [composite Action](action.yml) accepts a separately installed, integrity-verified binary, uploads both report artifacts, writes a job summary, and preserves scanner exit codes.

## Verify it

```sh
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

The public seam tests cover discovery and symlinks, reachable-state class composition, shared static styles, typed repository graphs and aliases, route adapters, Trusted Policy, resource ceilings, hostile filesystem and Markdown inputs, baseline migration and review metadata, parallel determinism, the nine rule paths, Page Archetypes, command lifecycles, progress, and report artifacts.

## Candidate limits

Shot 5 still leaves full Tailwind theme/custom-utility signal interpretation, array-valued and runtime CVA selections, complete symbolic condition constraints, deeper export and primitive-impact analysis, signal cancellation qualification, allocator-accounted memory enforcement, broad fuzz/mutation evidence, hosted release smoke tests, and blind customer calibration. See [Shot 5](docs/shot-5.md) and the explicit [support matrix](docs/support-matrix.md). A clean result is not proof of absence when coverage diagnostics are present.

## License

Original project code and documentation are available under either the [MIT License](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE), at your option.
