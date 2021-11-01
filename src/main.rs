use iced::{Application, Result, Settings};
use rust_epub::iced::EpubViewer;

fn main() -> Result {
    EpubViewer::run(Settings {
        default_font: Some(include_bytes!("../resources/font/Mamelon-3-Hi-Regular.otf")),
        ..Settings::default()
    })
}
