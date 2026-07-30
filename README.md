# AI UI Slop Bot

An evidence-first Rust prototype for detecting repeated, over-decorated React container treatments—the “everything is a floating gradient card” pattern common in generated interfaces.

This repository currently implements the disposable Discovery Prototype, not the full V1 product. Its only rule is the versioned [Repeated Decorative Shell contract](docs/rules/repeated-decorative-shell.md).

## Run it

```sh
cargo run -- scan ./path/to/react-repository --format json --progress auto
```

The scan:

- analyzes named function and arrow-function components in `.jsx` and `.tsx`;
- resolves literal Tailwind `className` strings and static inline style objects;
- requires the same three-or-more-signal shell signature in three distinct component owners;
- writes `.ai-ui-slop/reports/report.json` and `refactoring-brief.md`;
- sends progress and diagnostics to stderr, leaving requested JSON stdout machine-readable; and
- remains advisory, so detected Findings return exit code `0`.

Use `--progress always` to exercise the candidate progress display or `--progress never` for silent stderr. Use `--format terminal|json|markdown` to select stdout presentation.

## Verify it

```sh
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo fmt --check
```

The public seam tests cover recurrence thresholds, score determinism, counterexamples, static inline styles, partial parsing, explicit coverage loss, CLI stream separation, progress, and report artifacts.

## Prototype limits

The prototype intentionally does not resolve runtime class composition, stylesheets, component-library props, responsive/pseudo-state variants, route archetypes, House Style, configuration, baselines, enforcement, or the remaining rules in [requirements.md](requirements.md). A clean result is not proof of absence when coverage diagnostics are present.

## License

Original project code and documentation are available under either the [MIT License](LICENSE-MIT) or [Apache License 2.0](LICENSE-APACHE), at your option.
