use crate::domain::{
    entity::{
        epub::Epub,
        epub_path::{
            EpubPath,
            EpubPath::{LocalPath, NFTAddress, Url},
        },
    },
    repository::epub_repository::EpubRepository,
};
use crate::utils::zip_utils::open_zip;
use anyhow::{bail, Result};
use async_trait::async_trait;

#[derive(Debug, Copy, Clone)]
pub struct EpubRepositoryImpl {}

impl EpubRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl<'a> EpubRepository<'a> for EpubRepositoryImpl {
    async fn load(&self, epub_path: EpubPath<'a>) -> Result<Epub> {
        match epub_path {
            LocalPath(path) => {
                let zip = open_zip(path).await?;
                Epub::new(zip).await
            }
            Url(_) => {
                bail!("EpubPath::URL is unimplemented.");
            }
            NFTAddress(_) => {
                bail!("EpubPath::NFTAddress is unimplemented.");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[tokio::test]
    async fn epub_repository_load_from_local() -> Result<()> {
        let local_path = "resources/epub/essential-scala.epub";
        let epub_repo = EpubRepositoryImpl::new();
        let epub = epub_repo.load(LocalPath(local_path)).await;
        assert!(epub.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn epub_repository_load_from_url() -> Result<()> {
        let url = Url::parse("https://example.com/hogehoge/1.epub")?;
        let epub_repo = EpubRepositoryImpl::new();
        let epub = epub_repo.load(Url(&url)).await;
        assert!(epub.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn epub_repository_load_from_nft_address() -> Result<()> {
        let nft_address = "0x0123456789abcdef";
        let epub_repo = EpubRepositoryImpl::new();
        let epub = epub_repo.load(NFTAddress(nft_address)).await;
        assert!(epub.is_err());
        Ok(())
    }
}
