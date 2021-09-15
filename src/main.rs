use anyhow::{Result};
use rust_epub::epub::Epub;

#[tokio::main]
async fn main() -> Result<()> {
    // epubを読み込む
    let epub = Epub::new("tests/resources/essential-scala.epub").await?;
    dbg!(&epub.content_opf);
    Ok(())
}
