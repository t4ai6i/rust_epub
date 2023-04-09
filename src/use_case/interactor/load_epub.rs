use crate::{
    domain::entity::{epub::Epub, epub_path::EpubPath},
    domain::repository::epub_repository::EpubRepository,
    use_case::interface::load_epub::LoadEpubUseCase,
};
use anyhow::Result;
use async_trait::async_trait;

pub struct LoadEpubInteractor<R> {
    repository: R,
}

impl<'a, R> LoadEpubInteractor<R>
where
    R: EpubRepository<'a>,
{
    pub fn new(repository: R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<'a: 'b, 'b, R> LoadEpubUseCase<'a> for LoadEpubInteractor<R>
where
    R: EpubRepository<'b> + Sync + Send,
{
    async fn execute(&self, epub_path: EpubPath<'a>) -> Result<Epub> {
        self.repository.load(epub_path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::entity::epub_path::EpubPath::LocalPath;
    use crate::infrastructure::epub_repository_impl::EpubRepositoryImpl;

    #[tokio::test]
    async fn load_epub_interactor_execute() -> Result<()> {
        let epub_repository = EpubRepositoryImpl::new();
        let epub_interactor = LoadEpubInteractor::new(epub_repository);
        let local_path = "resources/epub/essential-scala.epub";
        let epub_path = LocalPath(local_path);
        let epub = epub_interactor.execute(epub_path).await;
        assert!(epub.is_ok());
        Ok(())
    }
}
