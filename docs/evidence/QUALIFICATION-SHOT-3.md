# Qualification Shot 3 evidence

Status: qualification program implemented and locally rehearsed; hosted gates remain pending.

Shot 3 converts raw automated results into fail-closed qualification decisions:

- `scripts/qualification-program.py` independently validates reference-runner, progress, and complete native-target evidence against the frozen program;
- `scripts/native-smoke.py` records binary identity, version/rule-pack identity, runner identity, and two deterministic fresh-fixture scans;
- progress evidence now records runner image, logical processor allocation, scanner revision/version, rule-pack version, and fixture version;
- the manual reference and progress workflow validates evidence before artifact upload;
- the native release workflow supports non-publishing manual qualification, smokes all five built binaries on their target runners, aggregates the complete matrix, and permits publishing only for a tag after native qualification passes;
- `qualification/program.md` defines the complete automated-to-human execution and evidence-promotion sequence.

The 2026-08-01 local rehearsal was deliberately not promoted. On the local 24-logical-processor machine, the 2,000-file workload completed in 66 ms at 14,636 KiB peak RSS and the 500,000-line workload completed in 111 ms at 14,456 KiB. The 20-pair progress trial measured -0.2478% median overhead with identical report hashes and scanner outcomes. The local Linux x86-64 binary was 4,443,784 bytes and produced identical SHA-256 report digests across two fresh-fixture scans.

All three program decisions correctly failed qualification: reference and progress rejected `local-unqualified`, the absent hosted `ImageVersion`, and the 24-processor allocation; native qualification rejected the local runner identity and four missing target records. These failures demonstrate evidence integrity, not product-gate failures on the required hosted environments.

No requirements-verification status changes in Shot 3. TEST-004 remains the completed automated local gate; reference-runner performance, progress, all-target native execution, authenticated release publication, and blind human calibration remain pending until their workflows or trials are actually executed.
