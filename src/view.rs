use anyhow::Result;
use serde::{Deserialize, Serialize};

pub trait Query: for<'de> Deserialize<'de> + Send + Sync {}
impl<T: for<'de> Deserialize<'de> + Send + Sync> Query for T {}

pub trait View: Serialize {}
impl<T: Serialize> View for T {}

pub trait Viewer<Q: Query> {
    fn view(&self, query: Q) -> impl Future<Output = Result<impl View>> + Send;
}
