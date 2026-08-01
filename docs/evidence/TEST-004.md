# TEST-004 mutation qualification

Status: local pass, requalified for scanner `0.14.0` on 2026-08-01.

The rule and policy mutation gate was executed with cargo-mutants 27.0.0 against frozen candidate `b034b33bcad3cd116a029f3df012f7e82cf9cb03`, covering `src/app.rs`, `src/baseline.rs`, `src/policy.rs`, and `src/style.rs`:

```sh
TMPDIR="$PWD/target/mutants-tmp-v014-exit-2" cargo mutants --no-shuffle --jobs 4 \
  --timeout-multiplier 3 --minimum-test-timeout 20 \
  --output target/qualification/mutants-v014-exit-2 \
  --file src/app.rs --file src/baseline.rs --file src/policy.rs --file src/style.rs
scripts/mutation-score.py target/qualification/mutants-v014-exit-2 \
  --minimum 80 --output target/qualification/mutation-score-v014-exit.json
```

The complete, non-iterative run tested 976 mutants: 762 caught, 179 missed, 5 timed out, and 30 unviable. Timeouts count as uncaught viable mutants, producing 762 / 946 = **80.5497%**, above the provisional 80% minimum. The evidence normalizer verified the cargo-mutants end timestamp and exact tested/total count; it rejects incomplete and iterative output. The normalized evidence is committed at [artifacts/mutation-score-v014-exit.json](artifacts/mutation-score-v014-exit.json). The ignored raw `outcomes.json` had SHA-256 `f4dfe3d3648cba3192e075f63c43d252dcf4f5d309fc2e37a09f8c3bbbad72e3`; the committed normalized artifact has SHA-256 `93d7407337c0303375af89653353afd45bd7037c6eac0670477cf0df35e64dfb`.

An initial `0.14.0` pass scored 756 / 946 = 79.9154%, below the gate. It was not rounded up. A public-seam regression was added for exact and first-excess aggregate auxiliary-byte ceilings across CSS imports. A focused diagnostic caught all ten relevant mutations, and the entire non-iterative gate was then rerun from scratch. The six-mutant net improvement in the complete run is attributable to root/import byte-limit comparisons and accounting updates; production sources were unchanged between the two passes.

Survivor review found meaningful gaps in coverage thresholds, cancellation classification, baseline compatibility and migration behavior, ordered score bands, calendar validation, Tailwind detection, exact auxiliary-file ceilings, CSS value resolution, route discovery, aggregation tie-breaks, and progress arithmetic. Public-seam tests were added for the high-value contracts rather than assertions against private implementation details. The remaining survivors cluster mainly around progress-percentage arithmetic, equivalent or indistinguishable boolean rewrites, repository aggregation tie-breaks, bounded route/CSS parser internals, and parser loop index arithmetic. They are retained visibly; none were suppressed or excluded to manufacture the threshold.

The public regression manifests include:

- `tests/qualification_coverage_boundaries_seam.rs`
- `tests/qualification_policy_baseline_seam.rs`
- `tests/qualification_style_route_seam.rs`
- `scripts/tests/test_mutation_score.py`

This evidence satisfies the local TEST-004 gate. It does not satisfy unrelated hosted, reference-runner, authenticated-release, or customer-calibration requirements.
