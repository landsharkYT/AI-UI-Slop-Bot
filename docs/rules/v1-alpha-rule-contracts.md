# V1 Alpha Rule Contracts

| Field | Value |
| --- | --- |
| Rule-pack version | `1.0.0-beta.8` |
| Status | Accepted for Shot 1 breadth implementation |
| Calibration status | Pending; not release-approved |
| Last updated | 2026-08-01 |

These contracts authorize conservative V1-shaped matchers for Shot 1. They make the calculation and evidence shapes executable across the full rule pack without claiming customer validation. A matcher must emit explicit coverage loss instead of guessing when its required evidence is unavailable. Threshold changes require a rule-pack version change and fixture updates.

The existing [Repeated Decorative Shell contract](repeated-decorative-shell.md) remains authoritative for that rule.

Design rationale and counterexample provenance are recorded in the [Refactoring UI traceability matrix](../references/refactoring-ui-traceability.md). That matrix informs explanations and fixture boundaries; this document remains the machine-enforceable contract.

## Shared contract

- The actionable owner is the nearest supported named React function or arrow-function component.
- The reachable-state family is `default`; unresolved runtime branches never contribute co-occurring signals.
- One rule, owner, semantic occurrence key, and state family aggregate into one Finding.
- Literal Tailwind classes, static inline-style values, and bounded statically referenced plain-CSS values are high-confidence evidence. Supported simple-class declarations follow source order, so later declarations override earlier values; plain-CSS custom properties contribute only when global definitions resolve uniquely within bounded recursion and output limits; generated simple `::before` and `::after` decoration composes with its base class. Scoped, unsupported, or ambiguous composition is a coverage diagnostic and does not contribute.
- House Style approval neutralizes only the matched signal contribution. An approved primitive suppresses Findings owned by that exact normalized module/export pair. A narrow Suppression applies only to its exact rule, path, and owner.
- Findings are advisory unless effective policy gives them an `enforce` disposition.
- Scores use the global bands `minimal` 0–19, `low` 20–39, `moderate` 40–59, `high` 60–79, and `dominant` 80–100.
- Every Finding exposes rule and contract versions, fingerprint, evidence digest, path, owner, location, semantic occurrence key, contributions, score, band, confidence, disposition, explanation, remediation, and applicable Page Archetypes.

## Effect Stacking

- Reference rationale: purposeful elevation and finishing treatments should communicate hierarchy; no individual treatment is prohibited.
- Rule ID: `effect-stacking`; contract version: `0.1.0-alpha`.
- Entity: one eligible JSX display element.
- Activation: at least four distinct supported high-intensity decorative categories on the same element.
- Exclusions: controls, dialogs, mutually exclusive or dynamic classes, and several values from one category.
- Score: signal weights from Repeated Decorative Shell plus an 18-point four-category interaction, 25 points for five categories, or 32 points for six; cap 100.
- Occurrence key: sorted participating signal IDs. One owner receives only its strongest compatible occurrence.
- House Style: approved signal IDs lose their weight and no longer count toward the interaction threshold.
- Coverage: unresolved styling prevents that expression from contributing; other fully static elements remain eligible.
- Remediation: remove or subordinate effects while preserving the element's hierarchy and interaction purpose.

## Decoration Saturation

- Reference rationale: visual emphasis loses meaning when the same treatment is applied everywhere.
- Rule ID: `decoration-saturation`; contract version: `0.1.0-alpha`.
- Entity: one component owner and one decorative signal category.
- Activation: the category occurs on at least four eligible display elements and on at least 60% of that owner's eligible display elements.
- Exclusions: fewer than four elements, one shared primitive definition, controls, and approved categories.
- Score: 40 base + 5 per occurrence above four + 20 times the proportion above 60%; cap 80.
- Occurrence key: saturated signal ID.
- Coverage: requires at least 75% resolved style expressions for the owner; otherwise no Finding and explicit coverage loss.
- Remediation: reserve the treatment for hierarchy-bearing regions rather than removing all consistency.

## Shape Homogenization

- Reference rationale: interface personality and hierarchy require role-sensitive choices, not one silhouette on every role.
- Rule ID: `shape-homogenization`; contract version: `0.1.0-alpha`.
- Entity: one component owner and one conspicuous shape (`pill` or `extreme-rounded`).
- Activation: the shape occurs at least four times across at least three distinct structural roles such as navigation, action, content, media, form, or generic container.
- Exclusions: one control family, a declared shared primitive, restrained radius, and approved shapes.
- Score: 45 base + 5 per distinct role + 3 per occurrence above four; cap 82.
- Occurrence key: normalized shape.
- Coverage: unresolved role or style evidence does not count.
- Remediation: restore role-specific silhouettes while retaining the approved design language.

