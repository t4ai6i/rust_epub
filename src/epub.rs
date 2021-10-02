use std::path::Path;
use std::fs::File;
use anyhow::{Result, Context, ensure, bail};
use zip::ZipArchive;
use crate::content_opf::ContentOpf;
use crate::spine::Spine;
use crate::manifest::Manifest;
use crate::zip_utils::{open_zip, read_to_string};

pub struct Epub {
    pub body: ZipArchive<File>,
    pub content_opf: ContentOpf,
}

impl Epub {
    /// Create Epub
    pub async fn new(path: impl AsRef<Path>) -> Result<Self> {
        let mut epub = open_zip(path).await?;
        // for i in 0..epub.len() {
        //     let file = epub.by_index(i)?;
        //     dbg!(file.name());
        // }
        let content_opf_path = ContentOpf::content_opf_path(&mut epub)?;
        let content_opf = read_to_string(&mut epub, &content_opf_path)?;
        let content_opf = ContentOpf::new(&content_opf)?;
        Ok(Epub {
            body: epub,
            content_opf,
        })
    }

    pub fn read_content_by_index_of_spine(&mut self, spine_index: usize) -> Result<String> {
        let spine = self.spine_by_index(spine_index)?;
        let manifest = self.manifest_by_spine(spine)?;
        let href = match &manifest.href {
            Ok(v) => v.to_string(),
            Err(e) => bail!(format!("Invalid Manifest.href. {:?}", e)),
        };
        read_to_string(&mut self.body, &href)
    }

    /// Returns length of spines
    fn spines_len(&self) -> usize {
        self.content_opf.spines.len()
    }

    /// Returns the Spine by index
    fn spine_by_index(&self, index: usize) -> Result<&Spine> {
        self.content_opf.spines.iter()
            .nth(index)
            .with_context(|| format!("Index out of bounds. {}/{}", index, self.content_opf.spines.len()))
    }

    /// Returns the manifest by Spine
    fn manifest_by_spine(&self, spine: &Spine) -> Result<&Manifest> {
        ensure!(
            spine.idref.as_ref().is_ok(),
            format!("Spine.idref is error. {}", spine.idref.as_ref().err().unwrap())
        );

        let idref = spine.idref.as_ref().unwrap();
        self.content_opf.manifests.iter()
            .find(|manifest| manifest.id.is_ok() && manifest.id.as_ref().unwrap().eq(idref))
            .with_context(|| format!("not found idref. {}", idref))
    }

