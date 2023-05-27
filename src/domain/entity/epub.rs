use crate::domain::entity::content_opf::ContentOpf;
use anyhow::Result;
use std::fs::File;
use zip::ZipArchive;

#[derive(Debug, Clone, Default)]
pub struct Epub {
    pub content_opf: ContentOpf,
}

impl Epub {
    /// Create Epub
    pub fn new(mut epub: ZipArchive<File>) -> Result<Self> {
        let content_opf = ContentOpf::new(&mut epub)?;
        Ok(Epub { content_opf })
    }

    pub fn number_of_table_of_contents(&self) -> usize {
        self.content_opf.spine_item_refs.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::zip_utils::open_zip;

    #[tokio::test]
    async fn succeeds_in_reading_a_epub_that_exists() -> Result<()> {
        let zip = open_zip("resources/epub/essential-scala.epub").await?;
        let _ = Epub::new(zip)?;
        Ok(())
    }

    #[tokio::test]
    async fn succeeds_in_getting_number_of_table_of_contents() -> Result<()> {
        let zip = open_zip("resources/epub/essential-scala.epub").await?;
        let epub = Epub::new(zip)?;
        let expected = 15;
        let actual = epub.number_of_table_of_contents();
        assert_eq!(actual, expected);
        Ok(())
    }
}
