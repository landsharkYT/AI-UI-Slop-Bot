# AI UI Slop Analysis

This context defines the language used to identify and report generic AI-default frontend aesthetics without claiming to determine authorship or universal design quality.

## Language

**AI UI Slop**:
A repeated, context-insensitive bundle of fashionable interface patterns that makes a product resemble generic vibe-coded web applications instead of expressing deliberate hierarchy, product identity, or usability needs. The term describes aesthetic convergence, not who or what authored the code.
_Avoid_: Bad UI, ugly UI, AI-generated UI

**Slop Signal**:
An observable source-code trait associated with AI UI Slop. A signal is evidence only in context and is not independently a violation.
_Avoid_: Offense, proof, AI tell

**Slop Pattern**:
A meaningful combination or repetition of Slop Signals that the configured policy considers evidence of aesthetic convergence.
_Avoid_: AI signature, bad style

**AI Slop Score**:
An explainable 0–100 estimate of how strongly a Finding, Component Profile, or Repository Profile exhibits configured Slop Patterns. It prioritizes review within one Analysis Scope under compatible scanner versions and policy; it is not a percentage, probability, authorship detector, objective measure of design quality, or a value to blend across applications.
_Avoid_: AI probability, quality score, certainty score

**Finding Confidence**:
The scanner's certainty that it correctly resolved the relevant source and matched the reported Slop Pattern. Confidence is distinct from the AI Slop Score: a strongly convergent pattern can still have low confidence when its classes or component relationships are only partially resolved.
_Avoid_: Severity, AI probability

**Analysis Coverage**:
The measured completeness vector for parsing eligible source bytes, resolving candidate style expressions, resolving supported local component edges, and resolving detected route declarations. Each dimension exposes its numerator, denominator, exclusions, and unresolved reasons; the dimensions are never collapsed into one percentage and remain distinct from Finding Confidence or absence of Findings.
_Avoid_: Confidence, clean score, overall coverage percentage

**Policy Disposition**:
The configured treatment of a Finding: report it, suppress it, or enforce it. Disposition expresses repository policy rather than convergence strength or analysis certainty.
_Avoid_: Severity, importance

**Trusted Policy**:
The complete set of enforcement-affecting inputs obtained from a protected target-branch revision or equivalently protected workflow source, including scope and ignore decisions, House Style, Suppressions, baseline, thresholds, dispositions, resource ceilings, custom archetypes, and version pins. Pull-request-controlled changes may propose future Trusted Policy but cannot weaken enforcement of the pull request that introduces them.
_Avoid_: PR configuration, current checkout policy

**Finding**:
A component-scoped, explainable report that exactly one Slop Pattern matches one actionable owner in one compatible reachable-state family and warrants review. Elements provide the evidence; the owning component identifies the practical refactoring unit.
_Avoid_: Conviction, proof of AI authorship

**Component Profile**:
The scored view of all Findings owned by one React component. It emphasizes the strongest Finding and only capped breadth from distinct additional patterns, so component size or duplicate evidence cannot dominate the score.
_Avoid_: Sum of findings, page score

**Finding Fingerprint**:
A stable identity derived from Analysis Scope, rule, normalized module/export owner, a Rule Contract-defined semantic occurrence key, and compatible reachable-state family. Matching evidence aggregates into one Finding by default; rules allowing several occurrences define collision behavior explicitly. The fingerprint supports matching across ordinary source movement while excluding locations, scores, messages, evidence weights, and presentation details.
_Avoid_: Line key, finding ID

**Evidence Digest**:
A versioned digest of a Finding's normalized Slop Signals and interactions, kept separate from Finding Fingerprint so evidence can change without losing the Finding's identity.
_Avoid_: Finding identity, source hash

**Reviewed Baseline**:
The explicitly accepted, auditable snapshot of unresolved Findings remaining after the initial cleanup pass. It records compatible versions, policy, review metadata, Finding identities, evidence, and rationale while allowing later enforcement to reject new or materially worsened Findings.
_Avoid_: Ignore list, clean state

**Materially Worsened Finding**:
A baseline-matched Finding that enters a higher score band, increases by at least 10 points, or gains a Rule Contract-designated enforcement-relevant interaction under compatible versions and policy. Confidence change alone is not worsening.
_Avoid_: Any changed finding, recalibration difference

**Repository Profile**:
An aggregate view of Component Profiles within one Analysis Scope that exposes bounded prevalence, recurrence, distribution, and aesthetic convergence invisible within any single component.
_Avoid_: Site score, quality grade

**Analysis Scope**:
A configured frontend application or workspace whose components share a House Style Profile, Repository Profile, and Reviewed Baseline. A monorepository may contain several independent Analysis Scopes that must not contaminate one another's scores.
_Avoid_: Package, whole monorepo

