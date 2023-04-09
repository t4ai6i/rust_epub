use crate::{
    domain::entity::{manifest::ManifestItem, spine::SpineItemRef},
    utils::zip_utils::read_to_string,
};
use anyhow::{bail, Context, Result};
use mime::Mime;
use roxmltree::{Document, Node};
use std::fs::File;
use std::str::FromStr;
use zip::ZipArchive;

#[derive(Debug, Clone, Default)]
pub struct ContentOpf {
    pub manifest_items: Vec<ManifestItem>,
    pub spine_item_refs: Vec<SpineItemRef>,
}

impl ContentOpf {
    // TODO: タイトル・著者・発行年などのメタ情報も含めるロジックを実装する
    pub fn new(epub: &mut ZipArchive<File>) -> Result<Self> {
        let content_opf_path = ContentOpf::get_path(epub)?;
        let content_opf = read_to_string(epub, &content_opf_path)?;
        let doc = Document::parse(&content_opf)?;
        let nodes = doc.root_element().children();
        let mut content_opf = ContentOpf::default();
        for node in nodes {
            match node.tag_name().name() {
                "manifest" => {
                    let to_manifest_item = |item: Node| {
                        let id = item.attribute("id");
                        let href = item.attribute("href");
                        let media_type = item.attribute("media-type");
                        // media-typeに不正なものがないかMime::from_strを使って判定する
                        let mime = match media_type {
                            None => Mime::from_str(""),
                            Some(media_type) => Mime::from_str(media_type),
                        };
                        match (id, href, mime) {
                            (Some(id), Some(href), Ok(ref mime)) => {
                                Some(ManifestItem::new(epub, id, href, mime))
                            }
                            _ => None,
                        }
                    };
                    let manifest_items: Vec<ManifestItem> = node
                        .children()
                        .filter_map(to_manifest_item)
                        .collect::<Vec<_>>();
                    if manifest_items.is_empty() {
                        // TODO: <manifest>のどの子要素に問題があるか確認できるようにエラーログの仕様を検討
                        bail!("Invalid <manifest>. Check content.opf.");
                    }
                    content_opf = ContentOpf {
                        manifest_items,
                        ..content_opf
                    };
                }
                "spine" => {
                    let to_spine_item_ref = |itemref: Node| {
                        itemref.attribute("idref").map(|idref| SpineItemRef {
                            idref: idref.to_string(),
                        })
                    };
                    let spine_item_refs: Vec<SpineItemRef> = node
                        .children()
                        .filter_map(to_spine_item_ref)
                        .collect::<Vec<_>>();
                    if spine_item_refs.is_empty() {
                        // TODO: <spine>のどの子要素に問題があるか確認できるようにエラーログの仕様を検討
                        bail!("Invalid <spine>. Check content.opf.");
                    }
                    content_opf = ContentOpf {
                        spine_item_refs,
                        ..content_opf
                    };
                }
                _ => {}
            }
        }
        Ok(content_opf)
    }

    fn get_path(epub: &mut ZipArchive<File>) -> Result<String> {
        let container = read_to_string(epub, "META-INF/container.xml")?;
        let doc = Document::parse(&container)?;
        doc.root_element()
            .children()
            .filter_map(|n| {
                n.descendants()
                    .find(|n| n.tag_name().name() == "rootfile")
                    .map(|n| n.attribute("full-path"))?
                    .map(|s| s.to_string())
            })
            .next()
            .context("Not exist content.opf")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::zip_utils::open_zip;

    #[tokio::test]
    async fn success_content_opf_get_path() -> Result<()> {
        let path = "resources/epub/essential-scala.epub";
        let mut epub = open_zip(path).await?;
        let actual = ContentOpf::get_path(&mut epub)?;
        assert_eq!(&actual, "content.opf");
        Ok(())
    }
    // TODO: epubの仕様を満たさない不正なデータ構造を持ったファイルでのテストを実施すること

    #[tokio::test]
    async fn success_content_opf_new() -> Result<()> {
        let path = "resources/epub/essential-scala.epub";
        let mut epub = open_zip(path).await?;
        let content_opf = ContentOpf::new(&mut epub)?;
        assert_eq!(content_opf.manifest_items.len(), 30);
        assert_eq!(content_opf.spine_item_refs.len(), 15);
        Ok(())
    }
}
