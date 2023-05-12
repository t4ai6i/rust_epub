use crate::domain::entity::{epub::Epub, epub_path::EpubPath};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait EpubRepository {
    async fn list(&self, epub_path: &EpubPath) -> Result<Vec<EpubPath>>;
    async fn load(&self, epub_path: &EpubPath) -> Result<Epub>;
}
