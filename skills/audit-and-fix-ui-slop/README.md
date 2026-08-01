# Audit and Fix UI Slop agent skill

This portable skill teaches an AI coding agent to run AI UI Slop Bot against a React repository, interpret the evidence conservatively, make justified UI fixes, and verify the result. The included Bash runner records every scan in an agent-readable directory while preserving the scanner's exit code.

## Install

Copy the entire `audit-and-fix-ui-slop` directory into either:

- `.agents/skills/audit-and-fix-ui-slop/` in one repository; or
- `~/.codex/skills/audit-and-fix-ui-slop/` for personal Codex-wide discovery.

The target machine also needs an `ai-ui-slop` binary. Put it on `PATH`, or point the runner to an exact binary:

```sh
export AI_UI_SLOP_BIN=/absolute/path/to/ai-ui-slop
```

Until packaged binaries are published, build this repository and use its release binary:

```sh
cargo build --locked --release
export AI_UI_SLOP_BIN=/absolute/path/to/AIUISlopBot/target/release/ai-ui-slop
```

## Ask an agent to use it

```text
Use $audit-and-fix-ui-slop to audit this repository, fix only justified findings,
run the repository's checks, rescan, and report the before/after evidence.
```

An agent that supports repository-local skills should discover `SKILL.md`. The shell runner is also usable directly:

```sh
.agents/skills/audit-and-fix-ui-slop/scripts/ai-ui-slop-agent.sh doctor .
.agents/skills/audit-and-fix-ui-slop/scripts/ai-ui-slop-agent.sh init .
.agents/skills/audit-and-fix-ui-slop/scripts/ai-ui-slop-agent.sh scan .
```

`init` is the only command that creates scanner configuration. `scan` writes normal scanner reports plus a timestamped evidence bundle beneath `.ai-ui-slop/agent-runs/`. It never edits application source or accepts a baseline.

## Exit codes

The runner preserves scanner exit semantics:

| Code | Meaning |
| ---: | --- |
| `0` | Advisory scan completed; findings may still exist. |
| `1` | Enforcement found a new or worsened finding. |
| `2` | Invalid command, configuration, path, or lifecycle state. |
| `3` | Analysis or artifact coverage is insufficient. |
| `4` | Local operational failure. |
| `130` | Cancelled; canonical artifacts were not committed. |

Do not interpret exit `0` as “the UI is good,” or exit `3` as “no findings.” Read the report's applicability, coverage, evidence, and unresolved diagnostics.
