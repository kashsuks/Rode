use iced::widget::button::{Style as ButtonStyle, Status as ButtonStatus};
use iced::widget::container;
use iced::widget::text_editor;
use iced::border::Radius;
use iced::{Background, Border, Color, Theme, Vector};

use crate::theme::*;

fn lighten(color: Color, amount: f32) -> Color {
    Color::from_rgba(
        (color.r + amount).min(1.0),
        (color.g + amount).min(1.0),
        (color.b + amount).min(1.0),
        color.a,
    )
}

pub fn tree_button_style(_theme: &Theme, status: ButtonStatus) -> ButtonStyle {
    let background = match status {
        ButtonStatus::Hovered => Some(Background::Color(theme().bg_hover)),
        ButtonStatus::Pressed => Some(Background::Color(theme().bg_pressed)),
        _ => None,
    };

    ButtonStyle {
        background,
        text_color: theme().text_secondary,
        border: Border::default(),
        shadow: Default::default(),
        snap: false,
    }
}

pub fn tab_button_style(is_active: bool) -> impl Fn(&Theme, ButtonStatus) -> ButtonStyle {
    move |_theme, status| {
        let (background, text_color) = if is_active {
            (
                Some(Background::Color(lighten(theme().bg_tab_bar, 0.08))),
                theme().text_primary,
            )
        } else {
            let bg = match status {
                ButtonStatus::Hovered => Some(Background::Color(lighten(theme().bg_tab_bar, 0.04))),
                _ => None,
            };
            (bg, theme().text_dim)
        };
        ButtonStyle {
            background,
            text_color,
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: BORDER_RADIUS_TAB.into(),
            },
            shadow: Default::default(),
            snap: false,
        }
    }
}

pub fn tab_close_button_style(_theme: &Theme, _status: ButtonStatus) -> ButtonStyle {
    ButtonStyle {
        background: None,
        text_color: theme().text_dim,
        border: Border::default(),
        shadow: Default::default(),
        snap: false,
    }
}

pub fn editor_container_style(_theme: &Theme) -> container::Style {
    let t = theme();
    container::Style {
        background: Some(Background::Color(t.bg_primary)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn sidebar_editor_separator_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme().border_subtle)),
        border: Border::default(),
        ..Default::default()
    }
}

pub fn sidebar_container_style(_theme: &Theme) -> container::Style {
    let t = theme();
    container::Style {
        background: Some(Background::Color(t.bg_secondary)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        ..Default::default()
    }
}

pub fn status_bar_style(_theme: &Theme) -> container::Style {
    let bg = theme().bg_status_bar;
    let bg_subtle = Color::from_rgba(bg.r, bg.g, bg.b, bg.a * 0.5);
    container::Style {
        background: Some(Background::Color(bg_subtle)),
        border: Border {
            color: theme().border_very_subtle,
            width: 0.0,
            radius: Radius { top_left: 0.0, top_right: 0.0, bottom_right: 0.0, bottom_left: 0.0 },
        },
        ..Default::default()
    }
}

pub fn tab_bar_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(theme().bg_tab_bar)),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: Radius { top_left: 0.0, top_right: 0.0, bottom_right: 0.0, bottom_left: 0.0 },
        },
        ..Default::default()
    }
}

pub fn text_editor_style(_theme: &Theme, _status: text_editor::Status) -> text_editor::Style {
    text_editor::Style {
        background: Background::Color(theme().bg_editor),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        placeholder: theme().text_placeholder,
        value: theme().text_primary,
        selection: theme().selection,
    }
}

pub fn drag_handle_style(_theme: &Theme, status: ButtonStatus) -> ButtonStyle {
    let background = match status {
        ButtonStatus::Hovered => Some(Background::Color(theme().bg_hover)),
        ButtonStatus::Pressed => Some(Background::Color(theme().bg_pressed)),
        _ => Some(Background::Color(theme().bg_drag_handle)),
    };

    ButtonStyle {
        background,
        text_color: Color::TRANSPARENT,
        border: Border::default(),
        shadow: Default::default(),
        snap: false,
    }
}

pub fn search_panel_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(
            theme().bg_primary.r,
            theme().bg_primary.g,
            theme().bg_primary.b,
            0.97,
        ))),
        border: Border {
            color: theme().border_subtle,
            width: 1.0,
            radius: BORDER_RADIUS.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
            offset: Vector::new(4.0, 4.0),
            blur_radius: 24.0,
        },
        ..Default::default()
    }
}

pub fn search_input_style(_theme: &Theme, _status: iced::widget::text_input::Status) -> iced::widget::text_input::Style {
    iced::widget::text_input::Style {
        background: Background::Color(Color::TRANSPARENT),
        border: Border {
            color: Color::TRANSPARENT,
            width: 0.0,
            radius: 0.0.into(),
        },
        icon: theme().text_dim,
        placeholder: theme().text_placeholder,
        value: theme().text_primary,
        selection: theme().selection,
    }
}

pub fn file_finder_panel_style(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(Color::from_rgba(
            (theme().bg_primary.r + 0.04).min(1.0),
            (theme().bg_primary.g + 0.04).min(1.0),
            (theme().bg_primary.b + 0.07).min(1.0),
            0.97,
        ))),
        border: Border {
            color: Color::from_rgba(1.0, 1.0, 1.0, 0.10),
            width: 1.0,
            radius: 18.0.into(),
        },
        shadow: iced::Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.7),
            offset: Vector::new(0.0, 24.0),
            blur_radius: 80.0,
        },
        ..Default::default()
    }
}

pub fn file_finder_item_style(is_selected: bool) -> impl Fn(&Theme, ButtonStatus) -> ButtonStyle {
    move |_theme, status| {
        let background = if is_selected {
            Some(Background::Color(theme().bg_pressed))
        } else {
            match status {
                ButtonStatus::Hovered => Some(Background::Color(theme().bg_hover)),
                _ => None,
            }
        };

        ButtonStyle {
            background,
            text_color: if is_selected { theme().text_primary } else { theme().text_muted },
            border: Border {
                color: Color::TRANSPARENT,
                width: 0.0,
                radius: 8.0.into(),
            },
            shadow: Default::default(),
            snap: false,
        }
    }
}