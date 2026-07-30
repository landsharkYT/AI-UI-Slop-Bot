# AI UI Slop Bot

An evidence-first Rust analyzer for detecting repeated, context-insensitive frontend aesthetics—the “everything is a floating gradient card” family of patterns common in generated interfaces.

The repository is at the **post-Shot C V1 implementation candidate** (`0.10.0`): the locally implementable product and hardening seams are represented end to end, followed by real-repository accuracy and score-calibration passes. Customer calibration, reference-runner performance, hosted platform qualification, and authenticated release validation remain open, so it must not yet be presented as Validated MVP or Full V1.

## Run it

```sh
cargo run -- scan ./path/to/react-repository --format json --progress auto
```

The scan:

- loads independent monorepository Analysis Scopes from JSONC configuration;
- analyzes named function and arrow-function components in `.jsx` and `.tsx` with Oxc;
- resolves literal and repository-local constant Tailwind `className` strings, bounded CVA defaults/static selections/compound variants, compatible theme states, safely interpretable arbitrary values, and literal/shared static inline-style objects;
- resolves configured, manifest, and lockfile Tailwind versions plus bounded recursive v4 CSS-first sources without importing or executing target code;
- recognizes restrained plain-CSS card and layout semantics independently of class naming, and composes page evidence through uniquely resolved local render edges under explicit depth, owner, and fact ceilings;
- honors the applicable checkout or Trusted Policy `.gitignore`, deduplicates internal source symlinks, and reports external symlinks as coverage loss;
- resolves relative imports, `tsconfig` path aliases, barrels, and static lazy imports into the typed graph;
- identifies Next App Router, Next Pages Router, configured, static React Router, and conventional React root-SPA boundaries;
- evaluates the complete eleven-rule alpha pack and all fourteen built-in Page Archetypes, including framework-default and cross-role control-surface convergence;
- supports explicit House Style values/primitives, narrow Suppressions, dispositions, configured routes, safe `unknown`, and declarative custom archetypes;
- produces Finding, Component, and Repository AI Slop Scores without blending scopes;
- explains Component and Repository scores with named capped contributions, selects one compatible reachable state per Component Profile, and marks incomplete repository-score interpretations as coverage-limited;
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
ai-ui-slop scan ./repo --max-wall-time-seconds 600
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

The public seam tests cover discovery and symlinks, reachable-state class composition, shared static styles, semantic plain-CSS structures, bounded page composition, typed repository graphs and aliases, root-SPA and framework route adapters, Trusted Policy, resource ceilings, hostile filesystem and Markdown inputs, baseline migration and review metadata, parallel determinism, the eleven rule paths, Page Archetypes, command lifecycles, progress, and report artifacts.

## Validation still required

Shot 7 completes the planned implementation passes. Full V1 validation still requires the committed reference-runner workloads, hosted release smoke tests, authenticated release assets, broader fuzz/mutation evidence, and blind customer calibration. See [Shot 7](docs/shot-7.md), its [local evidence](docs/evidence/SHOT7-LOCAL.md), and the explicit [support matrix](docs/support-matrix.md). A clean result is not proof of absence when coverage diagnostics are present.

Design rationale and counterexample provenance from *Refactoring UI* are documented in the [reference traceability matrix](docs/references/refactoring-ui-traceability.md). The reference guides explanations and fixtures; it is not a machine-enforceable taste specification.

## License

Original project code and documentation are available under either the [MIT License](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE), at your option.
