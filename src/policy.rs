use crate::{driver::MessageBusDriver, factory::Factory};
use anyhow::Result;

/// Provides read-only access to domain state for a `Policy`.
///
/// `PolicyContext` exists to support the evaluation of policies in response
/// to domain events. It grants read-only access to repositories or domain
/// views, allowing policies to determine follow-up behavior (e.g., additional
/// commands or projection messages).
///
/// Unlike a `UnitOfWork`, this context must not allow any mutation of domain
/// aggregates or emit events. It is created once per domain event and discarded
/// after the policy has completed.
// NOTE: The use of `Context` here isn't exactly correct. A Context
// type should not have access to a repository. However, for our purposes,
// we're going to use this until we can determine a more accurate name/pattern.
pub trait PolicyContext: Send {
    /// A factory used to construct new instances of `PolicyContext`.
    ///
    /// A fresh `PolicyContext` is created for every event that is handled
    /// by the message bus. The factory should be stateless and inexpensive
    /// to clone.
    type Factory: Factory<Output = Self> + Clone;

    /// Finalize and clean up the policy context.
    ///
    /// Called after policy application is complete. This gives implementations
    /// a chance to release connections, clean up internal state, or perform
    /// post-read cleanup. It is guaranteed to be called once per context.
    fn close(self) -> impl Future<Output = Result<()>> + Send;
}

/// A stateless rule that reacts to domain events.
///
/// `Policy` types define declarative, stateless business logic that reacts
/// to domain events and produces a set of downstream side effects, such as
/// new commands or projection messages.
///
/// Policies must not modify domain state. They are evaluated using a
/// `PolicyContext` that provides read-only access to domain data. The
/// returned messages will later be published to the broker by the message
/// bus, where they can be handled by a command handler or a projector.
///
/// Policies are often used to express cross-aggregate or workflow rules
/// that arise as consequences of a domain event.
pub trait Policy<E: Send, D: MessageBusDriver>: Clone + Send + Sync {
    /// The side effect message type returned by this policy.
    ///
    /// This is typically a wrapper enum such as `SideEffect<Command, Projection>`,
    /// but may be any message bus-compatible type.
    type Output: Send;

    /// Apply this policy to the given domain event using the provided context.
    ///
    /// This function must return a list of zero or more messages that should
    /// be emitted in response to the event. These may include commands (to be
    /// processed by a command handler) or projections (to be handled by a projector).
    ///
    /// This function must not mutate domain state and should only perform reads
    /// using the provided context.
    fn apply(
        &self,
        ctx: &mut D::PolicyContext,
        event: E,
    ) -> impl Future<Output = Result<Vec<Self::Output>>> + Send;
}
