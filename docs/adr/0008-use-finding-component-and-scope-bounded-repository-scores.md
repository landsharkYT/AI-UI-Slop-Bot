# Use Finding, Component Profile, and scope-bounded Repository Profile scores

Status: Accepted

AI Slop Scores will belong only to Findings, Component Profiles, and Repository Profiles: a Finding is one rule applied to one actionable owner and compatible reachable-state family; a Component Profile emphasizes its strongest Finding with capped breadth; and a Repository Profile uses bounded prevalence, recurrence, and distribution within one Analysis Scope. Signals carry evidence and intensity but no score, pages reuse the route component's Component Profile rather than introducing another scoring species, and scores from separate monorepository applications are never blended. This object model keeps score explanations actionable and prevents incompatible aggregation across rules, pages, components, and applications.

## Shot B calibrated aggregation

Accepted 2026-07-30 for rule pack `1.0.0-beta.4`.

A Component Profile evaluates each reachable-state family independently. Its selected family is the one with the highest score, then higher Finding Confidence, then canonical state identity. The score is the strongest Finding plus five points for each additional distinct Slop Pattern, with breadth capped at 20 and the total capped at 100. Repeated explanations of one rule across several Page Archetypes do not create breadth.

A Repository Profile is the bounded sum of four named contributions:

- strongest-component severity: 60% of the strongest Component Profile, capped at 60;
- affected-component prevalence: affected owners divided by analyzed owners, capped at 20;
- cross-owner recurrence: three points per pattern-owner occurrence beyond that pattern's first distinct owner, capped at 15; and
- multi-pattern density: two points per owner carrying at least two distinct patterns, capped at 5.

Arithmetic is performed in a widened integer type before normalization. Each contribution exposes its points, cap, evidence count, and explanation. Incomplete Analysis Coverage does not secretly reweight the score; it marks the Repository Profile interpretation `coverage_limited` instead of `qualified`.
