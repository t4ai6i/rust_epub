use anyhow::Result;
use rust_epub::use_case::interface::load_epub::LoadEpubUseCase;
use rust_epub::{
    domain::entity::epub_path::EpubPath,
    infrastructure::epub_repository_impl::EpubRepositoryImpl,
    use_case::{
        interactor::{list_epub::ListEpubInteractor, load_epub::LoadEpubInteractor},
        interface::list_epub::ListEpubUseCase,
    },
};

#[tokio::test]
async fn get_epub_integration_test() -> Result<()> {
    let repository = EpubRepositoryImpl::new();
    let list_epub_interactor = ListEpubInteractor::new(&repository);
    // EpubPath::LocalPathで指定されたlocal_pathの中のepubファイルリストを取得する
    let epub_dir_path = EpubPath::new("./resources/epub");
    let epub_paths = list_epub_interactor.execute(&epub_dir_path).await?;
    assert_eq!(epub_paths.len(), 1);
    let epub_path = epub_paths.first().unwrap();
    let load_epub_interactor = LoadEpubInteractor::new(&repository);
    let epub = load_epub_interactor.execute(epub_path).await;
    assert!(epub.is_ok());
    Ok(())
}
