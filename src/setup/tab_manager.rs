use eframe::egui;

#[derive(Clone)]
pub struct Tab {
    pub id: usize,
    pub title: String,
    pub text: String,
    pub file_path: Option<String>,
    pub cursor_pos: usize,
    pub saved_column: Option<usize>,
}

impl Tab {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            title: format!("Untitled {}", id),
            text: String::new(),
            file_path: None,
            cursor_pos: 0,
            saved_column: None,
        }
    }

    pub fn from_file(id: usize, path: String, content: String) -> Self {
        let title = std::path::Path::new(&path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled")
            .to_string();

        
        Self {
            id,
            title,
            text: content,
            file_path: Some(path),
            cursor_pos: 0,
            saved_column: None,
        }
    }

    pub fn update_title_from_path(&mut self) {
        if let Some(path) = &self.file_path {
            self.title = std::path::Path::new(path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Untitled")
                .to_string();
        }
    }
}

pub struct TabManager {
    pub tabs: Vec<Tab>,
    pub active_tab_index: usize,
    next_tab_id: usize,
}

impl Default for TabManager {
    fn default() -> Self {
        Self {
            tabs: vec![Tab::new(0)],
            active_tab_index: 0,
            next_tab_id: 1,
        }
    }
}

impl TabManager {
    pub fn new_tab(&mut self) {
        let new_tab = Tab::new(self.next_tab_id);
        self.next_tab_id += 1;
        self.tabs.push(new_tab);
        self.active_tab_index = self.tabs.len() - 1;
    }

    pub fn close_tab(&mut self, index: usize) {
        if self.tabs.len() <= 1 {
            // dont close the last tab, instead just clear it
            self.tabs[0] = Tab::new(self.next_tab_id);
            self.next_tab_id += 1;
            return;
        }

        self.tabs.remove(index);

        //adjust the active tab index
        if self.active_tab_index >= self.tabs.len() {
            self.active_tab_index = self.tabs.len() - 1;
        } else if self.active_tab_index > index {
            self.active_tab_index -= 1;
        }
    }

    pub fn close_active_tab(&mut self) {
        self.close_tab(self.active_tab_index);
    }

    pub fn get_active_tab(&self) -> &Tab {
        &self.tabs[self.active_tab_index]
    }

    pub fn get_active_tab_mut(&mut self) -> &mut Tab {
        &mut self.tabs[self.active_tab_index]
    }

    pub fn set_active_tab(&mut self, index: usize) {
        if index < self.tabs.len() {
            self.active_tab_index = index;
        }
    }

    pub fn show_tab_bar(&mut self, ctx: &egui::Context) -> egui::InnerResponse<()> {
        egui::TopBottomPanel::top("tab_bar")
            .exact_height(36.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;

                    let mut tab_to_close: Option<usize> = None;
                    let mut tab_to_activate: Option<usize> = None;

                    for (index, tab) in self.tabs.iter().enumerate() {
                        let is_active = index == self.active_tab_index;

                        //tab button
                        let button_color = if is_active {
                            ui.visuals().widgets.active.bg_fill
                        } else {
                            ui.visuals().widgets.inactive.bg_fill
                        };

                        let text_color = if is_active {
                            ui.visuals().strong_text_color()
                        } else {
                            ui.visuals().text_color()
                        };

                        ui.group(|ui| {
                            ui.style_mut().visuals.widgets.inactive.bg_fill = button_color;
                            ui.style_mut().visuals.widgets.hovered.bg_fill = button_color.linear_multiply(1.2);

                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 8.0;

                                //tab title button
                                let title_button = ui.add(
                                    egui::Button::new(
                                        egui::RichText::new(&tab.title)
                                            .color(text_color)
                                            .size(14.0)
                                    )
                                    .frame(false)
                                );

                                if title_button.clicked() {
                                    tab_to_activate = Some(index);
                                }

                                let close_button = ui.add(
                                    egui::Button::new(
                                        egui::RichText::new("×")
                                            .size(16.0)
                                            .color(text_color)
                                    )
                                    .frame(false)
                                    .small()
                                );

                                if close_button.clicked() {
                                    tab_to_close = Some(index);
                                }
                            });
                        });
                    }

                    //tab actions after iterating
                    if let Some(index) = tab_to_close {
                        self.close_tab(index);
                    }
                    if let Some(index) = tab_to_activate {
                        self.set_active_tab(index);
                    }
                });
            })
    }
}
