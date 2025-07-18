use crate::prelude::*;

/// Internal engine used to bootstrap and run a message bus.
///
/// `MessageBusEngine` is an internal support struct that wires together all
/// message bus components. It is not exposed outside the framework and should
/// only be constructed via `MessageBus::from(driver)`.
///
/// It provides factories and references for:
/// - Command execution (`UnitOfWork`)
/// - Policy application (`PolicyContext`)
/// - Message publishing (`Broker`)
/// - Projection execution (`Projector`)
///
/// This struct is intentionally minimal, and acts as a dependency container
/// for coordinating message flow at runtime.
pub struct MessageBusEngine<D: MessageBusDriver> {
    /// The concrete message bus driver that defines all associated components.
    pub driver: D,

    /// The message broker responsible for publishing and receiving messages.
    pub broker: D::Broker,

    /// The projector responsible for applying projection side effects.
    pub projector: D::Projector,

    pub handler: D::Handler,

    pub policy: D::Policy,

    pub viewer: D::Viewer,

    /// Factory to create a new policy context for each domain event.
    pub policy_context_factory: <D::PolicyContext as PolicyContext>::Factory,

    /// Factory to create a new unit of work for each command.
    pub uow_factory: <D::UnitOfWork as UnitOfWork>::Factory,
}

impl<D: MessageBusDriver> Clone for MessageBusEngine<D> {
    fn clone(&self) -> Self {
        Self {
            driver: self.driver.clone(),
            broker: self.broker.clone(),
            projector: self.projector.clone(),
            handler: self.handler.clone(),
            policy: self.policy.clone(),
            viewer: self.viewer.clone(),
            policy_context_factory: self.policy_context_factory.clone(),
            uow_factory: self.uow_factory.clone(),
        }
    }
}

impl<D: MessageBusDriver> From<&D> for MessageBusEngine<D>
where
    D::Broker: for<'a> From<&'a D>,
    D::Projector: for<'a> From<&'a D>,
    D::Handler: for<'a> From<&'a D>,
    D::Policy: for<'a> From<&'a D>,
    D::Viewer: for<'a> From<&'a D>,
    <D::UnitOfWork as UnitOfWork>::Factory: for<'a> From<&'a D>,
    <D::PolicyContext as PolicyContext>::Factory: for<'a> From<&'a D>,
{
    fn from(driver: &D) -> Self {
        Self {
            driver: driver.clone(),
            broker: From::from(driver),
            projector: From::from(driver),
            handler: From::from(driver),
            policy: From::from(driver),
            viewer: From::from(driver),
            policy_context_factory: From::from(driver),
            uow_factory: From::from(driver),
        }
    }
}
