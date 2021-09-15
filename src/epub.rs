use std::path::Path;
use std::io::Read;
use std::fs::File;
use roxmltree::Document;
use tokio::fs::File as TokioFile;
use anyhow::{Result, Context};
use zip::ZipArchive;
use zip::result::ZipResult;

#[derive(Debug, Default, PartialEq)]
pub struct ContentOpf {
    manifests: Vec<Manifest>,
    spines: Vec<Spine>,
}

#[derive(Debug, Default, PartialEq)]
pub struct Manifest {
    id: String,
    href: String,
    media_type: String,
}

#[derive(Debug, Default, PartialEq)]
pub struct Spine {
    idref: String,
}

impl ContentOpf {
    pub fn new(xml: &str) -> Result<Self> {
        let doc = Document::parse(xml)?;
        let nodes = doc.root_element().children();
        let mut content_opf = ContentOpf::default();
        for node in nodes {
            match node.tag_name().name() {
                "manifest" => {
                    let manifests = node.children()
                        .filter(|x| x.is_element())
                        .map(|manifest| Manifest {
                            id: manifest.attribute("id").unwrap().to_string(),
                            href: manifest.attribute("href").unwrap().to_string(),
                            media_type: manifest.attribute("media-type").unwrap().to_string(),
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
                            idref: spine.attribute("idref").unwrap().to_string(),
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
    // TODO: 実データの取得。media_typeがapplication/xhtml+xmlだったらStringとして取得。image/pngだったらVec<u8>として取得。
    // TODO: mime crate. https://docs.rs/mime/0.3.16/mime/
    /// Create Epub
    pub async fn new(path: impl AsRef<Path>) -> Result<Self> {
        let mut epub = Epub::open_zip_archive(path).await?;
        // for i in 0..epub.len() {
        //     let file = epub.by_index(i)?;
        //     dbg!(file.name());
        // }
        let content_opf_path = Epub::content_opf_path(&mut epub)?;
        let content_opf = Epub::content_opf(&mut epub, &content_opf_path)?;
        let content_opf = ContentOpf::new(&content_opf)?;
        Ok(Epub {
            body: epub,
            content_opf,
        })
    }

    /// Returns length of spines
    pub fn spines_len(&self) -> usize {
        self.content_opf.spines.len()
    }

    /// Returns the Spine by index
    pub fn spine_by_index(&self, index: usize) -> Result<&Spine> {
        self.content_opf.spines.iter()
            .nth(index)
            .with_context(|| format!("Index out of bounds. {}/{}", index, self.content_opf.spines.len()))
    }

    /// Returns the manifest by idref
    pub fn manifest_by_idref(&self, idref: &str) -> Result<&Manifest> {
        self.content_opf.manifests.iter()
            .find(|manifest| manifest.id.eq(idref))
            .with_context(|| format!("Not found idref. {}", idref))
    }

    fn content_opf(epub: &mut ZipArchive<File>, content_opf_path: &str) -> Result<String> {
        let content_opf = &mut epub.by_name(content_opf_path)?;
        let mut content_opf_buf = String::new();
        content_opf.read_to_string(&mut content_opf_buf)?;
        Ok(content_opf_buf)
    }

    fn content_opf_path(epub: &mut ZipArchive<File>) -> Result<String> {
        let container = &mut epub.by_name("META-INF/container.xml")?;
        let mut container_buf = String::new();
        container.read_to_string(&mut container_buf)?;
        let doc = Document::parse(&container_buf)?;
        doc.root_element().children()
            .filter_map(|n| {
                let rootfile = n.descendants()
                    .find(|n| n.tag_name().name() == "rootfile");
                rootfile.map(|n| {
                    n.attribute("full-path")
                })?
            })
            .next()
            .map(|p| p.to_string())
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
    async fn success_content_opf_path() {
        let path = "tests/resources/essential-scala.epub";
        let mut archive = Epub::open_zip_archive(path).await.unwrap();
        let actual = Epub::content_opf_path(&mut archive).unwrap();
        assert_eq!(&actual, "content.opf");
    }

    #[tokio::test]
    async fn success_epub_content_opf() {
        let path = "tests/resources/essential-scala.epub";
        let mut archive = Epub::open_zip_archive(path).await.unwrap();
        let content_opf_path = Epub::content_opf_path(&mut archive).unwrap();
        let actual = Epub::content_opf(&mut archive, &content_opf_path);
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
        let actual = epub.spine_by_index(0).unwrap();
        assert_eq!(*actual, Spine { idref: "cover_xhtml".to_string() });
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
        let actual = epub.manifest_by_idref("cover_xhtml").unwrap();
        assert_eq!(*actual, Manifest {
            id: "cover_xhtml".to_string(),
            href: "cover.xhtml".to_string(),
            media_type: "application/xhtml+xml".to_string(),
        });
    }

    #[tokio::test]
    #[should_panic(expected = "called `Result::unwrap()` on an `Err` value: Not found idref. hogehoge")]
    async fn failure_manifest_by_idref() {
        let epub = Epub::new("tests/resources/essential-scala.epub").await.unwrap();
        epub.manifest_by_idref("hogehoge").unwrap();
    }
}
