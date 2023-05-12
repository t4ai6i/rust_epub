use crate::{
    domain::{entity::epub_path::EpubPath, repository::epub_repository::EpubRepository},
    use_case::interface::list_epub::ListEpubUseCase,
};
use anyhow::Result;
use async_trait::async_trait;

pub struct ListEpubInteractor<'a, R> {
    repository: &'a R,
}

impl<'a, R> ListEpubInteractor<'a, R>
where
    R: EpubRepository,
{
    pub fn new(repository: &'a R) -> Self {
        Self { repository }
    }
}

#[async_trait]
impl<'a, R> ListEpubUseCase for ListEpubInteractor<'a, R>
where
    R: EpubRepository + Sync + Send,
{
    async fn execute(&self, epub_path: &EpubPath) -> Result<Vec<EpubPath>> {
        self.repository.list(epub_path).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::epub_repository_impl::EpubRepositoryImpl;
    use anyhow::Result;

    #[tokio::test]
    async fn list_epub_interactor_execute() -> Result<()> {
        let repository = EpubRepositoryImpl::new();
        let interactor = ListEpubInteractor::new(&repository);
        let epub_path = EpubPath::new("./resources/epub");
        let actual = interactor.execute(&epub_path).await?;
        assert_eq!(actual.len(), 1);
        Ok(())
    }
}
