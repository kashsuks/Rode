use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_title("CatEditor"),
        ..Default::default()
    };

    eframe::run_native(
        "CatEditor",
        options,
        Box::new(|_cc| Ok(Box::new(CatEditorApp::default()))),
    )
}

struct CatEditorApp {
    text: String,
}

impl Default for CatEditorApp {
    fn default() -> Self {
        Self {
            text: String::new(),
        }
    }
}

impl eframe::App for CatEditorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut self.text)
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Monospace)
                );
            });
        });
    }
}