# Use a typed hybrid rule pack

Status: Proposed

If ADR 0003 accepts the Rust/Oxc core, the built-in rule pack will keep extraction, reachability, graph analysis, scoring mechanics, and complex predicates in typed Rust while storing calibrated weights, caps, bands, archetype signatures, explanations, references, and counterexample metadata as schema-validated data embedded in the binary. If ADR 0003 is rejected, this ADR reopens with it; any replacement must preserve a typed mechanics layer, schema-validated calibration data, and one versioned artifact without creating a general-purpose external rule language.
