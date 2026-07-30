# Repeated Decorative Shell Rule Contract

| Field | Value |
| --- | --- |
| Rule identifier | `repeated-decorative-shell` |
| Contract version | `0.1.0-prototype` |
| Status | Accepted for the Discovery Prototype |
| Applies to | `MILESTONE-002` only |
| Last updated | 2026-07-29 |

## Purpose

Detect the repeated use of the same over-decorated display-container treatment across otherwise distinct React components. A decorative element is not a violation by itself. The rule activates only when a sufficiently rich treatment recurs across distinct component owners.

This contract is deliberately narrow and disposable. It is the executable hypothesis for the Discovery Prototype, not the final MVP rule.

The [Refactoring UI traceability matrix](../references/refactoring-ui-traceability.md) informs its purposeful-elevation counterexamples and hierarchy-oriented remediation. The reference does not prohibit any individual effect and does not alter the activation contract below.

## Analysis entity and scope

- The analysis unit is one JSX element owned by the nearest named React function or named arrow-function component in a `.jsx` or `.tsx` file.
- An owner is identified by normalized repository-relative module path plus component name.
- Only literal `className` strings and literal JSX `style` object values are evaluated.
- An element is eligible when it contains a child element or non-whitespace text and is not one of `button`, `input`, `select`, `textarea`, `option`, or `label`.
- Elements with `role="dialog"`, `role="alertdialog"`, or an ancestor `<dialog>` are excluded.
- Elements for which ownership, reachability, or relevant styling cannot be resolved are not candidates. They contribute to coverage diagnostics instead.
- The prototype analyzes the default render family only. It does not model responsive variants, pseudo-states, runtime class composition, CSS stylesheets, component-library props, or cross-file style indirection.

## Decorative signal categories

Each category can contribute at most once to an element.

| Signal ID | Prototype evidence | Weight |
| --- | --- | ---: |
| `extreme-radius` | `rounded-2xl`, `rounded-3xl`, or a static `borderRadius` of at least 24 px | 12 |
| `gradient-surface` | a Tailwind gradient direction with a static `from-*` and `to-*`, or a static CSS gradient background | 18 |
| `large-shadow` | `shadow-xl`, `shadow-2xl`, or a static `boxShadow` with at least two comma-separated layers | 16 |
| `backdrop-treatment` | a non-zero `backdrop-blur-*`, or a static `backdropFilter` containing `blur(` | 18 |
| `decorative-outline` | a non-zero Tailwind `ring-*`, or a static border paired with a non-default color | 10 |
| `generous-padding` | `p-8` or greater; both axes at `px-8`/`py-8` or greater; or static padding of at least 32 px | 12 |

Tailwind modifiers such as `md:` and `hover:` are outside prototype scope. Their presence is reported as unresolved styling rather than merged into the default state.

## Candidate and recurrence activation

1. A local decorative-shell candidate must contain at least three distinct signal categories on one eligible JSX element.
2. Its normalized shell signature is the sorted set of signal IDs. Raw utility order and literal values do not affect the signature.
3. A recurrence cluster activates when the exact signature appears in at least three distinct component owners.
4. Multiple matching elements in one owner count once toward recurrence. The deterministic representative is the earliest source span.
5. Once activated, the scanner emits one Finding for each participating owner and links each Finding to the same cluster identifier.
6. Similar-but-not-exact signatures do not activate one another in the prototype. Approximate similarity is an MVP calibration question.

## Interactions, score, and caps

The Finding score is the sum of its signal weights plus one interaction bonus, capped at 100:

| Distinct categories | Interaction bonus |
| ---: | ---: |
| 3 | 10 |
| 4 | 18 |
| 5 | 25 |
| 6 | 32 |

Recurrence count does not increase the local Finding score. The prototype reports no repository aggregate score; that remains a validation hypothesis. Bands use the provisional global boundaries: `minimal` 0–19, `low` 20–39, `moderate` 40–59, `high` 60–79, and `dominant` 80–100.

## Confidence and coverage

- A Finding has `high` confidence only when its owner and every contributing signal come from supported static syntax.
- Unsupported dynamic class names, spreads, computed style keys, non-literal style values, parser failures, and excluded files are counted by reason in coverage diagnostics.
- Unsupported styling cannot contribute a signal. The scanner must not infer its value.
- A file with a parser failure contributes no Findings.
- The Markdown and JSON reports must state that absence of Findings is not proof of absence when coverage is incomplete.

## House Style and acceptable counterexamples

The Discovery Prototype has no House Style configuration and makes no score adjustment for approved conventions. All results are advisory.

The following do not trigger:

- one or two decorated shells, regardless of intensity;
- three occurrences inside only one or two component owners;
- compact pills, badges, buttons, and form controls;
- modal/dialog framing;
- three owners whose category signatures differ;
- mutually exclusive or runtime-composed class treatments that cannot be resolved statically;
- a single decorative effect repeated broadly.

## Evidence and report contract

Each Finding must expose:

- rule identifier and contract version;
- stable Finding fingerprint;
- recurrence cluster identifier and distinct owner count;
- repository-relative path and component owner;
- one-based line and column of the representative JSX element;
- normalized shell signature;
- contributing signal IDs, weights, and source snippets;
- interaction bonus, numeric score, band, and confidence;
- a concise explanation and an advisory remediation direction.

The semantic occurrence key is the normalized shell signature. The Finding fingerprint is derived from analysis-scope identifier, rule identifier, normalized path, component owner, semantic occurrence key, and the default reachable-state family. Source position, message text, score, and utility order are excluded.

The cluster identifier is derived from the rule identifier and normalized shell signature. Output ordering is path, owner, semantic occurrence key.

## Calibration cases required by the prototype

The automated fixture set must include:

1. a positive case with the same three-category signature in three owners;
2. a positive case containing all six categories and deterministic scoring;
3. a boundary case with the same signature in only two owners;
4. a boundary case with repeated elements in one owner;
5. a counterexample containing compact interactive controls;
6. a counterexample containing three non-matching signatures;
7. a dynamic-class/style case that produces a coverage diagnostic without a Finding;
8. a malformed-file case that preserves results from valid files and reports the parse failure;
9. an ordering case proving byte-identical canonical JSON across repeated scans.

## Prototype exit criteria

This contract is successful only if the implementation:

- passes the calibration fixtures through the public scanner and CLI seams;
- emits actionable source evidence and explicit coverage;
- produces deterministic JSON and Markdown artifacts;
- demonstrates progress phases without corrupting machine-readable stdout; and
- yields enough evidence to accept, revise, or reject the rule before MVP hardening.

Any threshold, signal, score, or exclusion change requires a contract-version change and fixture update.
