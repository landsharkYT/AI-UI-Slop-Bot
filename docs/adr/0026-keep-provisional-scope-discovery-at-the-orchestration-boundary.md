# Keep provisional scope discovery at the orchestration boundary

Status: Accepted

An ordinary AI UI Slop scan continues to treat explicit Analysis Scope policy as canonical rather than silently inventing policy. An orchestrator auditing a repository without `ai-ui-slop.config.jsonc` may run `init` inside its disposable snapshot, scan the resulting Scope Draft, and label policy provenance as generated and unreviewed. Existing target configuration must be preserved when present, and a Scope Draft cannot authorize a Reviewed Baseline or enforcement. This keeps zero-write broad audits useful without allowing temporary discovery assumptions to contaminate repository policy.
