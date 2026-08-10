# Reserve time for graceful specialist completion

Status: Accepted

When AI UI Slop Bot runs beneath an orchestrator, the orchestrator owns the hard process deadline and supplies a shorter cooperative analysis deadline to the scanner. At least five seconds or ten percent of the outer budget, whichever is greater, is reserved for diagnostics, canonical serialization, and atomic artifact commitment. Cooperative expiry returns scanner exit `3` with valid incomplete evidence; orchestrator exit `124` is reserved for failure to finish within the grace period, and cancellation remains exit `130`. This avoids racing equal deadlines and losing the very artifacts needed to explain a slow or incomplete review.
