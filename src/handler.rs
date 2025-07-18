use crate::driver::MessageBusDriver;
use anyhow::Result;

/// Represents the response type of a command.
///
/// This trait should be implemented by all types used as commands within the
/// message bus system. Each command defines a corresponding `Response` type,
/// which will be returned when the command is successfully handled.
///
/// [`Command`] allows the system to reason about command handling in a
/// generic way without requiring dynamic dispatch or concrete knowledge of
/// the return type.
pub trait Command: Send {}
impl<T: Send> Command for T {}

/// A handler responsible for executing commands.
///
/// Implementors of this trait define the logic for processing a specific
/// `Command` within the context of a `UnitOfWork`. The `handle` method is
/// provided a mutable reference to the `UnitOfWork`, allowing full access
/// to domain state and repositories for the duration of the command lifecycle.
///
/// This method must return a future that resolves to either a successful
/// result of type `C::Response` or a failure, which will cause the unit of
/// work to be rolled back.
///
/// Command handlers are only invoked during the command execution phase
/// of the message bus.
pub trait CommandHandler<C: Command, D: MessageBusDriver>: Clone + Send + Sync {
    /// Handle a command using the given unit of work.
    ///
    /// This function should perform any necessary domain logic, mutate
    /// aggregates, and register domain events as needed.
    ///
    /// If the command is successful, the `UnitOfWork` will be committed.
    /// If an error is returned, the `UnitOfWork` will be rolled back.
    fn handle(
        &self,
        uow: &mut D::UnitOfWork,
        cmd: C,
    ) -> impl Future<Output = Result<Option<D::Identifier>>> + Send;
}
