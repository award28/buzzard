use anyhow::Result;

pub trait Factory: Send + Sync {
    type Output: Send;

    fn create(&self) -> impl Future<Output = Result<Self::Output>> + Send;
}
