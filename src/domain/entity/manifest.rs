use crate::utils::zip_utils::{read_to_end, read_to_string};
use mime::Mime;
use std::fs::File;
use zip::ZipArchive;

impl ManifestItem {
    pub fn new(epub: &mut ZipArchive<File>, id: &str, href: &str, mime: &Mime) -> ManifestItem {
        match (mime.type_(), mime.subtype(), mime.suffix()) {
            // application/xhtml+xml
            (type_, subtype, suffix)
                if type_ == mime::APPLICATION
                    && subtype == "xhtml"
                    && suffix == Some(mime::XML) =>
            {
                let content = read_to_string(epub, href).unwrap();
                ManifestItem::Documentation {
                    id: id.to_string(),
                    href: href.to_string(),
                    content,
                }
            }
            (type_, _, _) if type_ == mime::IMAGE => {
                let content = read_to_end(epub, href).unwrap();
                ManifestItem::Image {
                    id: id.to_string(),
                    href: href.to_string(),
                    content,
                }
            }
            _ => ManifestItem::None,
        }
    }
}

#[derive(Debug, Clone, Default, Ord, PartialOrd, Eq, PartialEq)]
pub enum ManifestItem {
    #[default]
    None,
    Documentation {
        id: String,
        href: String,
        content: String,
    },
    Image {
        id: String,
        href: String,
        content: Vec<u8>,
    },
}
