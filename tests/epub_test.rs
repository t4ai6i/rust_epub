use rust_epub::epub::core::Epub;
use anyhow::Result;

#[tokio::test]
async fn success_read_existing_epub() -> Result<()> {
    let _ = Epub::new("resources/epub/essential-scala.epub").await?;
    Ok(())
}

#[tokio::test]
async fn failure_read_non_existing_epub() {
    let epub = Epub::new("unidentified.epub").await;
    assert!(epub.is_err());
}
