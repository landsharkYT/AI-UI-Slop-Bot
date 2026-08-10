# Exclude non-application source from default UI evidence

Status: Accepted

Default Findings and Analysis Coverage will use application source rather than every syntactically eligible JSX module. Conventional test, specification, mock, and fixture modules are classified as non-application Source Roles and do not contribute Findings, ownership penalties, or coverage denominators; exclusions remain visible as policy decisions. Story and visual-test modules require explicit inclusion or a separate Analysis Scope because they may represent a design-system review surface without being shipped application UI.
