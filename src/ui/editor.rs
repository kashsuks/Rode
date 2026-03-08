use iced::Element;

use crate::message::Message;

pub fn empty_editor<'a>() -> Element<'a, Message> {
    iced::widget::text("").into()
}
