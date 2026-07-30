# Shot 7 Local Qualification Evidence

Status: local automated gates passed; external validation remains pending.

Date: 2026-07-29

Scanner `0.7.0` and canonical report schema 6 were qualified on Linux x86-64 with:

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
python3 scripts/audit-requirements.py
```

The suite contains 64 public-seam tests. Shot 7 adds direct evidence for:

- AST-node and parser-arena accounting;
- incomplete, non-clean outcomes when analysis-memory admission fails;
- deterministic per-reason diagnostic truncation;
- first-interrupt exit `130`, a concise cancellation diagnostic, empty machine-readable stdout, and absence of committed report artifacts;
- final CLI version, generated resource defaults, and composite-Action wall-time/outer-memory controls.

The full suite also retains deterministic parallel reports, hostile-output safety, Trusted Policy behavior, symlink boundaries, graph and source ceilings, baseline lifecycle checks, all nine rule paths, and all fourteen Page Archetype definitions.

The optimized local binary SHA-256 was `14eeef74b0302b6e1a591eb5a5f68129f0464ef943a6b90d357117b0585971a3`, its size was 4,111,728 bytes, and `Cargo.lock` SHA-256 was `50a7146175322544b574a2c94955bfcbbd473c597224b62063d32ccfdaf9c8db`.

The fixed 500-file, 500,500-line local workload completed in 98 ms with measured peak RSS of 14,700 KiB. The twenty-pair progress trial produced byte-identical reports and a median paired overhead of 0.122%, passing the provisional 2% median gate. These are local observations, not substitutes for the pinned reference runner.

This evidence is local implementation qualification. Customer calibration, reference-runner performance, hosted target smoke tests, and authenticated release artifacts remain external validation gates.
