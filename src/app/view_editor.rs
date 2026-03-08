use super::*;
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
                    TabKind::Editor {
                        code_editor,
                        ..
                    } => {
                        return container(
                            code_editor.view().map(Message::CodeEditorEvent),
                        )
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .style(|_theme| container::Style {
                            background: Some(iced::Background::Color(theme().bg_editor)),
                            ..Default::default()
                        })
                        .into();
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

    pub(super) fn view_status_bar(&self) -> Element<'_, Message> {
        let file_info = self
            .active_tab
            .and_then(|idx| self.tabs.get(idx))
            .map(|tab| tab.name.clone())
            .unwrap_or_default();

        let left = row![text(file_info).size(10).color(theme().text_dim),]
            .spacing(8)
            .align_y(iced::Alignment::Center);
        let vim_mode_label = match self.vim_mode {
            VimMode::Normal => "NORMAL",
            VimMode::Insert => "INSERT",
        };
        let vim_state = if self.vim_pending.is_empty() && self.vim_count.is_empty() {
            vim_mode_label.to_string()
        } else {
            format!(
                "{vim_mode_label} {}{}",
                self.vim_count,
                self.vim_pending
            )
        };

        let current_line_diag = self
            .active_tab
            .and_then(|idx| self.tabs.get(idx))
            .map(|tab| tab.path.clone())
            .and_then(|path| self.lsp_diagnostics.get(&path))
            .and_then(|items| items.iter().find(|d| d.line == self.cursor_line))
            .map(|d| d.message.clone())
            .unwrap_or_default();

        let right = row![
            text(vim_state).size(10).color(theme().text_muted),
            text(format!("Ln {}, Col {}", self.cursor_line, self.cursor_col))
                .size(10)
                .color(theme().text_placeholder),
            text(current_line_diag).size(10).color(theme().text_secondary),
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
