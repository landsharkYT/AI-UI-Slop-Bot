# TEST-004 mutation qualification

Status: local pass on 2026-07-31.

The rule and policy mutation gate was executed with cargo-mutants 27.0.0 against `src/app.rs`, `src/baseline.rs`, `src/policy.rs`, and `src/style.rs`:

```sh
TMPDIR="$PWD/target/mutants-tmp" cargo mutants --no-shuffle --jobs 4 \
  --timeout-multiplier 3 --minimum-test-timeout 20 \
  --output target/qualification/mutants-shot2-qualified \
  --file src/app.rs --file src/baseline.rs --file src/policy.rs --file src/style.rs
scripts/mutation-score.py target/qualification/mutants-shot2-qualified \
  --output target/qualification/mutation-score-shot2.json
```

The complete, non-iterative run tested 949 mutants: 741 caught, 173 missed, 5 timed out, and 30 unviable. Timeouts count as uncaught viable mutants, producing 741 / 919 = **80.6311%**, above the provisional 80% minimum. The evidence normalizer verified the cargo-mutants end timestamp and exact tested/total count; it rejects incomplete and iterative output.

Survivor review found meaningful gaps in coverage thresholds, cancellation classification, baseline compatibility and migration behavior, ordered score bands, calendar validation, Tailwind detection, exact auxiliary-file ceilings, CSS value resolution, route discovery, aggregation tie-breaks, and progress arithmetic. Public-seam tests were added for the high-value contracts rather than assertions against private implementation details. The remaining survivors cluster mainly around progress-percentage arithmetic, equivalent or indistinguishable boolean rewrites, repository aggregation tie-breaks, bounded route/CSS parser internals, and parser loop index arithmetic. They are retained visibly; none were suppressed or excluded to manufacture the threshold.

The added public regression manifests are:

- `tests/qualification_coverage_boundaries_seam.rs`
- `tests/qualification_policy_baseline_seam.rs`
- `tests/qualification_style_route_seam.rs`
- `scripts/tests/test_mutation_score.py`

This evidence satisfies the local TEST-004 gate. It does not satisfy unrelated hosted, reference-runner, authenticated-release, or customer-calibration requirements.
