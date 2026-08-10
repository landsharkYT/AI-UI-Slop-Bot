# Distinguish coverage limitations from blocking loss

Status: Accepted

An unsupported construct will not automatically make an Analysis Scope incomplete. Measured, localized uncertainty above the configured coverage floor is a Coverage Limitation and remains prominent in diagnostics and affected Finding Confidence; uncertainty becomes Blocking Coverage Loss only when coverage falls below policy, its blast radius cannot be bounded, critical input is missing, or analysis cannot finish reliably. This replaces the absolute requirement that every unresolved style-adapter entry block completion, preserving fail-closed behavior where uncertainty is material without making otherwise useful real-repository reviews provisional because of one bounded construct.

CSS uncertainty is attributed to reachable Style Resolution Units that it can materially affect. An unresolved conditional or compound selector reduces coverage for the reachable `className` or `style` expressions it may alter; unrelated or unreachable stylesheet content does not reduce coverage. A global selector, ambiguous cascade, or other CSS construct whose affected units cannot be bounded remains Blocking Coverage Loss.

Runtime-valued inline properties reduce coverage only when they are Rule-Relevant Style Properties for the active rule pack. Dynamic dimensions, coordinates, transforms, pointer behavior, and other properties outside the active signal vocabulary resolve as irrelevant; activating a rule that consumes one of those properties makes it coverage-relevant under that rule-pack version.

The first corrective pass preserves the existing provisional numerical warning and blocking floors. Attribution, relevance, and blocking semantics will be corrected before thresholds are recalibrated against fresh SlopSweep corpus evidence, so improved completion cannot be manufactured by changing measurement and policy simultaneously.
