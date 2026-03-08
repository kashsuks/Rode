//! Keyboard event subscription handlers.

use crate::message::{Message, VimKey};
use iced::keyboard::Key;
use iced::window;
use iced::{Event, Subscription};

/// Emits keyboard shortcut messages for global editor actions.
pub fn shortcuts() -> Subscription<Message> {
    iced::event::listen_with(|event, _status, _id| match event {
        Event::Keyboard(iced::keyboard::Event::KeyPressed { key, modifiers, .. }) => {
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
                if modifiers.command() && modifiers.control() {
                    if c.as_str() == "f" {
                        return Some(Message::ToggleFullscreen(window::Mode::Fullscreen));
                    }
                } else if modifiers.command() && modifiers.shift() {
                    match c.as_str() {
                        "v" | "V" => return Some(Message::PreviewMarkdown),
                        "f" | "F" => return Some(Message::ToggleFuzzyFinder),
                        "p" | "P" => return Some(Message::ToggleCommandPalette),
                        "s" | "S" => return Some(Message::ToggleSettings),
                        _ => {}
                    }
                } else if modifiers.command() {
                    match c.as_str() {
                        "b" | "r" => return Some(Message::ToggleSidebar),
                        "o" => return Some(Message::OpenFolderDialog),
                        "w" => return Some(Message::CloseActiveTab),
                        "s" => return Some(Message::SaveFile),
                        "t" => return Some(Message::ToggleFileFinder),
                        "j" => return Some(Message::ToggleTerminal),
                        "f" => return Some(Message::ToggleFindReplace),
                        "n" => return Some(Message::NewFile),
                        _ => {}
                    }
                } else if modifiers.control() {
                    let lower = c.to_lowercase();
                    if let Some(ch) = lower.chars().next() {
                        match ch {
                            'e' | 'y' | 'b' | 'f' | 'd' | 'u' => {
                                return Some(Message::VimKeyPressed(VimKey::Ctrl(ch)));
                            }
                            _ => {}
                        }
                    }
                } else if let Some(ch) = c.chars().next() {
                    return Some(Message::VimKeyPressed(VimKey::Char(ch)));
                }
            }

            if !modifiers.command() && !modifiers.control() {
                match key {
                    Key::Named(iced::keyboard::key::Named::Escape) => {
                        return Some(Message::VimKeyPressed(VimKey::Escape));
                    }
                    Key::Named(iced::keyboard::key::Named::Enter) => {
                        return Some(Message::VimKeyPressed(VimKey::Enter));
                    }
                    Key::Named(iced::keyboard::key::Named::Backspace) => {
                        return Some(Message::VimKeyPressed(VimKey::Backspace));
                    }
                    _ => {}
                }
            }

            None
        }
        _ => None,
    })
}
