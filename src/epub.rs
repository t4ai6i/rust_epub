use std::path::Path;
use std::io::Read;
use std::fs::File;
use roxmltree::{Document, Node};
use tokio::fs::File as TokioFile;
use anyhow::{Result, Context, ensure, bail};
use zip::ZipArchive;
use zip::result::ZipResult;
use mime::Mime;
use std::str::FromStr;

#[derive(Debug)]
pub struct ContentOpf {
    manifests: Vec<Manifest>,
    spines: Vec<Spine>,
}

impl Default for ContentOpf {
    fn default() -> Self {
        Self {
            manifests: Vec::new(),
            spines: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Manifest {
    id: Result<String>,
    href: Result<String>,
    media_type: Result<Mime>,
}

#[derive(Debug)]
pub struct Spine {
    idref: Result<String>,
}

impl ContentOpf {
    fn new(xml: &str) -> Result<Self> {
        let doc = Document::parse(xml)?;
        let nodes = doc.root_element().children();
        let mut content_opf = ContentOpf::default();
        let to_string = |node: &Node, key: &str|
            node.attribute(key)
                .with_context(|| format!("Not found :{}", key))
                .map(|x| x.to_string());
        let to_mime = |node: &Node, key: &str| {
            node.attribute(key)
                .with_context(|| format!("Not found :{}", key))
                .map(|x|
                    Mime::from_str(x).with_context(|| format!("Failed parse: {}", x))
                )?
        };
        for node in nodes {
            match node.tag_name().name() {
                "manifest" => {
                    let manifests = node.children()
                        .filter(|x| x.is_element())
                        .map(|manifest| Manifest {
                            id: to_string(&manifest, "id"),
                            href: to_string(&manifest, "href"),
                            media_type: to_mime(&manifest, "media-type"),
                        })
                        .collect::<Vec<Manifest>>();
                    content_opf = ContentOpf {
                        manifests,
                        ..content_opf
                    };
                }
                "spine" => {
                    let spines = node.children()
                        .filter(|x| x.is_element())
                        .map(|spine| Spine {
                            idref: to_string(&spine, "idref"),
                        })
                        .collect::<Vec<Spine>>();
                    content_opf = ContentOpf {
                        spines,
                        ..content_opf
                    };
                }
                _ => {}
            }
        }
        Ok(content_opf)
    }
}

pub struct Epub {
    pub body: ZipArchive<File>,
    pub content_opf: ContentOpf,
}

impl Epub {
    /// Create Epub
    pub async fn new(path: impl AsRef<Path>) -> Result<Self> {
        let mut epub = Epub::open_zip_archive(path).await?;
        // for i in 0..epub.len() {
        //     let file = epub.by_index(i)?;
        //     dbg!(file.name());
        // }
        let content_opf_path = Epub::content_opf_path(&mut epub)?;
        let content_opf = Epub::read_content_to_string(&mut epub, &content_opf_path)?;
        let content_opf = ContentOpf::new(&content_opf)?;
        Ok(Epub {
            body: epub,
            content_opf,
        })
    }

    pub fn read_content_by_index_of_spine(&mut self, spine_index: usize) -> Result<String> {
        let spine = self.spine_by_index(spine_index)?;
        let manifest = self.manifest_by_idref(spine)?;
        let href = match &manifest.href {
            Ok(v) => v.to_string(),
            Err(e) => bail!(format!("Invalid Manifest.href. {:?}", e)),
        };
        Epub::read_content_to_string(&mut self.body, &href)
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

    /// Returns the manifest by idref
    fn manifest_by_idref(&self, spine: &Spine) -> Result<&Manifest> {
        ensure!(
            spine.idref.as_ref().is_ok(),
            format!("Spine.idref is error. {}", spine.idref.as_ref().err().unwrap())
        );

        let idref = spine.idref.as_ref().unwrap();
        self.content_opf.manifests.iter()
            .find(|manifest| manifest.id.is_ok() && manifest.id.as_ref().unwrap().eq(idref))
            .with_context(|| format!("Not found idref. {}", idref))
    }

    fn read_content_to_string(epub: &mut ZipArchive<File>, path: &str) -> Result<String> {
        let content = &mut epub.by_name(path)?;
        let mut string_buf = String::new();
        content.read_to_string(&mut string_buf)?;
        Ok(string_buf)
    }

    fn content_opf_path(epub: &mut ZipArchive<File>) -> Result<String> {
        let container = Epub::read_content_to_string(epub, "META-INF/container.xml")?;
        let doc = Document::parse(&container)?;
        doc.root_element().children()
            .filter_map(|n|
                n.descendants()
                    .find(|n| n.tag_name().name() == "rootfile")
                    .map(|n| n.attribute("full-path"))?
                    .map(|s| s.to_string())
            )
            .next()
            .context("Not exist content.opf")
    }

    async fn open_zip_archive(path: impl AsRef<Path>) -> ZipResult<ZipArchive<File>> {
        let epub = TokioFile::open(path).await?;
        let epub = epub.into_std().await;
        ZipArchive::new(epub)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;
    use anyhow::anyhow;

    #[tokio::test]
    async fn success_read_epub() {
        let actual = Epub::new("tests/resources/essential-scala.epub").await;
        assert!(actual.is_ok());
        assert_eq!(actual.unwrap().spines_len(), 15);
    }

    #[tokio::test]
    async fn failure_read_non_existing_epub() {
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
    async fn success_content_opf_path() {
        let path = "tests/resources/essential-scala.epub";
        let mut archive = Epub::open_zip_archive(path).await.unwrap();
        let actual = Epub::content_opf_path(&mut archive).unwrap();
        assert_eq!(&actual, "content.opf");
    }

    #[tokio::test]
    async fn success_epub_read_content_to_string() {
        let path = "tests/resources/essential-scala.epub";
        let mut archive = Epub::open_zip_archive(path).await.unwrap();
        let content_opf_path = Epub::content_opf_path(&mut archive).unwrap();
        let actual = Epub::read_content_to_string(&mut archive, &content_opf_path);
        assert!(actual.is_ok());
    }

    #[tokio::test]
    async fn success_content_opf_new() {
        let mut content_opf_xml = TokioFile::open("tests/resources/epub_content_opf.xml").await.unwrap();
        let mut content_opf = String::new();
        content_opf_xml.read_to_string(&mut content_opf).await.unwrap();
        let content_opf = ContentOpf::new(&content_opf).unwrap();
        assert!(content_opf.manifests.len() > 0);
        assert!(content_opf.spines.len() > 0);
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
    async fn success_manifest_by_idref() {
        let epub = Epub::new("tests/resources/essential-scala.epub").await.unwrap();
        let spine = Spine {
            idref: Ok("cover_xhtml".to_string())
        };
        let manifest = epub.manifest_by_idref(&spine).unwrap();
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
        epub.manifest_by_idref(&spine).unwrap();
    }
}
