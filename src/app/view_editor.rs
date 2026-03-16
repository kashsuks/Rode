use super::*;
use frostmark::MarkWidget;
use iced::widget::column;

impl App {
    pub(super) fn view_tab_bar(&self) -> Element<'_, Message> {
        if self.tabs.is_empty() {
            return container(text("")).into();
        }

        let tabs: Vec<Element<'_, Message>> = self
            .tabs
            .iter()
            .enumerate()
            .map(|(idx, tab)| {
                let is_active = self.active_tab == Some(idx);
                let is_modified = matches!(&tab.kind, TabKind::Editor { code_editor, .. } if code_editor.is_modified());
                let close_icon = if is_modified {
                    text("●").size(10).color(theme().text_muted)
                } else {
                    text("x").size(10).color(theme().text_dim)
                };

                button(
                    row![
                        text(&tab.name).size(12).color(theme().text_muted),
                        button(close_icon)
                            .style(tab_close_button_style)
                            .on_press(Message::TabClosed(idx))
                            .padding(2),
                    ]
                    .spacing(8)
                    .align_y(iced::Alignment::Center),
                )
                .style(tab_button_style(is_active))
                .on_press(Message::TabSelected(idx))
                .padding(iced::Padding {
                    top: 8.0,
                    right: 16.0,
                    bottom: 8.0,
                    left: 16.0,
                })
                .into()
            })
            .collect();

