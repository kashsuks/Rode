use iced::widget::{text, column, row, button, scrollable, container};
use iced::widget::svg::{Svg, Handle};
use iced::widget::image;
use iced::{Element, Length};

use crate::file_tree::{FileEntry, FileTree};
use crate::icons::{get_file_icon, get_folder_icon};
use crate::message::Message;
use crate::theme::*;
use crate::ui::styles::{tree_button_style, sidebar_container_style};

/// Create an icon element from a path, choosing Svg or Image widget based on extension.
fn icon_widget<'a>(icon_path: &str) -> Element<'a, Message> {
    if icon_path.ends_with(".png") {
        image::Image::new(icon_path.to_string())
            .width(Length::Fixed(ICON_SIZE))
            .height(Length::Fixed(ICON_SIZE))
            .into()
    } else {
        Svg::new(Handle::from_path(icon_path))
            .width(Length::Fixed(ICON_SIZE))
            .height(Length::Fixed(ICON_SIZE))
            .into()
    }
}

pub fn view_sidebar<'a>(file_tree: Option<&'a FileTree>, width: f32) -> Element<'a, Message> {
    let sidebar_content: Element<'a, Message> = match file_tree {
        Some(tree) => view_file_tree(tree),
        None => view_empty_sidebar(),
    };

    let sidebar = container(
        scrollable(sidebar_content).height(Length::Fill)
    )
    .width(Length::Fixed(width))
    .height(Length::Fill)
    .padding(4)
    .style(sidebar_container_style);

    container(sidebar)
        .padding(0)
        .into()
}

fn view_file_tree(tree: &FileTree) -> Element<'_, Message> {
    let mut items: Vec<Element<'_, Message>> = Vec::new();
    render_entries(&tree.entries, tree, 0, &mut items);
    column(items).spacing(4).into()
}

fn view_empty_sidebar<'a>() -> Element<'a, Message> {
    container(
        column![
            text("No folder open").size(13).color(THEME.text_muted),
            text("Cmd+O to open").size(11).color(THEME.text_placeholder),
        ]
        .spacing(4)
        .align_x(iced::Alignment::Center)
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .center_x(Length::Fill)
    .center_y(Length::Fill)
    .into()
}

fn render_entries<'a>(
    entries: &'a [FileEntry],
    tree: &'a FileTree,
    depth: usize,
    items: &mut Vec<Element<'a, Message>>,
) {
    let indent_width = INDENT_WIDTH * depth as f32;

    for entry in entries {
        match entry {
            FileEntry::Directory { path, name, children } => {
                let is_expanded = tree.is_expanded(path);
                let icon_path = get_folder_icon(name, is_expanded);

                let icon: Element<'_, Message> = icon_widget(&icon_path);

                let btn = button(
                    row![
                        container(text("")).width(Length::Fixed(indent_width)),
                        icon,
                        text(name).size(13),
                    ]
                    .spacing(6)
                    .align_y(iced::Alignment::Center)
                )
                .style(tree_button_style)
                .on_press(Message::FolderToggled(path.clone()))
                .padding(iced::Padding { top: 6.0, right: 10.0, bottom: 6.0, left: 10.0 })
                .width(Length::Fill);

                items.push(btn.into());

                if is_expanded {
                    render_entries(children, tree, depth + 1, items);
                }
            }
            FileEntry::File { path, name } => {
                let icon_path = get_file_icon(name);

                let icon: Element<'_, Message> = icon_widget(&icon_path);

                let btn = button(
                    row![
                        container(text("")).width(Length::Fixed(indent_width)),
                        icon,
                        text(name).size(13),
                    ]
                    .spacing(6)
                    .align_y(iced::Alignment::Center)
                )
                .style(tree_button_style)
                .on_press(Message::FileClicked(path.clone()))
                .padding(iced::Padding { top: 6.0, right: 10.0, bottom: 6.0, left: 10.0 })
                .width(Length::Fill);

                items.push(btn.into());
            }
        }
    }
}
