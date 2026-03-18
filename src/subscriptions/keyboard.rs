//! Keyboard event subscription handlers.

use crate::message::Message;
use iced::keyboard::Key;
use iced::window;
use iced::{Event, Subscription};

/// Emits keyboard shortcut messages for global editor actions.
pub fn shortcuts() -> Subscription<Message> {
    iced::event::listen_with(|event, _status, _id| match event {
        Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
            let primary = modifiers.command() || modifiers.control();
            let navigation_msg = match &key {
                Key::Named(iced::keyboard::key::Named::ArrowUp) => {
                    Some(Message::FuzzyFinderNavigate(-1))
                }
                Key::Named(iced::keyboard::key::Named::ArrowDown) => {
                    Some(Message::FuzzyFinderNavigate(1))
                }
                Key::Named(iced::keyboard::key::Named::Enter) => Some(Message::FuzzyFinderSelect),
                _ => None,
            };

            if navigation_msg.is_some() {
                return navigation_msg;
            }

            if let Key::Character(c) = &key {
                if modifiers.command() && modifiers.control() && c.as_str() == "f" {
                    return Some(Message::ToggleFullscreen(window::Mode::Fullscreen));
                } else if primary && modifiers.shift() {
                    match c.as_str() {
                        "v" | "V" => return Some(Message::PreviewMarkdown),
                        "f" | "F" => return Some(Message::ToggleFuzzyFinder),
                        "p" | "P" => return Some(Message::ToggleCommandPalette),
                        "s" | "S" => return Some(Message::ToggleSettings),
                        "o" | "O" => return Some(Message::OpenFolderDialog),
                        _ => {}
                    }
                } else if primary {
                    match c.as_str() {
                        "b" | "r" => return Some(Message::ToggleSidebar),
                        "o" | "O" => return Some(Message::OpenFileDialog),
                        "w" | "W" => return Some(Message::CloseActiveTab),
                        "s" | "S" => return Some(Message::SaveFile),
                        "t" | "T" => return Some(Message::ToggleFileFinder),
                        "j" | "J" => return Some(Message::ToggleTerminal),
                        "f" | "F" => return Some(Message::ToggleFindReplace),
                        "n" | "N" => return Some(Message::NewFile),
                        _ => {}
                    }
                }
            }

            if !modifiers.command() && !modifiers.control() {
                if let Key::Named(iced::keyboard::key::Named::Escape) = key {
                    return Some(Message::EscapePressed);
                }
            }

            None
        }
        _ => None,
    })
}

/// Emits raw keyboard and mouse input messages for developer logging.
pub fn input_debug() -> Subscription<Message> {
    iced::event::listen_with(|event, _status, _id| match event {
        Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
            Some(Message::InputLog(format!(
                "input:key_pressed key={key:?} shift={} ctrl={} alt={} command={}",
                modifiers.shift(),
                modifiers.control(),
                modifiers.alt(),
                modifiers.command()
            )))
        }
        Event::Keyboard(iced::keyboard::Event::KeyReleased { key, modifiers, .. }) => {
            Some(Message::InputLog(format!(
                "input:key_released key={key:?} shift={} ctrl={} alt={} command={}",
                modifiers.shift(),
                modifiers.control(),
                modifiers.alt(),
                modifiers.command()
            )))
        }
        Event::Mouse(iced::mouse::Event::ButtonPressed(button)) => Some(Message::InputLog(
            format!("input:mouse_pressed button={button:?}"),
        )),
        Event::Mouse(iced::mouse::Event::ButtonReleased(button)) => Some(Message::InputLog(
            format!("input:mouse_released button={button:?}"),
        )),
        _ => None,
    })
}
