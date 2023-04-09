use crate::domain::entity::{epub::Epub, epub_path::EpubPath};
use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait LoadEpubUseCase<'a> {
    async fn execute(&self, epub_path: EpubPath<'a>) -> Result<Epub>;
}
