use iced::{executor, Application, Settings, Element, Command, Clipboard, Image, Result, Text, HorizontalAlignment, Column, Length, Align, Font};
use iced::image::Handle;

struct GUI {
    font: Font,
}

impl Application for GUI {
    type Executor = executor::Default;
    type Message = ();
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (
            GUI {
                font: Font::External {
                    name: "Mamelon-3-Hi-Regular.otf",
                    bytes: include_bytes!("../resources/font/Mamelon-3-Hi-Regular.otf"),
                }
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        String::from("iced test")
    }

    fn update(&mut self, _message: Self::Message, _clipboard: &mut Clipboard) -> Command<Self::Message> {
        Command::none()
    }

    fn view(&mut self) -> Element<'_, Self::Message> {
        let image = include_bytes!("../resources/media/epub-cover.png");
        let image = image.to_vec();
        let handle = Handle::from_memory(image);
        let image = Image::new(handle);
        let text = Text::new("iced テスト")
            .font(self.font)
            .horizontal_alignment(HorizontalAlignment::Center);
        let column = Column::new();
        column.push(text)
            .push(image)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_items(Align::Center)
            .into()
    }
}

fn main() -> Result {
    GUI::run(Settings::default())
}