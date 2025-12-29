use eframe::egui;

pub fn apply_theme(ctx: &egui::Context, app: &crate::app::CatEditorApp) {
    let rosewater = c(&app.color_rosewater);
    let flamingo = c(&app.color_flamingo);
    let pink = c(&app.color_pink);
    let mauve = c(&app.color_mauve);
    let red = c(&app.color_red);
    let maroon = c(&app.color_maroon);
    let peach = c(&app.color_peach);
    let yellow = c(&app.color_yellow);
    let green = c(&app.color_green);
    let teal = c(&app.color_teal);
    let sky = c(&app.color_sky);
    let sapphire = c(&app.color_sapphire);
    let blue = c(&app.color_blue);
    let lavender = c(&app.color_lavender);

    let text = c(&app.color_text);
    let subtext1 = c(&app.color_subtext1);
    let subtext0 = c(&app.color_subtext0);
    let overlay2 = c(&app.color_overlay2);
    let overlay1 = c(&app.color_overlay1);
    let overlay0 = c(&app.color_overlay0);

    let surface2 = c(&app.color_surface2);
    let surface1 = c(&app.color_surface1);
    let surface0 = c(&app.color_surface0);

    let base = c(&app.color_base);
    let mantle = c(&app.color_mantle);
    let crust = c(&app.color_crust);

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
    
    // Pad with zeros if incomplete
    let mut padded = h.to_string();
    while padded.len() < 6 {
        padded.push('0');
    }
    
    let r = u8::from_str_radix(&padded.get(0..2).unwrap_or("00"), 16).unwrap_or(0);
    let g = u8::from_str_radix(&padded.get(2..4).unwrap_or("00"), 16).unwrap_or(0);
    let b = u8::from_str_radix(&padded.get(4..6).unwrap_or("00"), 16).unwrap_or(0);
    (r, g, b)
}