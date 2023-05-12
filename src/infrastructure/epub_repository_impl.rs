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
use std::ffi::OsStr;
use std::path::Path;

#[derive(Debug, Copy, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub struct EpubRepositoryImpl {}

impl EpubRepositoryImpl {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl EpubRepository for EpubRepositoryImpl {
    async fn list(&self, epub_path: &EpubPath) -> Result<Vec<EpubPath>> {
        match epub_path {
            LocalPath(local_path) => {
                let mut read_dir = tokio::fs::read_dir(local_path).await?;
                let mut epub_paths = vec![];
                while let Some(entry) = read_dir.next_entry().await? {
                    let file_type = match entry.file_type().await {
                        Ok(file_type) => file_type,
                        Err(_) => continue,
                    };
                    if !file_type.is_file() {
                        continue;
                    };

                    let path_buf = entry.path();
                    let path = Path::new(&path_buf);
                    let extension = path.extension().and_then(OsStr::to_str);
                    if let Some(_extension @ "epub") = extension {
                        epub_paths.push(LocalPath(path.to_string_lossy().to_string()));
                    }
                }
                Ok(epub_paths)
            }
            Url(_) => {
                bail!("EpubPath::URL is unimplemented.");
            }
            NFTAddress(_) => {
                bail!("EpubPath::NFTAddress is unimplemented.");
            }
        }
    }

    async fn load(&self, epub_path: &EpubPath) -> Result<Epub> {
        match epub_path {
            LocalPath(local_path) => {
                let zip = open_zip(local_path).await?;
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

    #[tokio::test]
    async fn epub_repository_load_from_local() -> Result<()> {
        let epub_repo = EpubRepositoryImpl::new();
        let epub_path = EpubPath::new("resources/epub/essential-scala.epub");
        let epub = epub_repo.load(&epub_path).await;
        assert!(epub.is_ok());
        Ok(())
    }

    #[tokio::test]
    #[should_panic]
    async fn epub_repository_load_from_url() {
        let epub_repo = EpubRepositoryImpl::new();
        let epub_path = EpubPath::new("https://example.com/hogehoge/1.epub");
        let epub = epub_repo.load(&epub_path).await;
        epub.unwrap();
    }

    #[tokio::test]
    #[should_panic]
    async fn epub_repository_load_from_nft_address() {
        let epub_repo = EpubRepositoryImpl::new();
        let epub_path = EpubPath::new("0x0123456789abcdef");
        let epub = epub_repo.load(&epub_path).await;
        epub.unwrap();
    }

    #[tokio::test]
    async fn epub_repository_list_from_local() -> Result<()> {
        let epub_repo = EpubRepositoryImpl::new();
        let epub_path = EpubPath::new("./resources/epub");
        let list = epub_repo.list(&epub_path).await;
        let actual = list.unwrap();
        assert_eq!(actual.len(), 1);
        Ok(())
    }

    #[tokio::test]
    #[should_panic]
    async fn epub_repository_list_from_url() {
        let epub_repo = EpubRepositoryImpl::new();
        let epub_path = EpubPath::new("https://example.com/hogehoge/1.epub");
        let list = epub_repo.list(&epub_path).await;
        list.unwrap();
    }

    #[tokio::test]
    #[should_panic]
    async fn epub_repository_list_from_nft_address() {
        let epub_repo = EpubRepositoryImpl::new();
        let epub_path = EpubPath::new("0x0123456789abcdef");
        let list = epub_repo.list(&epub_path).await;
        list.unwrap();
    }
}