    fn manifest_by_href(&self, href: &str) -> Result<&Manifest> {
        self.content_opf.manifests.iter()
            .find(|manifest| manifest.href.is_ok() && manifest.href.as_ref().unwrap().eq(href))
            .with_context(|| format!("not found href. {}", href))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use roxmltree::Document;
    use anyhow::anyhow;
    use mime::Mime;
    use std::str::FromStr;

    #[tokio::test]
    async fn read_cover_xhtml() {
        let mut epub = Epub::new("tests/resources/essential-scala.epub").await.unwrap();
        let cover_xhtml =  epub.read_content_by_index_of_spine(0).unwrap();
        let doc = Document::parse(&cover_xhtml).unwrap();
        let root = doc.root_element();
        let body = root.children()
            .filter(|node| node.is_element() && node.tag_name().name().eq("body"))
            .nth(0)
            .unwrap();
        for child in body.children() {
            dbg!(&child);
        }
        assert!(true);
    }

    #[tokio::test]
    async fn success_epub_new() {
        let actual = Epub::new("tests/resources/essential-scala.epub").await;
        assert!(actual.is_ok());
        assert_eq!(actual.unwrap().spines_len(), 15);
    }

    #[tokio::test]
    async fn failure_epub_new() {
        let actual = Epub::new("non_existing.epub").await;
        assert!(actual.is_err());
    }

    #[tokio::test]
    async fn success_read_content_by_index_of_spine() {
        let mut epub = Epub::new("tests/resources/essential-scala.epub").await.unwrap();
        let actual =  epub.read_content_by_index_of_spine(0);
        assert!(actual.is_ok());
        let actual = actual.unwrap();
        assert_eq!(actual, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<!DOCTYPE html PUBLIC \"-//W3C//DTD XHTML 1.1//EN\" \"http://www.w3.org/TR/xhtml11/DTD/xhtml11.dtd\">\n<html xmlns=\"http://www.w3.org/1999/xhtml\">\n<head>\n  <meta http-equiv=\"Content-Type\" content=\"text/html; charset=utf-8\" />\n  <meta http-equiv=\"Content-Style-Type\" content=\"text/css\" />\n  <meta name=\"generator\" content=\"pandoc\" />\n  <title>Essential Scala</title>\n  <link rel=\"stylesheet\" type=\"text/css\" href=\"stylesheet.css\" />\n</head>\n<body>\n<div id=\"cover-image\">\n<img src=\"media/epub-cover.png\" alt=\"cover image\" />\n</div>\n</body>\n</html>\n\n");
    }

    #[tokio::test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: Index out of bounds. 100/15")]
    async fn failure_read_content_by_index_of_spine() {
        let mut epub = Epub::new("tests/resources/essential-scala.epub").await.unwrap();
        epub.read_content_by_index_of_spine(100).unwrap();
    }

    #[tokio::test]
    async fn success_spine_by_index() {
        let epub = Epub::new("tests/resources/essential-scala.epub").await.unwrap();
        let spine = epub.spine_by_index(0).unwrap();
        let actual = spine.idref.as_ref().unwrap();
        assert_eq!(actual, "cover_xhtml");
    }

    #[tokio::test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: Index out of bounds. 100/15")]
    async fn failure_spine_by_index() {
        let epub = Epub::new("tests/resources/essential-scala.epub").await.unwrap();
        epub.spine_by_index(100).unwrap();
    }

    #[tokio::test]
    async fn success_manifest_by_href() {
        let epub = Epub::new("tests/resources/essential-scala.epub").await.unwrap();
        let manifest = epub.manifest_by_href("media/epub-cover.png").unwrap();
        let actual = manifest.id.as_ref().unwrap();
        assert_eq!(actual, "epub-cover_png");
        let actual = manifest.href.as_ref().unwrap();
        assert_eq!(actual, "media/epub-cover.png");
        let actual = manifest.media_type.as_ref().unwrap();
        assert_eq!(actual, &Mime::from_str("image/png").unwrap());
    }

    #[tokio::test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: not found href. nothing")]
    async fn failure_manifest_by_href() {
        let epub = Epub::new("tests/resources/essential-scala.epub").await.unwrap();
        epub.manifest_by_href("nothing").unwrap();
    }

    #[tokio::test]
    async fn success_manifest_by_idref() {
        let epub = Epub::new("tests/resources/essential-scala.epub").await.unwrap();
        let spine = Spine {
            idref: Ok("cover_xhtml".to_string())
        };
        let manifest = epub.manifest_by_spine(&spine).unwrap();
        let actual = manifest.id.as_ref().unwrap();
        assert_eq!(actual, "cover_xhtml");
        let actual = manifest.href.as_ref().unwrap();
        assert_eq!(actual, "cover.xhtml");
        let actual = manifest.media_type.as_ref().unwrap();
        assert_eq!(actual, &Mime::from_str("application/xhtml+xml").unwrap());
    }

    #[tokio::test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: Spine.idref is error. hogehoge")]
    async fn failure_manifest_by_idref() {
        let epub = Epub::new("tests/resources/essential-scala.epub").await.unwrap();
        let spine = Spine {
            idref: Err(anyhow!("hogehoge".to_string()))
        };
        epub.manifest_by_spine(&spine).unwrap();
    }
}
