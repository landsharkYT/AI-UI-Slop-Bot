# Make Analysis Scopes the canonical report boundary

Status: Accepted

Canonical JSON will use one versioned report envelope containing tool, policy, and semantic invocation metadata plus an ordered array of independent Analysis Scope reports. Each scope owns its coverage vector, Component Profiles, Findings, Repository Profile, baseline comparison, diagnostics, and Refactoring Brief batches; the top-level summary contains only counts and per-scope statuses and never blends scores or coverage. Typed unavailable states, deterministic ordering, and presentation views generated from this model make JSON the single semantic source of truth for local output, agents, CI, Markdown, and baselines.
