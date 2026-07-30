# AI UI Slop Bot

An evidence-first Rust analyzer for detecting repeated, context-insensitive frontend aesthetics—the “everything is a floating gradient card” family of patterns common in generated interfaces.

The repository is at the **Shot 2 technical candidate** (`0.2.0`): product breadth and the principal hardening seams are represented end to end, but external calibration, hosted platform qualification, and remaining release evidence are incomplete. It must not be presented as Validated MVP or Full V1.

## Run it

```sh
cargo run -- scan ./path/to/react-repository --format json --progress auto
```

The scan:

- loads independent monorepository Analysis Scopes from JSONC configuration;
- analyzes named function and arrow-function components in `.jsx` and `.tsx` with Oxc;
- resolves literal Tailwind `className` strings and static inline style objects;
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

The public seam tests cover reachable-state class composition, typed repository graphs, Trusted Policy, resource ceilings, hostile filesystem and Markdown inputs, baseline migration, parallel determinism, the nine rule paths, Page Archetypes, command lifecycles, progress, and report artifacts.

## Candidate limits

Shot 2 still leaves complete CVA/Tailwind/CSS and framework-adapter resolution, signal cancellation qualification, allocator-accounted memory enforcement, broad fuzz/mutation evidence, hosted release smoke tests, and blind customer calibration. See [Shot 2](docs/shot-2.md) and the explicit [support matrix](docs/support-matrix.md). A clean result is not proof of absence when coverage diagnostics are present.

## License

Original project code and documentation are available under either the [MIT License](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE), at your option.
