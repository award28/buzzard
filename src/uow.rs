use crate::factory::Factory;
use anyhow::Result;

/// A transactional boundary for domain mutation.
///
/// `UnitOfWork` represents a scoped, transactional unit in which domain
/// operations (such as command handling) are performed. It ensures that
/// all changes made during its lifetime are either committed atomically
/// or rolled back on failure.
///
/// Any events produced during the unit of work should be captured using
/// `capture_event`. These events will be published only if the unit of
/// work is successfully committed.
pub trait UnitOfWork: Send {
    /// A factory used to produce new `UnitOfWork` instances.
    ///
    /// This allows the message bus to dynamically construct a new unit
    /// of work for each command processed. The factory must be cheap
    /// to clone and stateless in practice.
    type Factory: Factory<Output = Self> + Clone;

    /// The domain event type emitted by this unit of work.
    ///
    /// All captured events must implement this type. Events are collected
    /// during domain operations and returned from `commit` upon success.
    type Event: Send;

    /// Capture an event that occurred during the current unit of work.
    ///
    /// This should be called whenever a domain aggregate emits an event
    /// (e.g., via a method like `aggregate.record_event(...)`). Events
    /// captured here are retained until `commit` or discarded on failure.
    ///
    /// This method must never publish the event directly — events are only
    /// published by the message bus after a successful commit.
    // TODO: This shouldn't be allowed to throw an error.
    fn capture_event(&mut self, event: impl Into<Self::Event>) -> Result<()>;

    /// Commit all changes made within the unit of work.
    ///
    /// This is called once command handling is complete and no errors occurred.
    /// All pending domain state changes will be persisted atomically. Upon
    /// success, the unit of work will return the captured domain events, which
    /// will then be published by the message bus.
    ///
    /// If the commit fails, the caller is responsible for error handling;
    /// events must not be published unless commit succeeds.
    fn commit(self) -> impl Future<Output = Result<Vec<Self::Event>>> + Send;

    /// Roll back all changes made within the unit of work.
    ///
    /// This is called if command handling fails. All changes made to domain
    /// state will be discarded, and no events will be published.
    ///
    /// This function must leave the system in the same state as before the
    /// unit of work was created.
    fn rollback(self) -> impl Future<Output = Result<()>> + Send;
}
