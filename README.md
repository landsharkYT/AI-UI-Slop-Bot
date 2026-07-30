# AI UI Slop Bot

An evidence-first Rust analyzer for detecting repeated, context-insensitive frontend aesthetics—the “everything is a floating gradient card” family of patterns common in generated interfaces.

The repository is at **Shot 1 V1 alpha**: the complete product breadth is represented end to end, but calibration, platform qualification, performance/security hardening, and customer evidence remain Shot 2 and release-gate work. It must not be presented as validated V1.

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

The public seam tests cover the nine rule paths, Page Archetype catalogs and custom definitions, monorepository scopes, House Style and Suppression policy, baselines, command lifecycles, recurrence thresholds, determinism, counterexamples, partial parsing, progress, and report artifacts.

## Prototype limits

Shot 1 intentionally leaves finite runtime class composition, CSS/config resolution, complete component/import/route graphs, cancellation, resource ceilings, deterministic parallelism, cross-platform release qualification, authenticated distribution, and blind customer calibration for Shot 2. See the explicit [support matrix](docs/support-matrix.md). A clean result is not proof of absence when coverage diagnostics are present.

## License

Original project code and documentation are available under either the [MIT License](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE), at your option.
