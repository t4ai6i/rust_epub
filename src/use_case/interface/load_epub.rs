use crate::domain::entity::{epub::Epub, epub_path::EpubPath};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait LoadEpubUseCase {
    async fn execute(&self, epub_path: &EpubPath) -> Result<Epub>;
}
