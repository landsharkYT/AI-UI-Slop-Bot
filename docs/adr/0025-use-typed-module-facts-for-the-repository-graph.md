# Use typed module facts for the repository graph

Status: Accepted

Repository-graph import, export, lazy-loading, and component-render semantics will come from typed Oxc-derived module facts rather than a second text parser. Eligible source is parsed once and the resulting facts are passed into graph construction; parse failure produces explicit graph coverage loss instead of heuristic recovery. This removes formatting-sensitive disagreement such as semicolonless external imports becoming unresolved local component edges, accepting a tighter interface between source analysis and graph construction in exchange for one syntax authority and more trustworthy ownership evidence.
