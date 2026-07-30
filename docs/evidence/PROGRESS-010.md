# PROGRESS-010 Local Progress Overhead Evidence

Status: local automated gate passed; pinned reference-runner reproduction and human trial remain pending.

Date: 2026-07-29

`scripts/progress-trial.py` ran twenty alternating `always`/`never` pairs against the same deterministic 500,000-line workload. Every pair produced the same exit code and byte-identical JSON digest.

- Script SHA-256: `ce91ce2b578663a8d4c734c236951d4965b9bd2c1de0dba4cf72217d5a42cd2c`
- Binary SHA-256: `8b3039b44d3ac0ba59c80f59ea41cdcfb223190c8bad47106126f2505c483c65`
- Pairs: 20
- Median paired overhead: 1.3387%
- Empirical 95% interval: -16.0792% to 17.8133%
- Median acceptance threshold: no more than 2%
- Result: pass
- Canonical report SHA-256 in the first pair: `64ffe8338ae17c902eac7e8b3e2090ef6043f1d896ee7073ee3d5080597b2bee`

The interval is the empirical paired-delta interval, not a claim about all hardware. The raw JSON remains a generated benchmark artifact under `target/progress-evidence.json`; a pinned CI evidence job should publish it before release.
