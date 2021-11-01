use std::path::Path;
use std::collections::HashMap;
use anyhow::Result;
use crate::content_opf::ContentOpf;
use crate::zip_utils::{open_zip, read_to_end, read_to_string};

#[derive(Debug, Clone)]
pub struct Epub {
    pub table_of_contents: Vec<String>,
    pub xhtml: HashMap<String, String>,
    pub media: HashMap<String, Vec<u8>>,
}

impl Epub {
    /// Create Epub
    pub async fn new(path: impl AsRef<Path>) -> Result<Self> {
        let mut epub = open_zip(path).await?;
        let content_opf_path = ContentOpf::content_opf_path(&mut epub)?;
        let content_opf = read_to_string(&mut epub, &content_opf_path)?;
        let content_opf = ContentOpf::new(&content_opf)?;
        let table_of_contents = content_opf.spines.iter().fold(Vec::new(), |mut table_of_contents, spine| {
            let idref = spine.idref.as_ref().unwrap();
            let idref = format!("{}", idref);
            table_of_contents.push(idref);
            table_of_contents
        });
        let manifests = content_opf.manifests;
        let (xhtml, media) = manifests
            .into_iter().fold(
            (HashMap::new(), HashMap::new()),
            |(mut xhtml, mut media), manifest| {
                let id = manifest.id.unwrap();
                let href = manifest.href.unwrap();
                let mime = manifest.media_type.unwrap();
                match (mime.type_(), mime.subtype(), mime.suffix()) {
                    // application/xhtml+xml
                    (type_, subtype, suffix) if type_ == mime::APPLICATION && subtype == "xhtml" && suffix == Some(mime::XML) => {
                        let content = read_to_string(&mut epub, &href).unwrap();
                        xhtml.insert(id, content);
                    }
                    (type_, _, _) if type_ == mime::IMAGE => {
                        let content = read_to_end(&mut epub, &href).unwrap();
                        media.insert(href, content);
                    }
                    _ => ()
                }
                (xhtml, media)
            });
        Ok(Epub {
            table_of_contents,
            xhtml,
            media,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn success_epub_new() {
        let actual = Epub::new("resources/epub/essential-scala.epub").await;
        assert!(actual.is_ok());
        let actual = actual.unwrap();
        dbg!(actual.table_of_contents);
        for key in actual.xhtml.keys() {
            dbg!(key);
        };
        for key in actual.media.keys() {
            dbg!(key);
        };
    }

    #[tokio::test]
    async fn failure_epub_new() {
        let actual = Epub::new("non_existing.epub").await;
        assert!(actual.is_err());
    }
}
