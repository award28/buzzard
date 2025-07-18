use crate::{
    broker::MessageBroker,
    handler::{Command, CommandHandler},
    message::{DriverMessage, DriverSideEffect},
    policy::{Policy, PolicyContext},
    projector::Projector,
    uow::UnitOfWork,
};

pub trait Projection: Send {}
impl<T: Send> Projection for T {}

pub trait Event: Send {}
impl<T: Send> Event for T {}

/// A message bus driver.
///
/// This trait encapsulates a complete set of types and implementations
/// required to operate a fully functional message bus. It defines how
/// commands are handled, events are mapped to follow-up messages, and
/// projections are performed. Each type acts as a plug-in point for
/// application-specific behavior, ensuring consistent orchestration of
/// domain interactions across command, event, and projection flows.
pub trait MessageBusDriver: Clone + Sized + Send + Sync + 'static {
    // Used to identify something.
    type Identifier: Send;

    /// The domain-specific `Command` type for this message bus.
    ///
    /// `Command` types represent intent to change domain state and are
    /// handled by a `CommandHandler` implemented on the message bus.
    /// Each `Command` must implement the [`Command`] trait, which
    /// provides a way to represent a standardized response when the
    /// command completes.
    ///
    /// Commands are dispatched by the message bus and routed to the
    /// corresponding handler that will apply changes to the domain model.
    type Command: Command;

    /// The domain-specific `Event` type for this message bus.
    ///
    /// `Event` types are emitted by domain logic as a result of
    /// command execution and represent things that have already
    /// happened. Events are immutable and are used to drive follow-up
    /// behavior, such as further commands and/or projections.
    ///
    /// Each `Event` will be applied to a `Policy` defined on the message
    /// bus. Once applied, the `Policy` will determine what downstream side
    /// effects should occur.
    type Event: Event;

    /// The domain-specific `Projection` type for this message bus.
    ///
    /// `Projection` types represent read-side or infrastructure-focused
    /// changes that are derived from domain events. These are intended
    /// to be side effects such as updating a search index, sending
    /// notifications, or syncing to an external system.
    ///
    /// Each `Projection` will be processed by a `Projector` implementation,
    /// which performs the actual infrastructure-facing update logic.
    type Projection: Projection;

    /// The concrete `MessageBroker` implementation for this message bus.
    ///
    /// The `Broker` is responsible for routing messages to the message
    /// bus, publishing events, and exposing methods for acknowledging
    /// success or failuire after message processing. It serves as the
    /// transport layer between your application and the message pipeline.
    type Broker: MessageBroker<Message = DriverMessage<Self>>;

    /// The concrete `UnitOfWork` implementation for this message bus.
    ///
    /// The `UnitOfWork` has a `Factory` used to build a new `UnitOfWork`
    /// for each `Command` received from the message bus. The resulting
    /// `UnitOfWork` provides transactional access to repositories and
    /// domain state for command execution.
    ///
    /// Once complete, the `UnitOfWork` is responsible for committing the
    /// resulting changes and emitting any generated domain events.
    type UnitOfWork: UnitOfWork<Event = Self::Event>;

    /// The concrete `PolicyContext` implementation for this message bus.
    ///
    /// The `PolicyContext` provides read-only access to domain data
    /// and is constructed for each `Event` received from the message
    /// bus. It is passed to the `Policy` implementation responsible for
    /// generating downstream side effects (e.g., additional commands or
    /// projection messages).
    ///
    /// Unlike the `UnitOfWork`, this context must not allow domain mutation.
    type PolicyContext: PolicyContext;

    /// The concrete `Projector` implementation for this message bus.
    ///
    /// The `Projector` is responsible for handling each `Projection`
    /// received from the message bus. It applies infrastructure-facing
    /// updates based on previously emitted projection messages.
    ///
    /// This includes operations such as updating read models, search
    /// indexes, external integrations, or emitting telemetry events.
    type Projector: Projector<Self::Projection>;

    type Handler: CommandHandler<Self::Command, Self>;

    type Policy: Policy<Self::Event, Self, Output = DriverSideEffect<Self>>;

    type Viewer: Clone + Send + Sync;
}
