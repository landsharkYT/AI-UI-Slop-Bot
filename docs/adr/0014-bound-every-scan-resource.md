# Bound every scan resource

Status: Accepted

Every scan and Analysis Scope will have explicit budgets for scope count, eligible and auxiliary input bytes, file and AST size, configuration imports, graph edges, reachable-state families, diagnostics, allocator-accounted live analysis memory, generated artifacts, and GitHub Action wall time. Scan-global admission prevents many scopes from multiplying work, the Action adds a measurable outer process-memory ceiling, and pull-request configuration cannot raise trusted workflow ceilings. Exhaustion is an insufficient-analysis outcome that records the exact budget and coverage impact, stops only safely isolatable work, and never guesses or leaves partially serialized artifacts; operators may raise ordinary local limits but cannot disable structural defenses. This makes untrusted-repository handling a bounded contract rather than only a prohibition on executing target code.
