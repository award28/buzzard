use crate::{driver::MessageBusDriver, handler::Command};

/// A top-level message envelope for routing through the message bus.
///
/// The `Message` enum represents the three primary types of messages that can
/// be processed by the message bus:
///
/// - `Command`: An intent to change domain state, handled via a `UnitOfWork`.
/// - `Event`: A fact that something has already happened, emitted by the domain and
///            passed to `Policy` implementations.
/// - `Projection`: A side-effect-only message used to update external systems,
///                 handled by a `Projector`.
///
/// This enum is used internally to represent all message types in transit across
/// the system. Each variant will be routed to the appropriate handler based on
/// its type.
pub enum Message<C, E, P>
where
    C: Send + Command,
    E: Send,
    P: Send,
{
    /// A command message, representing an intent to mutate domain state.
    Command(C),

    /// A domain event message, representing something that has occurred.
    Event(E),

    /// A projection message, representing a read-side update or infrastructure
    /// effect.
    Projection(P),
}

/// A type alias for a fully typed message handled by the message bus.
///
/// `DriverMessage` resolves the concrete command, event, and projection types
/// using the associated types provided by a `MessageBusDriver`. This alias is
/// used by the broker, dispatcher, and internal routing logic to work generically
/// across all message variants.
pub type DriverMessage<D> = Message<
    <D as MessageBusDriver>::Command,
    <D as MessageBusDriver>::Event,
    <D as MessageBusDriver>::Projection,
>;

/// A message produced as a side effect of a domain event.
///
/// A `SideEffect` is a result of applying a `Policy` to a domain event. It may
/// include one or more follow-up actions:
///
/// - `Command`: A new command to be handled by the domain.
/// - `Projection`: A projection message to be sent to an external system.
///
/// These side effects will be published to the message bus and routed as if they
/// had been received externally.
pub enum SideEffect<C, P>
where
    C: Send + Command,
    P: Send,
{
    /// A follow-up command to be executed by a command handler.
    Command(C),

    /// A projection to be handled by a projector.
    Projection(P),
}

pub type DriverSideEffect<D> =
    SideEffect<<D as MessageBusDriver>::Command, <D as MessageBusDriver>::Projection>;