## Cardification

- Reference rationale: spacing, contrast, and grouping can communicate structure without surrounding every region with equivalent chrome.
- Rule ID: `cardification`; contract version: `0.1.0-alpha`.
- Entity: one component owner.
- Activation: either at least five card-like decorated display containers, or at least three with a nested card-like depth of two or more.
- A card-like container has padding plus at least one of outline, shadow, background treatment, or extreme radius. Restrained plain-CSS surfaces require padding and a non-transparent background plus a border or radius; class names are not evidence.
- Exclusions: a single card, data grids, dialogs, and approved shared card primitives.
- Score: 42 base + 5 per card beyond three + 12 for nested-card evidence; cap 85.
- Occurrence key: `owner-card-system`.
- Coverage: unresolved ownership or styling reduces coverage and never creates inferred cards.
- Remediation: recover semantic grouping and hierarchy before simplifying container chrome.

## Generic Container Depth

- Reference rationale: layout guidance informs the remediation, while wrapper depth remains an independently derived source-analysis signal.
- Rule ID: `generic-container-depth`; contract version: `0.1.0-alpha`.
- Entity: one component owner.
- Activation: a non-semantic `div`/`span` wrapper chain reaches depth six and the owner contains at least two supported decorative layers.
- Exclusions: depth alone, semantic sectioning, provider/render-prop wrappers without DOM output, and approved infrastructure primitives.
- Score: 45 base + 5 per level beyond six + 8 per decorative layer beyond two; cap 82.
- Occurrence key: `deep-decorative-wrapper-chain`.
- Coverage: only source JSX emitted by supported owners counts; unresolved component expansion is reported and not assumed.
- Remediation: flatten wrappers only where behavior, layout, focus, and event semantics can be preserved.

## Design Token Drift

- Reference rationale: deliberate spacing, sizing, type, and color systems reduce arbitrary choices; only an explicit House Style scale is enforceable.
- Rule ID: `design-token-drift`; contract version: `0.1.0-alpha`.
- Entity: one normalized visual value across an Analysis Scope, actionable at each participating owner.
- Activation: the same unapproved spacing, radius, shadow, or color value occurs at least three times across at least two owners and is outside a non-empty approved House Style scale for that category.
- Exclusions: a single exception, arbitrary syntax by itself, values in an approved scale, and repositories with no explicit scale for the category.
- Score: 38 base + 5 per occurrence above three + 8 per additional owner; cap 78.
- Occurrence key: category plus normalized value.
- House Style: this rule is inactive for categories without an explicit approved scale; missing policy is visible but not a Finding.
- Coverage: dynamic values never become drift evidence.
- Remediation: choose an approved token or deliberately add the value to the reviewed House Style.

## Rhythm Homogenization

- Reference rationale: spacing should express hierarchy and grouping rather than flatten different roles into one repeated cadence.
- Rule ID: `rhythm-homogenization`; contract version: `0.1.0-alpha`.
- Entity: one component owner and one spacing value.
- Activation: the value appears on at least five eligible display elements, represents at least 80% of resolved spacing-bearing elements, and spans at least two structural roles.
- Exclusions: small uniform lists, one repeated component primitive, approved rhythm tokens, and regular grids whose content role is homogeneous.
- Score: 42 base + 4 per occurrence above five + 10 for at least three roles; cap 78.
- Occurrence key: normalized spacing value.
- Coverage: requires at least five resolved spacing-bearing elements; otherwise not applicable.
- Remediation: introduce hierarchy-driven rhythm changes rather than arbitrary variation.

## Template Convergence

