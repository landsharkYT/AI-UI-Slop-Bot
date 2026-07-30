# REFERENCE-002 Evidence

Status: local pass; release licensing audit remains pending.

Date: 2026-07-30.

## Requirement

Derived rule rationale must cite its source without copying substantial proprietary text.

## Source and licensing review

The repository cites only official public pages:

- <https://refactoringui.com/>
- <https://refactoringui.com/previews/building-your-color-palette>
- <https://refactoringui.com/previews/labels-are-a-last-resort>
- <https://refactoringui.com/previews/line-height-is-proportional>

The [traceability matrix](../references/refactoring-ui-traceability.md) paraphrases high-level rationale and uses public chapter or preview titles for identification. It contains no book pages, screenshots, illustrations, component-gallery designs, palettes, video frames, or substantial passages.

The rule contracts link to that matrix and keep executable activation criteria independently specified. The calibration fixture is marked `independently-authored` and `MIT OR Apache-2.0` in its manifest.

The proprietary book itself was not supplied to this implementation and is not represented as fully reviewed. A future licensed full-book review may deepen the matrix, but it must preserve this citation and non-redistribution boundary.

## Automated guard

`tests/shot_d_refactoring_ui_traceability.rs` requires the official source URLs and the separation between deterministic evidence, remediation guidance, and human judgment. Normal repository review and `LICENSE-003` release evidence remain responsible for checking later additions and packaged artifacts.

## Conclusion

The Shot D additions satisfy local rationale attribution and non-redistribution review. Release-wide third-party-content verification remains pending under `LICENSE-003`.
