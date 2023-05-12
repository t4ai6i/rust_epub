use crate::domain::entity::epub_path::EpubPath;
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait ListEpubUseCase {
    async fn execute(&self, epub_path: &EpubPath) -> Result<Vec<EpubPath>>;
}
