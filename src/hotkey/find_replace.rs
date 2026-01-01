use eframe::egui;

pub struct FindReplace {
    pub open: bool,
    pub find_text: String,
    pub replace_text: String,
    pub case_sensitive: bool,
    pub match_count: usize,
    pub current_match: usize,
}

impl Default for FindReplace {
    fn default() -> Self {
        Self {
            open: false,
            find_text: String::new(),
            replace_text: String::new(),
            case_sensitive: false,
            match_count: 0,
            current_match: 0,
        }
    }
}

impl FindReplace {
    pub fn toggle(&mut self) {
        self.open = !self.open;
        if self.open {
            self.match_count = 0;
            self.current_match = 0;
        }
    }

    pub fn find_matches(&self, text: &str) -> Vec<usize> {
        if self.find_text.is_empty() {
            return Vec::new();
        }

        let mut matches = Vec::new();
        let search_text = if self.case_sensitive {
            text.to_string()
        } else {
            text.to_lowercase()
        };
        let find = if self.case_sensitive {
            self.find_text.clone()
        } else {
            self.find_text.to_lowercase()
        };

        let mut start = 0;
        while let Some(pos) = search_text[start..].find(&find) {
            matches.push(start + pos);
            start += pos + 1;
        }

        matches
    }

    pub fn replace_next(&mut self, text: &mut String) -> bool {
        let matches = self.find_matches(text);
        if matches.is_empty() || self.current_match >= matches.len() {
            return false;
        }

        let pos = matches[self.current_match];
        let end = pos + self.find_text.len();
        text.replace_range(pos..end, &self.replace_text);
        true
    }

    pub fn replace_all(&mut self, text: &mut String) -> usize {
        if self.find_text.is_empty() {
            return 0;
        }

        let matches = self.find_matches(text);
        let count = matches.len();

        for (i, &pos) in matches.iter().enumerate().rev() {
            let offset = i * (self.replace_text.len().saturating_sub(self.find_text.len()));
            let adjusted_pos = if self.replace_text.len() > self.find_text.len() {
                pos + offset
            } else {
                pos.saturating_sub(offset)
            };
            let end = adjusted_pos + self.find_text.len();
            text.replace_range(adjusted_pos..end, &self.replace_text);
        }

        count
    }

    pub fn show(&mut self, ctx: &egui::Context, text: &mut String) {
        if !self.open {
            return;
        }

        self.match_count = self.find_matches(text).len();

        egui::Window::new("Find and Replace")
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_TOP, egui::vec2(0.0, 50.0))
            .fixed_size(egui::vec2(500.0, 200.0))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Find:");
                    let find_response = ui.add(
                        egui::TextEdit::singleline(&mut self.find_text)
                            .desired_width(350.0)
                    );

                    if self.open && find_response.has_focus() {
                        find_response.request_focus();
                    }

                    if find_response.changed() {
                        self.current_match = 0;
                    }
                });

                ui.horizontal(|ui| {
                    ui.label("Replace:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.replace_text)
                            .desired_width(350.0)
                    );
                });

                ui.horizontal(|ui| {
                    ui.checkbox(&mut self.case_sensitive, "Case sensitive");

                    if self.match_count > 0 {
                        ui.label(format!("{} matches found", self.match_count));
                    }
                });

                ui.add_space(10.0);

                ui.horizontal(|ui| {
                    if ui.button("Find next").clicked() && self.match_count > 0 {
                        self.current_match = (self.current_match + 1) % self.match_count;
                    }

                    if ui.button("Replace").clicked() {
                        self.replace_next(text);
                    }

                    if ui.button("Replace all").clicked() {
                        self.replace_all(text);
                        self.match_count = 0;
                        self.current_match = 0;
                    }

                    if ui.button("Close").clicked() {
                        self.open = false;
                    }
                });

                if ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.open = false;
                }
            });
    }
}