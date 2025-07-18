//! # Message Bus Framework
//!
//! This module provides the core runtime abstraction for processing domain messages
//! in a message-driven system, including commands, events, and projections.
//!
//! The [`MessageBus`] struct is the primary entrypoint into the system, coordinating:
//! - Command execution and domain mutation via [`CommandHandler`]
//! - Event-driven workflows via [`Policy`]
//! - Read-model and infrastructure updates via [`Projector`]
//!
//! ## Core Features
//! - Declarative architecture powered by `MessageBusDriver`
//! - Full separation of write-side (commands) and read-side (projections)
//! - Async-safe, `Send + Sync + Clone` for multi-threaded environments
//! - Broker-agnostic: plug in Redis, Postgres, NATS, or any custom queue
//!
//! ## Runtime Behavior
//! - Commands are executed transactionally via a [`UnitOfWork`]
//! - Events emitted from commands are routed to a [`Policy`] that maps them to side effects
//! - Side effects may produce additional commands or projections
//! - Projections are dispatched to external systems for read-model updates or notifications
//!
//! ## Example
//! ```rust
//! use anyhow::Result;
//! use my_app::driver::MyDriver;
//! use my_framework::bus::MessageBus;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     // Build your domain driver (includes broker, projector, and repo factories)
//!     let driver = MyDriver::init().await?;
//!
//!     // Construct the message bus from the driver
//!     let bus = MessageBus::from(&driver);
//!
//!     // Clone the bus and run it on a background task
//!     let background = tokio::spawn(bus.clone().start());
//!
//!     // Dispatch a command manually from an API layer, CLI, or test
//!     let cmd = MyCommand { sku: "ABC-123".into() };
//!     let response = bus.dispatch(cmd).await?;
//!
//!     background.await??;
//!     Ok(())
//! }
//! ```
//!
//! ## When to Use
//! Use `MessageBus` when you want a strongly typed, message-oriented runtime that:
//! - Enforces clear command → event → side effect flow
//! - Supports declarative orchestration via policies
//! - Separates domain logic from infrastructure concerns
//!
//! This module forms the backbone of the message-based execution model used across your system.

mod engine;

pub mod broker;
pub mod bus;
pub mod driver;
pub mod factory;
pub mod handler;
pub mod message;
pub mod policy;
pub mod prelude;
pub mod projector;
pub mod uow;
pub mod view;
