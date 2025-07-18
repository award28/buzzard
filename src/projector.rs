use anyhow::Result;

/// A handler responsible for executing projections.
///
/// `Projector` types are used to apply external-facing projection logic,
/// such as updating read models, search indexes, analytics systems, or
/// triggering downstream effects like notifications. These operations
/// are triggered by `Projection` messages emitted from the policy layer
/// during event processing.
///
/// Projectors must be idempotent and side-effectful. Unlike command handlers,
/// they do not interact with the domain model and should not mutate domain aggregates.
pub trait Projector<P>: Clone + Sync + Send {
    /// Applies the given projection.
    ///
    /// The provided message contains all necessary data to perform the
    /// projection. This method should carry out the appropriate external-facing
    /// side effect (e.g., write to a search index or publish to an external service),
    /// and return a result indicating success or failure.
    ///
    /// Projection logic must be safe to retry and should not mutate domain state.
    fn project(&self, projection: P) -> impl Future<Output = Result<()>> + Send;
}
