# Use one bounded static-value model

Status: Accepted

Statically resolvable styling will use one shared finite-value model instead of accumulating syntax-specific class adapters. Repository-local literals, concatenation, templates, constant collections, finite branches and switches, side-effect-free local helpers, supported combinators, and CVA selections may produce conditioned Reachable Style States without executing target code. Runtime I/O, mutation-dependent values, arbitrary calls or getters, unknown string domains, and state families beyond `maxReachableStates` remain explicit Coverage Limitations. This broadens useful resolution for real repositories while retaining a single non-execution and complexity boundary.
