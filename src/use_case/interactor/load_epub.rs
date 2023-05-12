use crate::{
    domain::entity::{epub::Epub, epub_path::EpubPath},
    domain::repository::epub_repository::EpubRepository,
    use_case::interface::load_epub::LoadEpubUseCase,
};
use anyhow::Result;
use async_trait::async_trait;

pub struct LoadEpubInteractor<'a, R> {
    repository: &'a R,
}

impl<'a, R> LoadEpubInteractor<'a, R>
where
    R: EpubRepository,
{
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<'a, R> LoadEpubUseCase for LoadEpubInteractor<'a, R>
where
    R: EpubRepository + Sync + Send,
{
    async fn execute(&self, epub_path: &EpubPath) -> Result<Epub> {
        self.repository.load(epub_path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::epub_repository_impl::EpubRepositoryImpl;

    #[tokio::test]
    async fn load_epub_interactor_execute() -> Result<()> {
        let repository = EpubRepositoryImpl::new();
        let interactor = LoadEpubInteractor::new(&repository);
        let epub_path = EpubPath::new("resources/epub/essential-scala.epub");
        let epub = interactor.execute(&epub_path).await;
        assert!(epub.is_ok());
        Ok(())
    }
}
