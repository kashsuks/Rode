use super::*;

impl App {
    /// Builds the root application view tree.
    pub fn view(&self) -> Element<'_, Message> {
        use iced::widget::stack;

        let editor_area: Element<'_, Message> = if self.settings_open {
            self.view_settings_panel()
        } else {
            let tab_bar = self.view_tab_bar();
            let editor_widget = self.view_editor();
            let status_bar = self.view_status_bar();

            let mut editor_col_items: Vec<Element<'_, Message>> = Vec::new();
            if self.find_replace.open {
                editor_col_items.push(self.view_find_replace_panel());
            }
            editor_col_items.push(tab_bar);
            editor_col_items.push(editor_widget);
            if self.command_input.open {
                editor_col_items.push(self.view_command_input_bar());
            }
            editor_col_items.push(status_bar);

            let editor_container = if self.active_tab.is_some() || self.pending_sensitive_open.is_some() {
                container(column(editor_col_items))
            } else {
                self.view_welcome_screen()
            }
            .width(Length::Fill)
            .height(Length::Fill)
            .style(editor_container_style);

            container(editor_container)
                .padding(0)
                .width(Length::Fill)
                .into()
        };

        let base_content: Element<'_, Message> = if self.sidebar_visible {
            let sidebar = view_sidebar(self.file_tree.as_ref(), self.sidebar_width);

            let separator = container(text(""))
                .width(Length::Fixed(1.0))
                .height(Length::Fill)
                .style(sidebar_editor_separator_style);

            let resize_zone = mouse_area(
                container(text(""))
                    .width(Length::Fixed(4.0))
                    .height(Length::Fill),
            )
            .on_press(Message::SidebarResizeStart)
            .interaction(iced::mouse::Interaction::ResizingHorizontally);

            row![sidebar, separator, resize_zone, editor_area].into()
        } else {
            editor_area.into()
        };

        let wrapped = container(base_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(|_theme| container::Style {
                background: Some(Background::Color(theme().bg_editor)),
                ..Default::default()
            });

        let base_view: Element<'_, Message> = if self.command_palette.open {
            stack![wrapped, self.view_command_palette_overlay()].into()
        } else if self.fuzzy_finder.open {
            stack![wrapped, self.view_fuzzy_finder_overlay()].into()
        } else if self.file_finder_visible {
            stack![wrapped, self.view_file_finder_overlay()].into()
        } else if self.search_visible {
            let search_panel = container(self.view_search_panel())
                .padding(iced::Padding {
                    top: 20.0,
                    right: 0.0,
                    bottom: 0.0,
                    left: 20.0,
                })
                .width(Length::Fill)
                .height(Length::Fill);
            stack![wrapped, search_panel].into()
        } else {
            wrapped.into()
        };

        let with_notification: Element<'_, Message> = if self.notification.is_some() {
            stack![base_view, self.view_notification_toast()].into()
        } else {
            base_view
        };

        if self.update_banner.is_some() {
            stack![with_notification, self.view_update_banner()].into()
        } else {
            with_notification
        }
    }
}
