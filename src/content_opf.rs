use anyhow::{Result, Context};
use mime::Mime;
use roxmltree::{Document, Node};
use zip::ZipArchive;
use std::fs::File;
use std::str::FromStr;

use crate::manifest::Manifest;
use crate::spine::Spine;
use crate::zip_utils::read_to_string;

#[derive(Debug)]
pub struct ContentOpf {
    pub manifests: Vec<Manifest>,
    pub spines: Vec<Spine>,
}

impl Default for ContentOpf {
    fn default() -> Self {
        Self {
            manifests: Vec::new(),
            spines: Vec::new(),
        }
    }
}

impl ContentOpf {
    // TODO: タイトル・著者・発行年などのメタ情報も含めるロジックを実装する
    pub fn new(xml: &str) -> Result<Self> {
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

    pub fn content_opf_path(epub: &mut ZipArchive<File>) -> Result<String> {
        let container = read_to_string(epub, "META-INF/container.xml")?;
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;
    use crate::zip_utils::open_zip;

    #[tokio::test]
    async fn success_content_opf_path() {
        let path = "resources/epub/essential-scala.epub";
        let mut archive = open_zip(path).await.unwrap();
        let actual = ContentOpf::content_opf_path(&mut archive).unwrap();
        assert_eq!(&actual, "content.opf");
    }

    #[tokio::test]
    async fn success_content_opf_new() {
        let mut content_opf_xml = File::open("resources/xml/epub_content_opf.xml").await.unwrap();
        let mut content_opf = String::new();
        content_opf_xml.read_to_string(&mut content_opf).await.unwrap();
        let content_opf = ContentOpf::new(&content_opf).unwrap();
        assert_eq!(content_opf.manifests.len(), 30);
        assert_eq!(content_opf.spines.len(), 15);
    }
}