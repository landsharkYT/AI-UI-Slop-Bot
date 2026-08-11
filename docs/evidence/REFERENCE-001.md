# REFERENCE-001 Evidence

Status: local pass; representative-repository and customer calibration remain pending.

Date: 2026-07-30.

## Requirement

*Refactoring UI* must inform the product without becoming a machine-enforceable specification or the sole definition of good UI. Candidate tactics must be separated into deterministic evidence, explanation/remediation guidance, and human judgment.

## Evidence

The [Refactoring UI traceability matrix](../references/refactoring-ui-traceability.md):

- identifies the official public source boundary;
- classifies reference material into deterministic evidence, explanation/remediation only, and human judgment only;
- maps every executable rule to independently expressed rationale and a protected counterexample;
- states that no book tactic activates a Finding by itself; and
- identifies the private initial design brief as product-discovery input rather than repository content or source provenance.

The executable seam `tests/shot_d_refactoring_ui_traceability.rs` verifies that every catalog rule appears in the matrix and that all three evidence classes and official sources remain explicit. Its permanent acceptable-negative fixture proves that one divider, one elevated focal surface, context-dependent typography, explicit labels, and one rounded control produce no Findings.

The pre-existing Shot C seams retain the positive convergence boundaries: repeated framework recipes and genuinely cross-role dense chrome activate, while one toolbar and dispersed traits do not.

## Reproduction

```sh
cargo test --test shot_d_refactoring_ui_traceability
```

Local result: 2 passed, 0 failed.

## Fixture record

| File | Lines | Bytes | SHA-256 |
| --- | ---: | ---: | --- |
| `tests/fixtures/refactoring-ui-boundaries/IntentionalInterface.tsx` | 9 | 483 | `0befa809e9bb0c8c4c5a9334c1c17327433ee3750a1e1873f83f522ed625507d` |
| `tests/fixtures/refactoring-ui-boundaries/fixture-manifest.json` | 15 | 508 | `050fcf679ca7f244bb788ff38d9b93c1cad6128922ca957ff11a2fa96d79ba31` |

The fixture is independently authored and contains no copied book text or visual material.

## Remaining validation

This evidence verifies the reference-policy implementation. It does not validate taste, customer satisfaction, or universal-default accuracy. Those claims remain subject to the blind calibration requirements.
