use eframe::egui;

pub fn apply_catppuccin(ctx: &egui::Context) {
    let rosewater = c("#f5e0dc");
    let flamingo = c("#f2cdcd");
    let pink = c("#f5c2e7");
    let mauve = c("#cba6f7");
    let red = c("#f38ba8");
    let maroon = c("#eba0ac");
    let peach = c("#fab387");
    let yellow = c("#f9e2af");
    let green = c("#a6e3a1");
    let teal = c("#94e2d5");
    let sky = c("#89dceb");
    let sapphire = c("#74c7ec");
    let blue = c("#89b4fa");
    let lavender = c("#b4befe");

    let text = c("#cdd6f4");
    let subtext1 = c("#bac2de");
    let subtext0 = c("#a6adc8");
    let overlay2 = c("#9399b2");
    let overlay1 = c("#7f849c");
    let overlay0 = c("#6c7086");

    let surface2 = c("#585b70");
    let surface1 = c("#45475a");
    let surface0 = c("#313244");

    let base = c("#1e1e2e");
    let mantle = c("#181825");
    let crust = c("#11111b");

    let mut visuals = egui::Visuals::dark();

    visuals.panel_fill = base;
    visuals.faint_bg_color = surface0;

    visuals.hyperlink_color = blue;
    visuals.selection.bg_fill = surface2;
    visuals.selection.stroke = egui::Stroke::new(1.0, lavender);

    visuals.widgets.noninteractive.bg_fill = base;
    visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(1.0, subtext1);

    visuals.widgets.inactive.bg_fill = surface0;
    visuals.widgets.inactive.fg_stroke = egui::Stroke::new(1.0, text);
    visuals.widgets.inactive.bg_stroke = egui::Stroke::new(1.0, surface1);

    visuals.widgets.hovered.bg_fill = surface1;
    visuals.widgets.hovered.fg_stroke = egui::Stroke::new(1.0, rosewater);
    visuals.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, overlay1);

    visuals.widgets.active.bg_fill = surface2;
    visuals.widgets.active.fg_stroke = egui::Stroke::new(1.0, text);
    visuals.widgets.active.bg_stroke = egui::Stroke::new(1.0, lavender);

    visuals.error_fg_color = red;
    visuals.warn_fg_color = yellow;

    ctx.set_visuals(visuals);

    ctx.style_mut(|style| {
        style.visuals.window_rounding = egui::Rounding::same(10.0);
        style.visuals.menu_rounding = egui::Rounding::same(10.0);
        style.visuals.popup_shadow = egui::Shadow {
            offset: egui::vec2(0.0, 4.0),
            blur: 12.0,
            spread: 0.0,
            color: egui::Color32::from_black_alpha(120),
        };

        style.spacing.item_spacing = egui::vec2(8.0, 6.0);
        style.spacing.window_margin = egui::Margin::same(10.0);

        style.visuals.text_cursor.stroke = egui::Stroke::new(1.5, flamingo);

        style.spacing.button_padding = egui::vec2(10.0, 6.0);
    });
}

fn c(hex: &str) -> egui::Color32 {
    let (r, g, b) = parse_hex_rgb(hex);
    egui::Color32::from_rgb(r, g, b)
}

fn parse_hex_rgb(hex: &str) -> (u8, u8, u8) {
    let h = hex.trim().trim_start_matches('#');
    let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(0);
    let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(0);
    (r, g, b)
}