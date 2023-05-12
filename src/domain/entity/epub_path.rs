use crate::domain::entity::epub_path::EpubPath::{LocalPath, Url};
use url;

#[derive(Debug, Clone, Hash, PartialOrd, PartialEq)]
pub enum EpubPath {
    LocalPath(String),
    Url(url::Url),
    NFTAddress(String),
}

impl EpubPath {
    pub fn new(path: impl Into<String>) -> EpubPath {
        let path = path.into();
        if let Ok(url) = url::Url::parse(&path) {
            Url(url)
        } else {
            LocalPath(path)
        }
        // TODO: NFTAddressの判定と生成
    }
}