**Refactoring Brief**:
An ordered, agent-readable handoff that groups related Findings into coherent cleanup work while stating evidence, House Style constraints, Preservation Obligations, permitted discretion, and independent verification expectations. It directs work without supplying an automatic patch, replacement aesthetic, or runtime-equivalence guarantee.
_Avoid_: Autofix, redesign specification

**Preservation Obligation**:
An observable, configured, or explicitly unknown behavior or accessibility contract that a cleanup must preserve or improve and independently verify. It records evidence and verification expectations but is not a claim that static analysis proved runtime equivalence.
_Avoid_: Preservation guarantee, verified behavior

**Capability Requirement**:
A vendor-neutral description of specialized help a Refactoring Brief expects, such as visual hierarchy, design systems, interaction design, or brand expression. A House Style Profile may map it to locally available agent skills.
_Avoid_: Hard-coded skill name, plugin dependency

**Repeated Decorative Shell**:
A display-container treatment that combines several fashionable decorative effects and recurs across otherwise distinct components, making the interface feel templated or interchangeable. Individual effects are not sufficient evidence.
_Avoid_: Card usage, rounded container

**Effect Stacking**:
Several high-intensity decorative effects coexisting on one reachable UI element or tightly coupled element group. It does not require repository-wide repetition.
_Avoid_: Styled element, multiple classes

**Decoration Saturation**:
One decorative treatment repeated across enough of a component or page that it overwhelms hierarchy and makes distinct content regions look interchangeable.
_Avoid_: Consistency, decoration use

**Shape Homogenization**:
Indiscriminate repetition of the same conspicuous silhouette, such as pills or heavily rounded containers, across elements with different roles.
_Avoid_: Border radius, shared shape token

**Cardification**:
Content divided into unnecessary nested or repetitive floating containers until the card treatment replaces meaningful grouping and hierarchy.
_Avoid_: Card component, sectioning

**Generic Container Depth**:
Deep non-semantic wrapper hierarchy that participates in decorative layering without adding meaningful structure. Nesting depth alone is not sufficient evidence.
_Avoid_: Deep JSX, div usage

**Design Token Drift**:
Repeated visual values that diverge from the approved House Style scale and create arbitrary or template-like styling. Arbitrary syntax or a single exception is not sufficient evidence.
_Avoid_: Arbitrary value, custom token

**Rhythm Homogenization**:
Excessively uniform spacing, sizing, and repeated section or component rhythm that flattens distinctions between different kinds of content.
_Avoid_: Consistent spacing, grid use

**Template Convergence**:
A page-level combination of several stock structures commonly assembled into interchangeable web applications, such as an eyebrow pill, centered promotional hero, CTA pair, framed product image, bento grid, or generic feature cards. No individual structure is sufficient evidence.
_Avoid_: Landing page, hero section, template use

**Framework Default Convergence**:
A recurring stock framework recipe—neutral palette, familiar accent, preset rounding, elevation, compact type, or mirrored dark treatment—shared across distinct component owners. Palette or framework use alone is not sufficient evidence.
_Avoid_: Tailwind usage, default color

**Control Surface Homogenization**:
Compact outlined surface chrome applied across controls, content, and structural regions until unlike roles share one dense visual treatment. A coherent toolbar or one control family is not sufficient evidence.
_Avoid_: Dense UI, small text, square controls

**Page Archetype**:
A recognizable page purpose and structural family used to interpret composition in context, such as marketing, dashboard, checkout, settings, or documentation. The taxonomy is open through declarative combinations of versioned built-in structural signals; an unknown archetype still receives generic analysis rather than being excluded.
_Avoid_: Template, route name

**House Style**:
The product-specific visual language against which generic defaults and acceptable exceptions are judged.
_Avoid_: Good taste, correct design

**House Style Profile**:
The explicit, reviewable repository policy that identifies approved design tokens, primitives, reference components, exceptions, and unwanted combinations. Inferred conventions are only proposals until accepted by the Design Authority.
_Avoid_: Learned taste, automatic style

**Suppression**:
A narrow, attributable exception for one Finding or rule application, including its rationale and optionally its expiry. Reusable intentional patterns belong in the House Style Profile instead.
_Avoid_: Ignore, blanket exemption

**Design Authority**:
The person responsible for deciding whether proposed Findings accurately identify unwanted convergence and whether a cleanup fits the intended House Style. The project owner is the initial Design Authority; external maintainers validate whether default policy generalizes.
_Avoid_: Average user, universal taste

**Calibration Case**:
A labeled example containing frontend source, a rendered visual reference, expected Findings, acceptable exceptions, and reviewer judgments. Visuals validate whether source signals correspond to perceived convergence; they are not scanner input in V1.
_Avoid_: Training example, screenshot rule
