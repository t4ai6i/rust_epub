use std::fs::File;
use std::io::Read;
use std::path::Path;
use tokio::fs::File as TokioFile;
use zip::ZipArchive;
use zip::result::ZipResult;
use anyhow::Result;

/// open zip archive file
pub async fn open_zip(path: impl AsRef<Path>) -> ZipResult<ZipArchive<File>> {
    let epub = TokioFile::open(path).await?;
    let epub = epub.into_std().await;
    ZipArchive::new(epub)
}

/// read zip content to string
pub fn read_to_string(epub: &mut ZipArchive<File>, path: &str) -> Result<String> {
    let content = &mut epub.by_name(path)?;
    let mut buffer = String::new();
    content.read_to_string(&mut buffer)?;
    Ok(buffer)
}

/// read zip content to Vec<u8>
pub fn read_to_end(epub: &mut ZipArchive<File>, path: &str) -> Result<Vec<u8>> {
    let content = &mut epub.by_name(path)?;
    let mut buffer = Vec::new();
    content.read_to_end(&mut buffer)?;
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn success_open_zip() {
        let path = "resources/epub/essential-scala.epub";
        let archive = open_zip(path).await.unwrap();
        // for i in 0..archive.len() {
        //     let file = archive.by_index(i).unwrap();
        //     dbg!(file.name());
        // }
        assert_eq!(archive.len(), 34);
    }

    #[tokio::test]
    async fn failure_open_zip() {
        let path = "nothing.epub";
        let archive = open_zip(path).await;
        assert!(archive.is_err());
    }

    #[tokio::test]
    async fn success_read_to_string() {
        let path = "resources/epub/essential-scala.epub";
        let mut archive = open_zip(path).await.unwrap();
        let path = "META-INF/container.xml";
        let actual = read_to_string(&mut archive, path).unwrap();
        assert_eq!(actual, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<container version=\"1.0\" xmlns=\"urn:oasis:names:tc:opendocument:xmlns:container\">\n  <rootfiles>\n    <rootfile full-path=\"content.opf\" media-type=\"application/oebps-package+xml\" />\n  </rootfiles>\n</container>");
    }

    #[tokio::test]
    async fn failure_read_to_string() {
        let path = "resources/epub/essential-scala.epub";
        let mut archive = open_zip(path).await.unwrap();
        let path = "nothing";
        let actual = read_to_string(&mut archive, path);
        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn success_read_to_end() {
        let path = "resources/epub/essential-scala.epub";
        let mut archive = open_zip(path).await.unwrap();
        let path = "media/epub-cover.png";
        let actual = read_to_end(&mut archive, path).unwrap();
        assert_eq!(actual.len(), 28932);
    }

    #[tokio::test]
    async fn failure_read_to_end() {
        let path = "resources/epub/essential-scala.epub";
        let mut archive = open_zip(path).await.unwrap();
        let path = "nothing";
        let actual = read_to_end(&mut archive, path);
        assert!(actual.is_err());
    }
}