- Reference rationale: starting from product purpose and choosing a deliberate personality protects conventional structures from becoming an interchangeable formula.
- Rule ID: `template-convergence`; contract version: `0.3.0-alpha`.
- Entity: one detected route/page owner evaluated independently for every applicable Page Archetype. Page owners include exact owners established by route discovery plus `App`, `Page`, and owners ending in `Page`, `Screen`, or `View`; a page-like filename alone does not promote helper components.
- Versioned route/archetype structural signals remain `eyebrow-pill`, `centered-hero`, `gradient-heading`, `paired-cta`, `framed-product-media`, `bento-grid`, and `three-card-features`. Rule evaluation also accepts the plain-CSS semantic equivalents `eyebrow-label` and `repeated-panel-grid`; these two internal evidence IDs are not configurable custom-archetype signals.
- Activation: at least three distinct structures participating in one page owner; four signals score high and five or more may score dominant.
- Exclusions: one or two stock structures, structures split across unrelated owners, one grid counted under multiple aliases, functional command grids, action controls carrying layout spans, ordinary equal-column layouts without explicit bento spans, `unknown` classification by itself, and explicitly approved distinctive structures. A three-card feature structure requires exactly three direct elements in an explicit three-column grid. A paired CTA requires exactly two direct elements in a navigation region, or in a flex container with explicit gap.
- Score: 15 per structure + a 10-point three-structure interaction, 18 points for four, or 25 for five or more; cap 100.
- Occurrence key: Page Archetype ID plus sorted structural signal IDs.
- Multiple archetypes: emit one independently explained Finding per matching archetype; component aggregation uses the strongest score plus capped breadth.
- Coverage: route declarations and archetype evidence are reported separately. Page evidence may compose only through uniquely resolved local component names, to depth 8, at most 64 owners, and at most 512 facts. Ambiguous names do not compose. Unknown is a valid result, not a coverage failure.
- Remediation: preserve the page's purpose while replacing interchangeable composition with product-specific hierarchy, interaction, or content shape.

## Framework Default Convergence

- Reference rationale: limiting choices should produce a deliberate product system, not the same framework-default recipe across unrelated products.
- Rule ID: `framework-default-convergence`; contract version: `0.2.0-alpha`.
- Entity: one Analysis Scope, actionable at each participating component owner.
- Activation: one eligible display element carries a coherent recipe of at least four stock framework signals in each of at least three owners. The recurring intersection must include a neutral framework palette and framework rounding.
- Supported signals: neutral slate/gray/zinc/neutral/stone palette, sky accent, framework rounding, large preset elevation, compact preset typography, and mirrored neutral dark-mode treatment.
- Exclusions: signals dispersed across unrelated elements in one owner, isolated use, fewer than three owners, palette use without a recurring component recipe, controls, dialog/transient surfaces, restrained custom CSS, and House Style-approved signals.
- Score: 42 base + 4 per recurring signal on the owner; cap 82. Cross-owner recurrence affects only the bounded Repository Profile contribution, not intrinsic Finding severity.
- Occurrence key: sorted recurring signal IDs.
- Coverage: only statically resolved class states contribute. Dynamic styling remains explicit coverage loss. Each evidence snippet is selected from an element that produced its signal.
- Remediation: replace the repeated stock recipe with product-specific tokens, hierarchy, or component treatment; do not mechanically remove useful dark mode or accessibility behavior.

## Control Surface Homogenization

- Reference rationale: hierarchy can use spacing, contrast, and selective dividers; outlining every role with the same compact chrome creates clutter and sameness.
- Rule ID: `control-surface-homogenization`; contract version: `0.1.0-alpha`.
- Entity: one component owner.
- Activation: at least eight statically styled elements span at least three structural roles, with at least three control-surface traits each recurring four times. Compact typography and outlined chrome must be among the saturated traits, and every participating element has at least three saturated traits.
- Supported traits: compact typography, outlined chrome, neutral surfaces, square chrome, and compact spacing. Traits come from Tailwind utilities or declarations on statically referenced simple plain-CSS classes, independent of class names.
- Exclusions: one coherent control family, fewer than three roles, sparse chrome, unreferenced stylesheets, and House Style-approved traits.
- Score: 42 base + 5 per saturated trait + 3 per role beyond three + 2 per styled element beyond eight; cap 82.
- Occurrence key: `cross-role-compact-chrome`.
- Coverage: ambiguous variables and conditional or compound signal-bearing selectors do not contribute and remain coverage diagnostics.
- Remediation: restore role-specific hierarchy rather than applying one dense control treatment to controls, content, and structural regions alike.

## Built-in Page Archetypes

The V1 alpha catalog contains these stable IDs:

`marketing`, `dashboard`, `authentication`, `onboarding`, `settings`, `pricing`, `commerce`, `portfolio`, `content`, `administration`, `search`, `social`, `workflow`, and `status`.

Classification uses versioned path, owner-name, and structural signals. Several IDs may apply. A detected page with no supported match is `unknown`. Custom archetypes may use only built-in structural signal IDs with required, supporting, and excluding sets; they cannot execute code or define new extractors.

## Required alpha fixtures

Each rule must have at least one positive, one acceptable-negative, one boundary, and one unresolved-input case before Shot 2 calibration. Template Convergence additionally requires a catalog test proving every built-in archetype ID is addressable and that unknown/custom classifications remain safe.
