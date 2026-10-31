//! The single frontend that answers one kind of question.
//!
//! Approvals and user questions both go to whichever person-facing surface is
//! attached (the terminal chat UI or a browser's `$events` stream). At most one
//! is attached per kind: a later attach replaces the earlier one, and a question
//! the earlier one still held is refused rather than left to hang.
//!
//! Locking protocol: `slot` is the only lock, it is never held across an
//! `.await`, and every change of who the frontend is ([`Frontend::attach`],
//! [`Inbox`] drop) sends its revocation while holding it. A caller that has
//! received an answer takes the lock once more through [`Frontend::is_current`]
//! and accepts the answer only if its frontend is still the current one. So an
//! answer given just before a detach or replacement is either seen as revoked or
//! refused; it can never be acted on after its frontend has gone.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use tokio::sync::{mpsc, watch};

/// Questions that may wait for a frontend that is not reading them.
pub const QUEUE_CAPACITY: usize = 64;

/// The attached frontend, as the sender sees it.
struct Attached<Q> {
    generation: u64,
    questions: mpsc::Sender<Q>,
    /// Set to true when this frontend detaches or is replaced.
    revoke: Arc<watch::Sender<bool>>,
}

struct Inner<Q> {
    slot: Mutex<Option<Attached<Q>>>,
    next_generation: AtomicU64,
}

/// Routes questions to the attached frontend. Cheap to clone.
pub struct Frontend<Q> {
    inner: Arc<Inner<Q>>,
}

impl<Q> Clone for Frontend<Q> {
    fn clone(&self) -> Self {
        Frontend {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl<Q> Default for Frontend<Q> {
    fn default() -> Self {
        Frontend {
            inner: Arc::new(Inner {
                slot: Mutex::new(None),
                next_generation: AtomicU64::new(1),
            }),
        }
    }
}

/// The frontend's end: questions arrive here. Dropping it detaches the frontend.
pub struct Inbox<Q> {
    pub questions: mpsc::Receiver<Q>,
    frontend: Frontend<Q>,
    generation: u64,
    revoke: Arc<watch::Sender<bool>>,
}

impl<Q> Drop for Inbox<Q> {
    fn drop(&mut self) {
        // Calls still waiting on this frontend are refused, even if it already
        // took their questions out of the queue.
        let mut slot = self.frontend.inner.slot.lock().unwrap();
        let _ = self.revoke.send(true);
        if slot
            .as_ref()
            .is_some_and(|attached| attached.generation == self.generation)
        {
            *slot = None;
        }
    }
}

/// Why a question could not be handed to the frontend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    /// The frontend is not reading its questions.
    Full,
    /// The frontend has gone away.
    Closed,
}

/// A claim on the frontend that was current when it was taken.
pub struct Lease<Q> {
    sender: mpsc::Sender<Q>,
    generation: u64,
    /// Turns true when the frontend detaches or is replaced.
    pub revoked: watch::Receiver<bool>,
}

impl<Q> Lease<Q> {
    /// Hand `question` to the frontend without waiting for room.
    pub fn send(&self, question: Q) -> Result<(), Refusal> {
        self.sender.try_send(question).map_err(|error| match error {
            mpsc::error::TrySendError::Full(_) => Refusal::Full,
            mpsc::error::TrySendError::Closed(_) => Refusal::Closed,
        })
    }
}

impl<Q> Frontend<Q> {
    /// Become the frontend that answers questions. A later attach replaces an
    /// earlier one: the earlier inbox stops receiving, and calls still waiting on
    /// it are refused.
    pub fn attach(&self) -> Inbox<Q> {
        let (tx, rx) = mpsc::channel(QUEUE_CAPACITY);
        let generation = self.inner.next_generation.fetch_add(1, Ordering::SeqCst);
        let revoke = Arc::new(watch::channel(false).0);
        {
            // Replace and revoke under one lock hold: see the module docs.
            let mut slot = self.inner.slot.lock().unwrap();
            let previous = slot.replace(Attached {
                generation,
                questions: tx,
                revoke: Arc::clone(&revoke),
            });
            if let Some(previous) = previous {
                let _ = previous.revoke.send(true);
            }
        }
        Inbox {
            questions: rx,
            frontend: self.clone(),
            generation,
            revoke,
        }
    }

    /// Claim the current frontend, or `None` when none is attached.
    pub fn lease(&self) -> Option<Lease<Q>> {
        let slot = self.inner.slot.lock().unwrap();
        slot.as_ref().map(|attached| Lease {
            sender: attached.questions.clone(),
            generation: attached.generation,
            revoked: attached.revoke.subscribe(),
        })
    }

    /// Whether the frontend `lease` was taken from is still the current one.
    pub fn is_current(&self, lease: &Lease<Q>) -> bool {
        let slot = self.inner.slot.lock().unwrap();
        slot.as_ref()
            .is_some_and(|attached| attached.generation == lease.generation)
            && !*lease.revoked.borrow()
    }
}
