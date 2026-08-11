# Shot 6: Semantic Repository Ownership Candidate

Status: implemented locally; final implementation and validation gates remain explicit.

Shot 6 concentrates React ownership, source admission, module resolution, and shared-primitive impact behind the repository analysis interface. Findings remain attached to the component that owns the styling; use sites are impact evidence rather than duplicate Findings.

## Implemented

- configurable transparent component wrappers with nested `memo`/`forwardRef` support and diagnostics for unconfigured render-transforming wrappers;
- `React.Component` and `React.PureComponent` class ownership, with explicit decorator and unsupported-inheritance diagnostics;
- classic `createElement` and automatic `_jsx`/`_jsxs`/`jsx`/`jsxs` static factory extraction through the same candidate path as JSX;
- configured JSX parsing in `.js`, `.jsx`, `.ts`, and `.tsx` files;
- inherited local tsconfig/jsconfig path aliases and workspace package `exports` resolution;
- deterministic import edges for named/wildcard re-exports, cyclic barrels, and literal lazy imports;
- canonical Finding impact evidence for rendered uses of a responsible shared primitive (upgraded from file-level to component-level `path#owner` identities in `0.13.0`);
- scalar-or-array CVA compound selectors;
- static exclusivity for conflicting `data-[key=value]` and `aria-[key=value]` conditions, in addition to `dark`/`light`;
- canonical report schema 5 and scanner `0.6.0`.

## Deliberate boundaries

Wrapper aliases must be configured and transparent. Arbitrary HOCs, decorators, mixins, and dynamic inheritance produce diagnostics rather than guessed ownership. Automatic-runtime factories transformed or renamed beyond the supported static names are unresolved. Repository exports resolve module targets, but symbol-level provenance through arbitrary conditional exports is not yet claimed. Primitive impact sites are canonical files, not exact call ranges. Runtime CVA values, arbitrary custom-variant logic, CSS Modules, CSS-in-JS, the browser cascade, and target plugins remain outside the implemented static surface.

Shot 7 still owns the final implementation closeout: cancellation and allocator-accounted memory enforcement, remaining release/packaging qualification, complete traceability evidence, and cleanup of any acceptance gaps found by the full audit.

This is scanner `0.6.0`, not Validated MVP or Full V1.
