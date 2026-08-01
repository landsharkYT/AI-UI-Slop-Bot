# Frozen external-maintainer protocol

Use exactly seven external React maintainers. At least four must also attempt the unguided local first-use workflow on a repository they maintain or a representative private/redacted snapshot. Implementation-team members may resolve installation defects but may not explain findings or steer triage.

Before each session, record an anonymous participant ID, relevant React experience, repository revision, scanner/rule-pack versions, and whether the case is part of the balanced holdout assignment. Never collect repository contents automatically.

The participant receives installation instructions, the normal CLI help, the report, and this task only: “Decide whether any reported concern is real, select the first item you would act on, and outline the next implementation step while preserving existing behavior and accessibility.”

Record setup duration, time to first accepted useful Finding, time to initial triage decision, accepted/rejected/ambiguous findings with rationale, abandoned steps, and independent yes/no answers for `accurate`, `actionable`, and `worth using`. Also record reuse intent and willingness to consider advisory CI. A participant passes the main gate only when all three required ratings are yes. At least five of seven must pass. At least three of the four local participants must reach an accepted useful Finding and triage decision within 15 minutes.

Do not discard abstentions, disagreements, failures, or incomplete sessions. Full V1 uses the expanded rule and archetype surface; earlier MVP sessions cannot be relabeled as Full V1 evidence.
