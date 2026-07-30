# Separate Finding identity, evidence, and baseline review

Status: Accepted

Finding identity will be a versioned fingerprint of scope, rule, normalized owner, a Rule Contract-defined semantic occurrence key, and reachable-state family, while a separate evidence digest captures normalized signals and interactions. Matching evidence aggregates into one Finding by default; rules permitting several occurrences define collision behavior explicitly. A baseline becomes Reviewed only through an explicit acceptance command that records both algorithm versions, policy, source and review metadata, and rationale after showing a semantic diff. Under compatible inputs, worsening means an upward score-band move, a score increase of at least 10, or a newly gained Rule Contract-designated material interaction; confidence changes and incompatible recalibration require review instead of enforcement, and ambiguous matches produce insufficient-analysis status rather than policy failure.
