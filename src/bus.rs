use std::any::type_name;

use anyhow::Result;
use futures::{StreamExt, pin_mut};

use crate::{
    engine::MessageBusEngine,
    prelude::*,
    view::{Query, View, Viewer},
};

/// A runtime processor for command, event, and projection messages.
///
/// `MessageBus` is the central component of the framework responsible for
/// dispatching messages, coordinating transactional boundaries, applying
/// domain policies, and executing projection side effects.
///
/// You can construct a `MessageBus` from any type that implements
/// [`MessageBusDriver`]. The driver provides the associated types needed
/// for orchestration — including your domain-specific `Command`, `Event`,
/// and `Projection` types, along with the backing broker, UoW, and policy logic.
///
/// # Example
///
/// ```rust
/// let bus = MessageBus::from(&MyAppDriver);
/// bus.start().await?;
/// ```
///
/// # Message Lifecycle
///
/// The message bus supports the following message types:
///
/// - **Command**: Triggers domain mutation inside a [`UnitOfWork`].
/// - **Event**: Emitted by successful commands; passed to a [`Policy`] to derive
/// follow-up actions.
/// - **Projection**: Infrastructure-facing side effect messages handled by a
/// [`Projector`].
///
/// All message types are received via a [`MessageBroker`], dispatched internally,
/// and acknowledged or retried based on their outcome.
///
/// The bus guarantees:
/// - Atomic command processing via the UoW
/// - Stateless and safe policy application
/// - Separation of domain mutation from infrastructure projection
///
/// # Traits You Must Implement
///
/// - [`MessageBusDriver`]: Defines your domain message types and supporting
///     components.
/// - [`CommandHandler<C, D>`]: Implements logic for your commands.
/// - [`Policy<Event, D>`]: Maps events to follow-up commands and projections.
/// - [`Projector<Projection, D>`]: Applies projections to external systems.
///
/// Once your driver and handlers are in place, just call [`start()`] to begin
/// processing.
pub struct MessageBus<D: MessageBusDriver> {
    engine: MessageBusEngine<D>,
}

impl<D: MessageBusDriver> Clone for MessageBus<D> {
    fn clone(&self) -> Self {
        Self {
            engine: self.engine.clone(),
        }
    }
}

impl<D> From<&D> for MessageBus<D>
where
    D: MessageBusDriver,
    D::Broker: for<'a> From<&'a D>,
    D::Projector: for<'a> From<&'a D>,
    D::Handler: for<'a> From<&'a D>,
    D::Policy: for<'a> From<&'a D>,
    D::Viewer: for<'a> From<&'a D>,
    <D::PolicyContext as PolicyContext>::Factory: for<'a> From<&'a D>,
    <D::UnitOfWork as UnitOfWork>::Factory: for<'a> From<&'a D>,
{
    fn from(driver: &D) -> Self {
        let engine = MessageBusEngine::from(driver);
        Self { engine }
    }
}

impl<D: MessageBusDriver> MessageBus<D> {
    /// Dispatch a command for immediate execution.
    ///
    /// The provided command is handled by the corresponding `CommandHandler`,
    /// using a fresh `UnitOfWork` for transaction isolation. On success,
    /// any captured domain events are published to the message bus. If
    /// command handling or commit fails, the unit of work is rolled back
    /// and the error is returned.
    ///
    /// This method is primarily used to execute commands from within an
    /// application service, CLI, or HTTP controller.
    pub async fn dispatch<C: Command>(&self, cmd: C) -> Result<Option<D::Identifier>>
    where
        D::Handler: CommandHandler<C, D>,
    {
        println!("User provided command: {}", type_name::<C>());
        let mut uow = self.engine.uow_factory.create().await?;
        match self.engine.handler.handle(&mut uow, cmd).await {
            Ok(res) => {
                let events = uow
                    .commit()
                    .await?
                    .into_iter()
                    .map(DriverMessage::<D>::Event)
                    .collect();
                self.engine.broker.publish_batch(events).await?;
                Ok(res)
            }
            Err(e) => {
                uow.rollback().await?;
                Err(e)
            }
        }
    }

    pub async fn view<Q: Query>(&self, query: Q) -> Result<impl View>
    where
        D::Viewer: Viewer<Q>,
    {
        self.engine.viewer.view(query).await
    }

    /// Starts the message bus processing loop.
    ///
    /// This continuously receives messages from the message broker, routes
    /// them to the appropriate handler (command, event, or projection), and
    /// acknowledges them based on the result.
    ///
    /// This function should be run for the duration of the application
    /// lifecycle — typically as a background task or top-level service.
    pub async fn start(self) -> Result<()>
    where
        D::Handler: CommandHandler<D::Command, D>,
        D::Policy: Policy<D::Event, D, Output = SideEffect<D::Command, D::Projection>>,
    {
        let stream = self.engine.broker.receiver();
        pin_mut!(stream);
        while let Some((id, msg)) = stream.next().await {
            match self.handle_message(msg).await {
                Ok(_) => {
                    self.engine.broker.ack(id).await?;
                    println!("Handled message successfully.");
                }
                Err(e) => {
                    self.engine.broker.nack(id).await?;
                    println!("Handled message unsuccessfully: {e:#?}");
                }
            };
        }
        Ok(())
    }

    /// Routes an incoming message to its corresponding handler.
    ///
    /// This internal function dispatches commands, executes projections, or
    /// applies event policies depending on the message variant.
    async fn handle_message(&self, msg: DriverMessage<D>) -> Result<()>
    where
        D::Handler: CommandHandler<D::Command, D>,
        D::Policy: Policy<D::Event, D, Output = SideEffect<D::Command, D::Projection>>,
    {
        match msg {
            Message::Command(cmd) => {
                println!("Executing command");
                self.dispatch(cmd).await?;
            }
            Message::Event(event) => {
                println!("Executing event");
                self.handle_event(event).await?;
            }
            Message::Projection(projection) => {
                println!("Executing projection");
                self.engine.projector.project(projection).await?;
            }
        };
        Ok(())
    }

    /// Handles a domain event by applying the associated policy.
    ///
    /// A new `PolicyContext` is created for the event, and the policy is
    /// applied using the event data. The resulting side effects (commands
    /// and/or projections) are then published back to the message bus.
    ///
    /// The context is closed after handling, even if the policy fails.
    async fn handle_event(&self, event: D::Event) -> Result<()>
    where
        D::Policy: Policy<D::Event, D, Output = SideEffect<D::Command, D::Projection>>,
    {
        let mut ctx = self.engine.policy_context_factory.create().await?;
        let res = match self.engine.policy.apply(&mut ctx, event).await {
            Ok(events) => {
                let messages = events
                    .into_iter()
                    .map(|side_effect| match side_effect {
                        SideEffect::Command(cmd) => Message::Command(cmd),
                        SideEffect::Projection(proj) => Message::Projection(proj),
                    })
                    .collect::<Vec<_>>();
                let num_events = messages.len();
                self.engine.broker.publish_batch(messages).await?;
                println!("Published {num_events} events.");
                Ok(())
            }
            Err(e) => Err(e),
        };

        ctx.close().await?;
        res
    }
}
