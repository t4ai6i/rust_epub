use url::Url;

pub enum EpubPath<'a> {
    LocalPath(&'a str),
    Url(&'a Url),
    NFTAddress(&'a str),
}
