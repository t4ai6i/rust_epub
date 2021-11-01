use roxmltree::{Document, Node as XmlNode};
use anyhow::Result as AnyhowResult;
use anyhow::Context;
use iced::{Application, Clipboard, Column, Command, Container, Element, HorizontalAlignment, Image, Length, Text};
use iced::image::Handle;
use crate::epub::Epub;

#[derive(Debug, Default)]
pub struct Status {
    pub current_page: usize
}

#[derive(Debug)]
pub enum EpubViewer {
    Loading,
    ReadyToRead(Epub, Status),
    Errored(Error),
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadedEpub(Result<Epub, Error>),
}

#[derive(Debug, Clone)]
pub enum Error {
    EpubLoadError(String)
}

impl Application for EpubViewer {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            EpubViewer::Loading,
            Command::perform(
                load_epub("resources/epub/essential-scala.epub"),
                Message::LoadedEpub
            )
        )
    }

    fn title(&self) -> String {
        let subtitle = match self {
            EpubViewer::Loading => "Loading",
            EpubViewer::ReadyToRead(_epub, _status) => "ReadyToRead",
            EpubViewer::Errored(_v) => "Whoops!",
        };

        format!("{} - EpubViewer", subtitle)
    }

    fn update(&mut self, message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
        match message {
            Message::LoadedEpub(Ok(epub)) => {
                let status = Status {
                    current_page: 0,
                    ..Status::default()
                };
                *self = EpubViewer::ReadyToRead(epub, status);
                Command::none()
            }
            Message::LoadedEpub(Err(v)) => {
                *self = EpubViewer::Errored(v);
                Command::none()
            }
        }
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let content = match self {
            EpubViewer::Loading => {
                let text = Text::new("読み込み中")
                    .horizontal_alignment(HorizontalAlignment::Center);
                let content = Column::new()
                    .push(text);
                content
            },
            EpubViewer::ReadyToRead(epub, status) => {
                let table_of_content = epub.table_of_contents.get(status.current_page).unwrap();
                let xhtml = epub.xhtml.get(table_of_content).unwrap();
                let doc = Document::parse(xhtml).unwrap();
                let elements = build_elements(epub, doc).unwrap();
                let content = Column::new()
                    .push(elements);
                content
            },
            EpubViewer::Errored(_err) => {
                let text = Text::new("エラー！！！")
                    .horizontal_alignment(HorizontalAlignment::Center);
                let content = Column::new()
                    .push(text);
                content
            }
        };
        Container::new(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}

async fn load_epub(path: &str) -> Result<Epub, Error> {
    let epub = Epub::new(path).await;
    let epub = epub.map_err(|v| {
        dbg!(&v);
        Error::EpubLoadError(format!("{}", path))
    })?;
    Ok(epub)
}

fn build_image<'a>(epub: &Epub, node: &XmlNode) -> Element<'a, Message> {
    let src = node.attribute("src").unwrap();
    let image = epub.media.get(src).unwrap();
    let handle = Handle::from_memory(image.to_vec());
    let image = Image::new(handle);
    image.into()
}

fn recursive_build_elements<'a>(epub: &Epub, node: &XmlNode) -> Element<'a, Message> {
    dbg!(node);
    let tag_name = node.tag_name().name().to_lowercase();
    let has_children = node.has_children();
    dbg!(has_children);
    let container = if has_children {
        let mut container = Column::new();
        for node in node.children().filter(|node| node.is_element()) {
            let element = recursive_build_elements(epub, &node);
            container = container.push(element);
        }
        container.into()
    } else {
        match tag_name.as_str() {
            "img" => {
                build_image(epub, &node)
            }
            name => Text::new(name).into()
        }
    };
    container
}

fn build_elements<'a>(epub: &Epub, doc: Document) -> AnyhowResult<Element<'a, Message>> {
    let body = doc.root_element().children()
        .filter(|node| node.is_element() && node.tag_name().name().eq("body"))
        .nth(0)
        .with_context(|| format!("Body element is nothing."))?;
    let container = recursive_build_elements(epub, &body);
    Ok(container)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn success_render_cover_xhtml() {
        let epub = Epub::new("resources/epub/essential-scala.epub").await.unwrap();
        let table_of_content = epub.table_of_contents.get(0).unwrap();
        let xhtml = epub.xhtml.get(table_of_content).unwrap();
        let doc = Document::parse(xhtml).unwrap();
        let actual = build_elements(&epub, doc);
        assert!(actual.is_ok());
    }

    #[tokio::test]
    async fn success_load_epub() {
        let result = load_epub("resources/epub/essential-scala.epub").await;
        assert!(result.is_ok());
    }
}