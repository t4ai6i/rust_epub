// use rust_epub::epub::Epub;
//
// #[tokio::test]
// async fn success_read_existing_epub() {
//     let epub = Epub::new("tests/resources/essential-scala.epub").await;
//     let epub = epub.read_epub().await;
//     assert!(epub.is_ok());
// }
//
// #[test]
// fn failure_read_non_existing_epub() {
//     let epub = Epub::new("unidentified.epub");
//     let epub = epub.read_epub();
//     assert!(epub.is_err());
// }
