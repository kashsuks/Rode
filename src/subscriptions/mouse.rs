//! Mouse event subscription handlers.

use crate::message::Message;
use iced::{Event, Subscription};

/// Emits sidebar resize messages from mouse events.
pub fn sidebar_resize() -> Subscription<Message> {
    iced::event::listen_with(|event, _status, _id| match event {
        Event::Mouse(iced::mouse::Event::CursorMoved { position }) => {
            Some(Message::SidebarResizing(position.x))
        }
        Event::Mouse(iced::mouse::Event::ButtonReleased(iced::mouse::Button::Left)) => {
            Some(Message::SidebarResizeEnd)
        }
        _ => None,
    })
}
