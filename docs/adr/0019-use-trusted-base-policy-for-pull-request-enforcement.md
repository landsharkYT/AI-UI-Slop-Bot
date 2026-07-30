# Use Trusted Policy for pull-request enforcement

Status: Accepted

Full V1 pull-request enforcement will analyze contributor-controlled source using every enforcement-affecting input from the protected target branch or equivalently protected workflow inputs, including scopes, ignore policy, House Style, Suppressions, baseline, thresholds, resource ceilings, custom archetypes, and version pins. Pull-request changes to those inputs are reported as proposals but cannot weaken their own check; new supported source outside trusted scopes creates explicit coverage failure. This preserves read-only fork safety without allowing a contributor to approve a baseline, hide files, or suppress the Findings being reviewed.