        container(row(tabs).spacing(6))
            .padding(iced::Padding {
                top: 8.0,
                right: 12.0,
                bottom: 8.0,
                left: 12.0,
            })
            .width(Length::Fill)
            .style(tab_bar_style)
            .into()
    }

    pub(super) fn view_search_panel(&self) -> Element<'_, Message> {
        let input = text_input("Search across workspace...", &self.search_query)
            .id(self.search_input_id.clone())
            .on_input(Message::SearchQueryChanged)
            .style(search_input_style)
            .size(13)
            .padding(10)
            .width(Length::Fill);

        let mut content_col = column![input].spacing(6);

        if !self.search_results.is_empty() {
            let mut result_items: Vec<Element<'_, Message>> = Vec::new();

            for result in &self.search_results {
                result_items.push(
                    container(
                        text(&result.file_name)
                            .size(11)
                            .color(theme().text_secondary),
                    )
                    .padding(iced::Padding {
                        top: 6.0,
                        right: 6.0,
                        bottom: 2.0,
                        left: 6.0,
                    })
                    .into(),
                );

                for m in result.matches.iter().take(3) {
                    let line_text = format!("  {}:  {}", m.line_number, m.line_content.trim());
                    let path = result.path.clone();
                    let line_num = m.line_number;

                    result_items.push(
                        button(text(line_text).size(11).color(theme().text_muted))
                            .style(tree_button_style)
                            .on_press(Message::SearchResultClicked(path, line_num))
                            .padding(iced::Padding {
                                top: 3.0,
                                right: 6.0,
                                bottom: 3.0,
                                left: 12.0,
                            })
                            .width(Length::Fill)
                            .into(),
                    );
                }

                if result.matches.len() > 3 {
                    result_items.push(
                        container(
                            text(format!("  ... and {} more", result.matches.len() - 3))
                                .size(10)
                                .color(theme().text_dim),
                        )
                        .padding(iced::Padding {
                            top: 1.0,
                            right: 6.0,
                            bottom: 2.0,
                            left: 12.0,
                        })
                        .into(),
                    );
                }
            }

            let results_scroll = scrollable(column(result_items).spacing(1)).height(Length::Shrink);

            content_col = content_col.push(container(results_scroll).max_height(400.0));
        }

        container(content_col)
            .width(Length::Fixed(320.0))
            .padding(10)
            .style(search_panel_style)
            .into()
    }

    pub(super) fn view_editor(&self) -> Element<'_, Message> {
        if self.pending_sensitive_open.is_some() {
            return container(
                column![
                    text("You are opening a sensitive file, continue?")
                        .size(18)
                        .color(theme().text_muted),
                    row![
                        button(text("Yes").size(13))
                            .on_press(Message::SensitiveFileOpenConfirm(true))
                            .padding(iced::Padding {
                                top: 8.0,
                                right: 16.0,
                                bottom: 8.0,
                                left: 16.0,
                            }),
                        button(text("No").size(13))
                            .on_press(Message::SensitiveFileOpenConfirm(false))
                            .padding(iced::Padding {
                                top: 8.0,
                                right: 16.0,
                                bottom: 8.0,
                                left: 16.0,
                            }),
                    ]
                    .spacing(12)
                    .align_y(iced::Alignment::Center),
                ]
                .spacing(16)
                .align_x(iced::Alignment::Center),
            )
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        }

        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get(idx) {
                match &tab.kind {
                    TabKind::Editor { code_editor, .. } => {
                        let editor = container(code_editor.view().map(Message::CodeEditorEvent))
                            .width(Length::Fill)
                            .height(Length::Fill)
                            .style(|_theme| container::Style {
                                background: Some(iced::Background::Color(theme().bg_editor)),
                                ..Default::default()
                            });

                        let lsp_overlay = if self.lsp_enabled {
                            iced_code_editor::view_lsp_overlay(
                                &self.lsp_overlay,
                                code_editor,
                                &iced::Theme::CatppuccinMocha,
                                13.0,
                                20.0,
                                Message::LspOverlay,
                            )
                        } else {
                            container(iced::widget::Space::new()).into()
                        };

                        let show_panel = !self.lsp_enabled
                            && self.autocomplete.active
                            && !self.autocomplete.suggestions.is_empty();
                        let editor_stack: Element<'_, Message> = if show_panel {
                            // ── Purple autocomplete modal with navigation ───────────────────
                            let kind_color = |kind: &crate::autocomplete::types::SuggestionKind| {
                                use crate::autocomplete::types::SuggestionKind;
                                match kind {
                                    SuggestionKind::Keyword => Color::from_rgb(0.796, 0.651, 0.969),
                                    SuggestionKind::Function => {
                                        Color::from_rgb(0.000, 0.663, 1.000)
                                    }
                                    SuggestionKind::Method => Color::from_rgb(0.537, 0.863, 0.922),
                                    SuggestionKind::Type => Color::from_rgb(0.976, 0.886, 0.686),
                                    SuggestionKind::Constant => {
                                        Color::from_rgb(0.976, 0.886, 0.686)
                                    }
                                    SuggestionKind::Variable => {
                                        Color::from_rgb(0.706, 0.745, 0.996)
                                    }
                                    SuggestionKind::Property => {
                                        Color::from_rgb(0.000, 0.663, 1.000)
                                    }
                                    SuggestionKind::Module => Color::from_rgb(0.537, 0.863, 0.922),
                                    SuggestionKind::Macro => Color::from_rgb(0.000, 1.000, 0.824),
                                    SuggestionKind::Snippet => Color::from_rgb(0.976, 0.886, 0.686),
                                }
                            };

                            let accent_purple = Color::from_rgb(0.796, 0.651, 0.969);
                            let bg_modal = Color::from_rgb(0.149, 0.149, 0.212);
                            let bg_selected = Color::from_rgba(0.796, 0.651, 0.969, 0.18);
                            let divider = Color::from_rgba(1.0, 1.0, 1.0, 0.06);

                            let mut items: Vec<Element<'_, Message>> = Vec::new();
                            let visible_count = self.autocomplete.suggestions.len().min(8);

                            for (i, suggestion) in self
                                .autocomplete
                                .suggestions
                                .iter()
                                .take(visible_count)
                                .enumerate()
                            {
                                let is_selected = i == self.autocomplete.selected_index;
                                let ic = kind_color(&suggestion.kind);
                                let label_color = if is_selected {
                                    theme().text_primary
                                } else {
                                    theme().text_muted
                                };
                                let row_bg = if is_selected {
                                    Some(iced::Background::Color(bg_selected))
                                } else {
                                    None
                                };
                                items.push(
                                    container(
                                        row![
                                            container(
                                                text(suggestion.kind.icon()).size(11).color(ic)
                                            )
                                            .width(Length::Fixed(20.0))
                                            .center_x(Length::Fixed(20.0)),
                                            text(&suggestion.text).size(12).color(label_color),
                                            iced::widget::Space::new().width(Length::Fill),
                                            text(format!("{:?}", suggestion.kind).to_lowercase())
                                                .size(10)
                                                .color(Color::from_rgba(ic.r, ic.g, ic.b, 0.65)),
                                        ]
                                        .spacing(6)
                                        .align_y(iced::Alignment::Center),
                                    )
                                    .padding(iced::Padding {
                                        top: 4.0,
                                        right: 10.0,
                                        bottom: 4.0,
                                        left: 8.0,
                                    })
                                    .width(Length::Fill)
                                    .style(move |_theme| container::Style {
                                        background: row_bg,
                                        border: iced::Border {
                                            color: Color::TRANSPARENT,
                                            width: 0.0,
                                            radius: 4.0.into(),
                                        },
                                        ..Default::default()
                                    })
                                    .into(),
                                );
                            }

                            // Navigation footer
                            items.push(
                                container(
                                    container(
                                        row![
                                            text("↑↓").size(9).color(accent_purple),
                                            text(" navigate · ").size(9).color(theme().text_dim),
                                            text("↵").size(9).color(accent_purple),
                                            text(" accept · ").size(9).color(theme().text_dim),
                                            text("esc").size(9).color(accent_purple),
                                            text(" dismiss").size(9).color(theme().text_dim),
                                        ]
                                        .spacing(0)
                                        .align_y(iced::Alignment::Center),
                                    )
                                    .padding(iced::Padding {
                                        top: 4.0,
                                        right: 8.0,
                                        bottom: 4.0,
                                        left: 8.0,
                                    })
                                    .width(Length::Fill)
                                    .style(move |_theme| {
                                        container::Style {
                                            background: Some(iced::Background::Color(
                                                Color::from_rgba(0.796, 0.651, 0.969, 0.06),
                                            )),
                                            border: iced::Border {
                                                color: divider,
                                                width: 1.0,
                                                radius: 0.0.into(),
                                            },
                                            ..Default::default()
                                        }
                                    }),
                                )
                                .width(Length::Fill)
                                .into(),
                            );

                            let panel = container(column(items).spacing(1))
                                .padding(4)
                                .max_width(320.0)
                                .style(move |_theme| container::Style {
                                    background: Some(iced::Background::Color(bg_modal)),
                                    border: iced::Border {
                                        color: Color::from_rgba(
                                            accent_purple.r,
                                            accent_purple.g,
                                            accent_purple.b,
                                            0.35,
                                        ),
                                        width: 1.0,
                                        radius: 8.0.into(),
                                    },
                                    shadow: iced::Shadow {
                                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.6),
                                        offset: iced::Vector::new(0.0, 4.0),
                                        blur_radius: 20.0,
                                    },
                                    ..Default::default()
                                });

                            let cursor_pos = code_editor
                                .cursor_screen_position()
                                .unwrap_or(iced::Point::new(48.0, 20.0));
                            let x = cursor_pos.x.clamp(0.0, 500.0);
                            let y = (cursor_pos.y + 20.0).clamp(0.0, 560.0);

                            let positioned_panel = container(panel)
                                .padding(iced::Padding {
                                    top: y,
                                    left: x,
                                    bottom: 0.0,
                                    right: 0.0,
                                })
                                .width(Length::Fill)
                                .height(Length::Fill);

                            stack![editor, positioned_panel, lsp_overlay]
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .into()
                        } else {
                            stack![editor, lsp_overlay]
                                .width(Length::Fill)
                                .height(Length::Fill)
                                .into()
                        };

                        if let Some(preview) = self
                            .markdown_preview
                            .as_ref()
                            .filter(|preview| preview.source_path == tab.path)
                        {
                            let separator = container(text(""))
                                .width(Length::Fixed(1.0))
                                .height(Length::Fill)
                                .style(|_theme| container::Style {
                                    background: Some(iced::Background::Color(Color::from_rgba(
                                        1.0, 1.0, 1.0, 0.08,
                                    ))),
                                    ..Default::default()
                                });

                            let preview_panel = container(
                                scrollable(
                                    container(MarkWidget::new(&preview.state))
                                        .padding(16)
                                        .width(Length::Fill),
                                )
                                .height(Length::Fill),
                            )
                            .width(Length::FillPortion(1))
                            .height(Length::Fill)
                            .style(|_theme| container::Style {
                                background: Some(iced::Background::Color(theme().bg_secondary)),
                                ..Default::default()
                            });

                            return row![
                                container(editor_stack)
                                    .width(Length::FillPortion(1))
                                    .height(Length::Fill),
                                separator,
                                preview_panel,
                            ]
                            .height(Length::Fill)
                            .into();
                        }

                        return editor_stack;
                    }
                    TabKind::Preview { md_items } => {
                        return scrollable(
                            markdown::view(
                                md_items,
                                markdown::Settings::with_style(markdown::Style::from_palette(
                                    iced::theme::Palette::CATPPUCCIN_MOCHA,
                                )),
                            )
                            .map(Message::MarkdownLinkClicked),
                        )
                        .height(Length::Fill)
                        .into();
                    }
                }
            }
        }
        empty_editor()
    }

    pub(super) fn view_terminal_panel(&self) -> Element<'_, Message> {
        let height = Length::Fixed(self.terminal_panel_height);

        if let Some(term) = &self.terminal_pane {
            let header = container(
                row![
                    text("Terminal").size(12).color(theme().text_muted),
                    iced::widget::Space::new().width(Length::Fill),
                    button(text("x").size(12).color(theme().text_dim))
                        .style(tab_close_button_style)
                        .on_press(Message::ToggleTerminal),
                ]
                .align_y(iced::Alignment::Center),
            )
            .padding(iced::Padding {
                top: 6.0,
                right: 8.0,
                bottom: 6.0,
                left: 10.0,
            })
            .style(|_theme| container::Style {
                background: Some(Background::Color(theme().bg_secondary)),
                border: iced::Border {
                    color: theme().border_subtle,
                    width: 1.0,
                    radius: 0.0.into(),
                },
                ..Default::default()
            });

            let body = container(iced_term::TerminalView::show(term).map(Message::TerminalEvent))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|_theme| container::Style {
                    background: Some(Background::Color(theme().bg_editor)),
                    ..Default::default()
                });

            return container(column![header, body].spacing(0))
                .width(Length::Fill)
                .height(height)
                .into();
        }

        container(
            text("Embedded terminal unavailable")
                .size(12)
                .color(theme().text_dim),
        )
        .padding(10)
        .width(Length::Fill)
        .height(height)
        .into()
    }

    pub(super) fn view_status_bar(&self) -> Element<'_, Message> {
        let file_info = self
            .active_tab
            .and_then(|idx| self.tabs.get(idx))
            .map(|tab| tab.name.clone())
            .unwrap_or_default();

        let left = row![text(file_info).size(10).color(theme().text_dim),]
            .spacing(8)
            .align_y(iced::Alignment::Center);

        let current_line_diag = self
            .active_tab
            .and_then(|idx| self.tabs.get(idx))
            .map(|tab| tab.path.clone())
            .and_then(|path| self.lsp_diagnostics.get(&path))
            .and_then(|items| items.iter().find(|d| d.line == self.cursor_line))
            .map(|d| d.message.clone())
            .unwrap_or_default();

        let right = row![
            text(format!("Ln {}, Col {}", self.cursor_line, self.cursor_col))
                .size(10)
                .color(theme().text_placeholder),
            text(current_line_diag)
                .size(10)
                .color(theme().text_secondary),
        ]
        .spacing(8)
        .align_y(iced::Alignment::Center);

        container(
            row![left, iced::widget::Space::new().width(Length::Fill), right,]
                .align_y(iced::Alignment::Center),
        )
        .padding(iced::Padding {
            top: 4.0,
            right: 12.0,
            bottom: 6.0,
            left: 12.0,
        })
        .width(Length::Fill)
        .style(status_bar_style)
        .into()
    }

    pub(super) fn view_welcome_screen(&self) -> iced::widget::Container<'_, Message> {
        let folder_name = self
            .file_tree
            .as_ref()
            .map(|t| {
                t.root
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string()
            })
            .unwrap_or_else(|| String::from("No folder open"));

        container(
            column![
                text(folder_name).size(24).color(theme().text_muted),
                text("Select a file from the sidebar to begin editing")
                    .size(13)
                    .color(theme().text_placeholder),
            ]
            .spacing(12)
            .align_x(iced::Alignment::Center),
        )
        .center_x(Length::Fill)
        .center_y(Length::Fill)
    }
}
