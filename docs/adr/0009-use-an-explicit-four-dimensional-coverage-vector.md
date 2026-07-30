# Use an explicit four-dimensional coverage vector

Status: Accepted

Analysis Coverage will be reported independently for parsed eligible source bytes, resolved candidate style expressions, resolved supported local component edges, and resolved route declarations, with numerator, denominator, exclusions, and unresolved reason codes for every dimension. The scanner will never present a composite coverage percentage; enforcement uses explicit per-dimension policy floors and returns the insufficient-analysis outcome when a required floor is missed. This prevents a strong result in one analyzer stage from concealing a blind spot in another and makes apparent absence of Findings harder to mistake for a clean repository.
