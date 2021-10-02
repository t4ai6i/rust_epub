use anyhow::Result;
use mime::Mime;

#[derive(Debug)]
pub struct Manifest {
    pub id: Result<String>,
    pub href: Result<String>,
    pub media_type: Result<Mime>,
}
