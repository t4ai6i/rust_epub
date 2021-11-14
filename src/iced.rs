use std::ops::Add;
use roxmltree::{Document, Node as XmlNode};
use anyhow::Result as AnyhowResult;
use anyhow::Context;
use iced::{Application, Clipboard, Column, Command, Container, Element, HorizontalAlignment, Image, Length, Subscription, Text};
use iced::image::Handle;
use iced_native::{keyboard, Event};
use crate::epub::Epub;

#[derive(Debug, Default)]
pub struct EpubViewer {
    pub current_page: usize,
    pub status: Status,
    pub epub: Epub,
}

impl EpubViewer {
    fn turn_previous_page(&mut self) -> usize {
        let new_page = self.current_page.checked_sub(1).unwrap_or(usize::MIN);
        self.current_page = new_page;
        self.current_page
    }

    fn turn_next_page(&mut self) -> usize {
        let new_page = self.current_page.checked_add(1).unwrap_or(usize::MAX);
        let new_page = if new_page + 1 >= self.epub.table_of_contents.len() {
            new_page.checked_sub(1).unwrap_or(usize::MIN)
        } else {
            new_page
        };
        self.current_page = new_page;
        self.current_page
    }
}

#[derive(Debug)]
pub enum Status {
    Loading,
    Reading,
    Errored(Error),
}

impl Default for Status {
    fn default() -> Self {
        Self::Loading
    }
}

/// Kind of operation EpubViewer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    TurnPreviousPage,
    TurnNextPage,
}

#[derive(Debug, Clone)]
pub enum Message {
    LoadedEpub(Result<Epub, Error>),
    OccurredOperation(Operation),
}

#[derive(Debug, Clone)]
pub enum Error {
    EpubLoadError(String)
}

// TODO: title_page.xhtmlのレンダリング
// TODO: メニューバーへのタイトル表記
// TODO: ファイルのドラッグ・アンド・ドロップ
// TODO: 前回まで読み進めた分のマーカー
// TODO: i18n
// TODO: GoogleDriveやDropboxなどFileSharingServiceとの連携
impl Application for EpubViewer {
    type Executor = iced::executor::Default;
    type Message = Message;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            EpubViewer::default(),
            Command::perform(
                load_epub("resources/epub/essential-scala.epub"),
                Message::LoadedEpub,
            )
        )
    }

    fn title(&self) -> String {
        match &self.status {
            Status::Loading => "Loading".to_string(),
            // TODO: epubのタイトルの表示
            Status::Reading => {
                let current_page = self.current_page.add(1);
                format!("Page {}", current_page)
            }
            Status::Errored(_v) => "Whoops!".to_string(),
        }
    }

    fn update(&mut self, message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
        match message {
            Message::LoadedEpub(Ok(epub)) => {
                self.status = Status::Reading;
                self.epub = epub;
                Command::none()
            }
            Message::LoadedEpub(Err(err)) => {
                self.status = Status::Errored(err);
                Command::none()
            }
            Message::OccurredOperation(operation) => {
                match operation {
                    Operation::TurnPreviousPage => self.turn_previous_page(),
                    Operation::TurnNextPage => self.turn_next_page(),
                };
                Command::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        iced_native::subscription::events_with(|event, status| {
            if let iced_native::event::Status::Captured = status {
                return None;
            }

            match event {
                Event::Keyboard(
                    keyboard::Event::KeyPressed {
                        key_code,
                        ..
                    }
                ) => handle_hotkey(key_code),
                _ => None,
            }
        })
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let content = match &self.status {
            Status::Loading => {
                let text = Text::new("読み込み中")
                    .horizontal_alignment(HorizontalAlignment::Center);
                let content = Column::new()
                    .push(text);
                content
            }
            Status::Reading => {
                let table_of_content = self.epub.table_of_contents.get(self.current_page).unwrap();
                let xhtml = self.epub.xhtml.get(table_of_content).unwrap();
                let doc = Document::parse(xhtml).unwrap();
                let elements = build_elements(&self.epub, doc).unwrap();
                let content = Column::new()
                    .push(elements);
                content
            }
            Status::Errored(_err) => {
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

fn handle_hotkey(key_code: keyboard::KeyCode) -> Option<Message> {
    match key_code {
        keyboard::KeyCode::Left => Some(Message::OccurredOperation(Operation::TurnPreviousPage)),
        keyboard::KeyCode::Right => Some(Message::OccurredOperation(Operation::TurnNextPage)),
        _ => None,
    }
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