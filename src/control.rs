use std::sync::{
    Arc,
    atomic::{AtomicU8, Ordering},
};

/// Cooperative cancellation shared by the CLI, repository scheduler, and analyzers.
#[derive(Debug, Clone, Default)]
pub struct CancellationToken {
    interrupts: Arc<AtomicU8>,
}

impl CancellationToken {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record an interrupt and return its one-based sequence number.
    pub fn cancel(&self) -> u8 {
        self.interrupts
            .fetch_add(1, Ordering::SeqCst)
            .saturating_add(1)
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.interrupts.load(Ordering::Acquire) > 0
    }
}
