use iced::keyboard::{key, Key};
use iced::widget::{column, container, row, scrollable, text};
use iced::widget::text_editor::{TextEditor, Content, Binding, KeyPress, Motion};
use iced::{Element, Length};

use crate::message::Message;
use crate::syntax::{VscodeHighlighter, Settings};
use crate::theme::*;
use crate::ui::styles::text_editor_style;

/// Font size used for line numbers (should match editor font size).
const LINE_NUMBER_SIZE: f32 = 14.0;
/// Width of the line number gutter column.
const GUTTER_WIDTH: f32 = 48.0;

pub fn create_editor<'a>(content: &'a Content, extension: &str, current_line: usize) -> Element<'a, Message> {
    let line_count = content.line_count();

    // ── Line number gutter ──────────────────────────────────────────────
    let mut numbers: Vec<Element<'_, Message>> = Vec::with_capacity(line_count);
    for i in 1..=line_count {
        let color = if i == current_line { TEXT_1 } else { OVERLAY_2 };
        numbers.push(
            container(
                text(format!("{}", i))
                    .size(LINE_NUMBER_SIZE)
                    .color(color)
            )
            .width(Length::Fill)
            .align_right(Length::Fill)
            .into(),
        );
    }

    let gutter: Element<'_, Message> = container(
        scrollable(
            column(numbers)
                .spacing(4)
                .padding(iced::Padding { top: 7.0, right: 8.0, bottom: 0.0, left: 4.0 })
        )
        .height(Length::Fill)
    )
    .width(Length::Fixed(GUTTER_WIDTH))
    .height(Length::Fill)
    .into();

    // ── Text editor ─────────────────────────────────────────────────────
    let editor = TextEditor::new(content)
        .on_action(Message::EditorAction)
        .key_binding(editor_key_bindings)
        .highlight_with::<VscodeHighlighter>(
            Settings {
                extension: extension.to_string(),
            },
            |highlight, _theme| highlight.to_format(),
        )
        .style(text_editor_style)
        .height(Length::Fill);

    row![gutter, editor]
        .height(Length::Fill)
        .into()
}

fn editor_key_bindings(key_press: KeyPress) -> Option<Binding<Message>> {
    let modifiers = key_press.modifiers;

    if let Key::Character(_c) = key_press.key.as_ref() {
        if modifiers.command() {
            return None;
        }
    }

    match key_press.key.as_ref() {
        Key::Named(key::Named::Backspace) => {
            if modifiers.command() {
                Some(Binding::Sequence(vec![ // Detects when the cmd key is pressed and begin a sequence
                    Binding::Select(Motion::Home),
                    Binding::Backspace, // If home + backspace is detected, remove whole line
                ]))
            } else if modifiers.alt() {
                Some(Binding::Sequence(vec![
                    Binding::Select(Motion::WordLeft),
                    Binding::Backspace, // If the alt + delete, the word to the left is gone
                ]))
            } else {
                Binding::from_key_press(key_press) // Returns the default key press.
            }
        }
        Key::Named(key::Named::Delete) => {
            if modifiers.command() {
                Some(Binding::Sequence(vec![
                    Binding::Select(Motion::End),
                    Binding::Delete, // cmd + delete (the one that deletes a character to the right) deletes the line to the right of the cursor
                ]))
            } else if modifiers.alt() {
                Some(Binding::Sequence(vec![
                    Binding::Select(Motion::WordRight),
                    Binding::Delete, // alt + delete removes the word to the right
                ]))
            } else {
                Binding::from_key_press(key_press) // Again, ensures default actions
            }
        }
        _ => Binding::from_key_press(key_press),
    }
}

pub fn empty_editor<'a>() -> Element<'a, Message> {
    iced::widget::text("").into()
}
