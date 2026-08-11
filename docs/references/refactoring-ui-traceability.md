# Refactoring UI Traceability

Status: Accepted reference policy for the V1 candidate.

Last reviewed: 2026-07-30.

## Source boundary

This project uses *Refactoring UI* by Adam Wathan and Steve Schoger as a major source of design vocabulary, counterexamples, fixture ideas, and remediation direction. It does not treat the book as a machine-enforceable specification or a universal definition of good design.

The committed analysis below is independently expressed and is based on the authors' official public material:

- [Refactoring UI official site and complete chapter list](https://refactoringui.com/)
- [Building Your Color Palette](https://refactoringui.com/previews/building-your-color-palette)
- [Labels Are a Last Resort](https://refactoringui.com/previews/labels-are-a-last-resort)
- [Line-height Is Proportional](https://refactoringui.com/previews/line-height-is-proportional)

The official material establishes two especially important boundaries for this scanner. First, visual choices communicate hierarchy and should form deliberate systems. Second, the appropriate treatment depends on content and context. Therefore:

> No book tactic activates a Finding by itself.

The proprietary book, illustrations, component gallery, palettes, and videos are not committed, reproduced, converted into fixtures, or treated as though they were reviewed when only public material was available. The private initial design brief is product-discovery input, not repository content or provenance for claims about the book.

## Gemini proposal disposition

| Proposal in the private initial design brief | Disposition | Implemented interpretation |
| --- | --- | --- |
| Use weighted combinations instead of binary gradient detection. | Accepted | Findings use distinct semantic signals, nonlinear interactions, recurrence, and caps. |
| Produce source-located structured output for a refactoring agent. | Accepted | Canonical Findings and Refactoring Briefs carry owner, location, evidence, explanation, remediation, and preservation context. |
| Use *Refactoring UI* and design-system tooling as heuristic inputs. | Accepted with provenance limits | Official public material informs the matrix; executable rules remain independently specified and calibrated. |
| Penalize values merely for exceeding a book-derived maximum. | Refined | Intensity can support a rule, but no radius, shadow, spacing, palette, or class count is a violation by itself. |
| Declare a single element “mathematically confirmed” as slop at a fixed point threshold. | Rejected | Scores express convergence strength under policy, not mathematical certainty, authorship, or universal quality. |
| Flag long class strings or deep `div` nesting directly. | Rejected | Raw class count never contributes; generic depth requires decorative layering and contextual bounds. |
| Automatically strip visual classes and impose a brutalist/minimalist replacement. | Rejected | Remediation preserves behavior and House Style and must not replace one generic aesthetic with another. |
| Assume model training on a particular proprietary book explains observed defaults. | Not used | Authorship and training-corpus speculation are irrelevant to observable aesthetic convergence. |

## Evidence classes

### Deterministic evidence

Observable source facts may contribute only after a versioned Rule Contract supplies recurrence, co-occurrence, role, ownership, House Style, score, and counterexample boundaries. Examples include repeated spacing values, repeated visual traits across roles, or several high-intensity effects on one reachable element.

### Explanation and remediation only

Principles such as restoring hierarchy, limiting arbitrary choices, using elevation purposefully, or differentiating supporting content guide Finding explanations and cleanup briefs. They do not independently change activation or score.

### Human judgment only

Personality, beauty, brand fit, font quality, photographic quality, color taste, whether a grid is aesthetically appropriate, and whether a composition feels distinctive require the Design Authority or calibrated reviewers. They are never presented as deterministic conclusions.

## Rule traceability matrix

| Rule | Public reference area | Translation used by this project | Protected counterexample | Evidence class |
| --- | --- | --- | --- | --- |
| `repeated-decorative-shell` | Creating Depth; Finishing Touches | Several intense surface treatments must recur across distinct owners. | One focal elevated surface or a reviewed shared primitive. | Deterministic evidence |
| `template-convergence` | Starting from Scratch; Hierarchy Is Everything | Several stock structures must participate in one page formula. | A conventional section in isolation or product-specific composition. | Deterministic evidence plus human judgment |
| `effect-stacking` | Creating Depth; Finishing Touches | Several supported high-intensity categories must coexist on one reachable element. | One purposeful shadow, border, gradient, or rounded surface. | Deterministic evidence |
| `decoration-saturation` | Hierarchy Is Everything; Finishing Touches | One treatment must dominate enough eligible elements to flatten hierarchy. | Consistent treatment reserved for one hierarchy-bearing role. | Deterministic evidence |
| `shape-homogenization` | Hierarchy Is Everything; Starting from Scratch | One conspicuous silhouette must recur across several structural roles. | Rounded controls within one coherent control family. | Deterministic evidence |
| `cardification` | Hierarchy Is Everything; Layout and Spacing | Repetitive or nested surfaces must replace meaningful grouping. | One semantic card or a restrained grouping surface. | Deterministic evidence plus explanation |
| `generic-container-depth` | Layout and Spacing | Deep generic wrappers matter only when coupled with decorative layering. | Layout or behavior wrappers without decorative convergence. | Independently derived deterministic evidence |
| `design-token-drift` | Layout and Spacing; Designing Text; Working with Color | Repeated values are compared only with an explicit reviewed House Style scale. | A deliberate exception or a repository without an approved scale. | Deterministic evidence |
| `rhythm-homogenization` | Hierarchy Is Everything; Layout and Spacing | One spacing value must dominate several elements across distinct roles. | A uniform list or grid whose items share one content role. | Deterministic evidence |
| `framework-default-convergence` | Choose a Personality; Limit Your Choices; Working with Color | A stock palette and component recipe must recur across at least three owners. | Framework use, palette use, or one component recipe in isolation. | Deterministic evidence plus human judgment |
| `control-surface-homogenization` | Hierarchy Is Everything; Use Fewer Borders | Compact outlined chrome must saturate elements across distinct roles. | One toolbar, one control family, or one useful divider. | Deterministic evidence |

## Remediation vocabulary

Refactoring Briefs may use these independently expressed directions:

- restore differences in emphasis between primary, supporting, and structural content;
- replace arbitrary values with a small reviewed scale where a House Style exists;
- reserve elevation, borders, and conspicuous silhouettes for roles where they communicate something;
- use spacing, contrast, grouping, or typography to communicate hierarchy instead of surrounding every region with the same chrome;
- preserve deliberate product personality rather than replacing one stock aesthetic with another.

They must not command an agent to remove every gradient, shadow, border, card, or rounded corner. They must preserve behavior and accessibility obligations and defer taste-specific choices to the House Style or Design Authority.

## Calibration fixtures

| Case | Location | Expected result | Boundary established |
| --- | --- | --- | --- |
| Isolated tactics and apparent violations | `tests/fixtures/refactoring-ui-boundaries` | No Findings | Advice about borders, elevation, typography, labels, and rounded controls is not directly enforced. |
| Repeated framework recipe | `tests/shot_c_broader_convergence_seam.rs` | Framework Default Convergence across three owners | Convergence requires a recurring multi-signal recipe. |
| Cross-role compact chrome | `tests/shot_c_broader_convergence_seam.rs` | One Control Surface Homogenization Finding | Dense chrome requires saturation across roles. |
| One coherent toolbar | `tests/shot_c_broader_convergence_seam.rs` | No Control Surface Homogenization | Consistency within one role is protected. |
| Dispersed unrelated traits | `tests/shot_c_broader_convergence_seam.rs` | No Control Surface Homogenization | Separate traits are not assembled into a fictional treatment. |

These fixtures are independently authored under this repository's license. They reproduce no book text, screenshots, illustrations, component-gallery assets, or proprietary code.
