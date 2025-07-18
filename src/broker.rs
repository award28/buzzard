use anyhow::Result;
use futures::stream::Stream;

/// A trait for implementing message transport across the message bus.
///
/// `MessageBroker` abstracts over the mechanics of sending and receiving
/// messages in a queue-based or event-driven system. It is responsible for
/// delivering incoming messages to the bus and acknowledging whether those
/// messages were processed successfully or should be retried.
///
/// This trait does not impose any requirements on the underlying transport,
/// making it suitable for a wide range of messaging systems (e.g., Redis,
/// NATS, Kafka, or Postgres-based queues).
pub trait MessageBroker: Clone + Send + Sync {
    /// The message type sent and received over the broker.
    ///
    /// This message will typically represent a `Command`, `Projection`, or
    /// `Event` wrapped in an envelope (e.g. `DriverMessage<D>`). The broker
    /// does not inspect the message, but passes it through to the message bus.
    type Message: Send;

    /// A unique identifier for a delivered message.
    ///
    /// This identifier is used to acknowledge (`ack`) or reject (`nack`)
    /// the message after processing. The identifier must be unique per
    /// message and persistable across retries if necessary.
    type Id: Send;

    /// A stream of incoming messages to be processed by the message bus.
    ///
    /// This method returns a `Stream` of `(Id, Message)` pairs. Each message
    /// received should be processed, then acknowledged or negatively acknowledged
    /// by calling `ack` or `nack` respectively.
    ///
    /// This stream should be infinite (or long-lived), and drive the core
    /// consumption loop of the message bus.
    fn receiver(&self) -> impl Stream<Item = (Self::Id, Self::Message)> + Send;

    /// Publish a single message to be processed asynchronously.
    ///
    /// The message will be queued and delivered to the receiver at some point
    /// in the future. This method does not wait for the message to be executed,
    /// only for it to be successfully published.
    fn publish(&self, message: Self::Message) -> impl Future<Output = Result<()>> + Send;

    /// Publish a batch of messages to be processed asynchronously.
    ///
    /// All messages will be enqueued and delivered independently. This method
    /// is often more efficient than publishing individual messages if supported
    /// by the underlying transport.
    fn publish_batch(&self, message: Vec<Self::Message>)
    -> impl Future<Output = Result<()>> + Send;

    /// Acknowledge successful processing of a previously received message.
    ///
    /// This signals to the broker that the message has been handled and should
    /// not be redelivered. Some brokers may use this to remove the message from
    /// persistence or checkpoint consumer offsets.
    fn ack(&self, id: Self::Id) -> impl Future<Output = Result<()>> + Send;

    /// Negatively acknowledge a message that failed during processing.
    ///
    /// This signals to the broker that the message was not successfully handled,
    /// and should be retried or moved to a dead-letter queue depending on the
    /// broker configuration.
    fn nack(&self, id: Self::Id) -> impl Future<Output = Result<()>> + Send;
}
