use iced::Color;
use iced_code_editor::theme::Style as CodeEditorStyle;
use once_cell::sync::Lazy;
use std::str::FromStr;
use std::sync::RwLock;

use syntect::highlighting::{
    Color as SynColor, FontStyle, ScopeSelectors, StyleModifier, Theme as SynTheme, ThemeItem,
    ThemeSettings,
};

// ── Layout constants ────────────────────────────────────────────────────────
pub const SIDEBAR_DEFAULT_WIDTH: f32 = 180.0;
pub const SIDEBAR_MIN_WIDTH: f32 = 100.0;
pub const SIDEBAR_MAX_WIDTH: f32 = 500.0;
pub const RESIZE_HIT_WIDTH: f32 = 12.0;
pub const ICON_SIZE: f32 = 16.0;
pub const INDENT_WIDTH: f32 = 16.0;
pub const BORDER_RADIUS: f32 = 14.0;
pub const BORDER_RADIUS_TAB: f32 = 10.0;

// ═══════════════════════════════════════════════════════════════════════════
// PALETTE – Generic color slots.  Swap these values to re-theme the editor.
// ═══════════════════════════════════════════════════════════════════════════

// -- Accent colours (warm → cool) --
pub const ACCENT_WARM_1: Color = Color::from_rgb(0.961, 0.878, 0.863); // #f5e0dc
pub const ACCENT_WARM_2: Color = Color::from_rgb(0.949, 0.804, 0.804); // #f2cdcd
pub const ACCENT_PINK: Color = Color::from_rgb(0.961, 0.761, 0.906); // #f5c2e7
pub const ACCENT_PURPLE: Color = Color::from_rgb(0.796, 0.651, 0.969); // #cba6f7
pub const ACCENT_RED: Color = Color::from_rgb(0.953, 0.545, 0.659); // #f38ba8
pub const ACCENT_DARK_RED: Color = Color::from_rgb(0.922, 0.627, 0.675); // #eba0ac
pub const ACCENT_ORANGE: Color = Color::from_rgb(0.980, 0.702, 0.529); // #fab387
pub const ACCENT_YELLOW: Color = Color::from_rgb(0.976, 0.886, 0.686); // #f9e2af
pub const ACCENT_GREEN: Color = Color::from_rgb(0.651, 0.890, 0.631); // #a6e3a1
pub const ACCENT_TEAL: Color = Color::from_rgb(0.580, 0.886, 0.835); // #94e2d5
pub const ACCENT_SKY: Color = Color::from_rgb(0.537, 0.863, 0.922); // #89dceb
pub const ACCENT_MID_BLUE: Color = Color::from_rgb(0.455, 0.780, 0.925); // #74c7ec
pub const ACCENT_BLUE: Color = Color::from_rgb(0.537, 0.706, 0.980); // #89b4fa
pub const ACCENT_SOFT_BLUE: Color = Color::from_rgb(0.706, 0.745, 0.996); // #b4befe

// -- Text hierarchy --
pub const TEXT_1: Color = Color::from_rgb(0.804, 0.839, 0.957); // #cdd6f4
pub const TEXT_2: Color = Color::from_rgb(0.729, 0.761, 0.871); // #bac2de
pub const TEXT_3: Color = Color::from_rgb(0.651, 0.678, 0.784); // #a6adc8

// -- Overlay layers --
pub const OVERLAY_3: Color = Color::from_rgb(0.576, 0.600, 0.698); // #9399b2
pub const OVERLAY_2: Color = Color::from_rgb(0.498, 0.518, 0.612); // #7f849c
pub const OVERLAY_1: Color = Color::from_rgb(0.424, 0.439, 0.525); // #6c7086

// -- Surface layers --
pub const SURFACE_3: Color = Color::from_rgb(0.345, 0.357, 0.439); // #585b70
pub const SURFACE_2: Color = Color::from_rgb(0.271, 0.278, 0.353); // #45475a
pub const SURFACE_1: Color = Color::from_rgb(0.192, 0.196, 0.267); // #313244

// -- Background layers --
pub const BG_BASE: Color = Color::from_rgb(0.118, 0.118, 0.180); // #1e1e2e
pub const BG_MANTLE: Color = Color::from_rgb(0.094, 0.094, 0.145); // #181825
pub const BG_CRUST: Color = Color::from_rgb(0.067, 0.067, 0.106); // #11111b

// ═══════════════════════════════════════════════════════════════════════════
// ThemeColors – the struct the rest of the app consumes
// ═══════════════════════════════════════════════════════════════════════════

#[derive(Clone)]
pub struct ThemeColors {
    pub bg_primary: Color,
    pub bg_secondary: Color,
    pub bg_editor: Color,
    pub bg_tab_active: Color,
    pub bg_tab_inactive: Color,
    pub bg_status_bar: Color,
    pub bg_tab_bar: Color,
    pub bg_hover: Color,
    pub bg_pressed: Color,
    pub bg_drag_handle: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_muted: Color,
    pub text_dim: Color,
    pub text_placeholder: Color,
    pub border_subtle: Color,
    pub border_very_subtle: Color,
    pub selection: Color,
    pub shadow_dark: Color,
    pub shadow_light: Color,
    pub editor_style: CodeEditorStyle,
    pub syntax_theme: SynTheme,
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Convert an iced Color to a syntect SynColor (u8 components).
const fn to_syn(c: Color) -> SynColor {
    SynColor {
        r: (c.r * 255.0) as u8,
        g: (c.g * 255.0) as u8,
        b: (c.b * 255.0) as u8,
        a: 255,
    }
}

/// Build a single syntect ThemeItem from a scope selector string + foreground Color.
fn scope_item(scope_str: &str, fg: Color, style: FontStyle) -> ThemeItem {
    ThemeItem {
        scope: ScopeSelectors::from_str(scope_str).unwrap_or_default(),
        style: StyleModifier {
            foreground: Some(to_syn(fg)),
            background: None,
            font_style: Some(style),
        },
    }
}

fn build_palette_syntax_theme() -> SynTheme {
    let none = FontStyle::empty();
    let italic = FontStyle::ITALIC;
    let bold = FontStyle::BOLD;

    let scopes = vec![
        // Comments
        scope_item(
            "comment, comment.line, comment.block, punctuation.definition.comment",
            OVERLAY_2,
            italic,
        ),
        // Keywords & control flow
        scope_item(
            "keyword, keyword.control, keyword.operator.logical, storage.type, storage.modifier",
            ACCENT_PURPLE,
            none,
        ),
        // Functions / methods
        scope_item(
            "entity.name.function, support.function, meta.function-call",
            ACCENT_BLUE,
            none,
        ),
        // Types / classes
        scope_item(
            "entity.name.type, entity.name.class, support.type, support.class",
            ACCENT_YELLOW,
            none,
        ),
        // Strings
        scope_item(
            "string, string.quoted, punctuation.definition.string",
            ACCENT_GREEN,
            none,
        ),
        // Numbers
        scope_item(
            "constant.numeric, constant.numeric.integer, constant.numeric.float",
            ACCENT_ORANGE,
            none,
        ),
        // Boolean / language constants
        scope_item(
            "constant.language, constant.language.boolean",
            ACCENT_ORANGE,
            italic,
        ),
        // Other constants
        scope_item(
            "constant.other, variable.other.constant",
            ACCENT_ORANGE,
            none,
        ),
        // Variables
        scope_item("variable, variable.other, variable.parameter", TEXT_1, none),
        // Properties / fields
        scope_item(
            "variable.other.property, variable.other.member, support.variable.property",
            ACCENT_SOFT_BLUE,
            none,
        ),
        // Operators
        scope_item(
            "keyword.operator, keyword.operator.assignment, punctuation.accessor",
            ACCENT_SKY,
            none,
        ),
        // Punctuation / brackets
        scope_item(
            "punctuation, punctuation.section, punctuation.separator, meta.brace",
            OVERLAY_3,
            none,
        ),
        // Tags (HTML / XML)
        scope_item(
            "entity.name.tag, punctuation.definition.tag",
            ACCENT_PURPLE,
            none,
        ),
        // Attributes
        scope_item("entity.other.attribute-name", ACCENT_YELLOW, italic),
        // Namespaces / modules
        scope_item(
            "entity.name.namespace, entity.name.module",
            ACCENT_WARM_1,
            none,
        ),
        // Macros
        scope_item(
            "entity.name.macro, support.function.macro",
            ACCENT_TEAL,
            bold,
        ),
        // Lifetimes / labels
        scope_item(
            "storage.modifier.lifetime, entity.name.lifetime",
            ACCENT_DARK_RED,
            italic,
        ),
        // Escape sequences
        scope_item("constant.character.escape", ACCENT_PINK, none),
        // Regex
        scope_item("string.regexp", ACCENT_ORANGE, none),
        // Decorators / annotations
        scope_item(
            "meta.decorator, meta.annotation, punctuation.decorator",
            ACCENT_ORANGE,
            italic,
        ),
        // Markdown headings
        scope_item("markup.heading, entity.name.section", ACCENT_BLUE, bold),
        // Markdown bold / italic
        scope_item("markup.bold", TEXT_1, bold),
        scope_item("markup.italic", TEXT_1, italic),
        // Links
        scope_item(
            "markup.underline.link, string.other.link",
            ACCENT_MID_BLUE,
            none,
        ),
        // Diff
        scope_item("markup.inserted", ACCENT_GREEN, none),
        scope_item("markup.deleted", ACCENT_RED, none),
        scope_item("markup.changed", ACCENT_YELLOW, none),
        // Invalid / errors
        scope_item("invalid, invalid.illegal", ACCENT_RED, none),
    ];

    SynTheme {
        name: Some("Palette".to_string()),
        author: None,
        settings: ThemeSettings {
            foreground: Some(to_syn(TEXT_1)),
            background: Some(to_syn(BG_BASE)),
            caret: Some(to_syn(ACCENT_WARM_1)),
            line_highlight: Some(to_syn(SURFACE_1)),
            selection: Some(SynColor {
                r: 137,
                g: 180,
                b: 250,
                a: 77,
            }), // ACCENT_BLUE @ 0.3
            ..ThemeSettings::default()
        },
        scopes,
    }
}

fn editor_style(
    background: Color,
    text_color: Color,
    gutter_background: Color,
    gutter_border: Color,
    line_number_color: Color,
    scrollbar_background: Color,
    scroller_color: Color,
    current_line_highlight: Color,
) -> CodeEditorStyle {
    CodeEditorStyle {
        background,
        text_color,
        gutter_background,
        gutter_border,
        line_number_color,
        scrollbar_background,
        scroller_color,
        current_line_highlight,
    }
}

impl Default for ThemeColors {
    fn default() -> Self {
        // Default is Pinel Blueberry Dark — built lazily to avoid recursion
        pinel_blueberry_dark()
    }
}

impl ThemeColors {
    /// Convert a theme_manager::ThemeColors (hex strings) into a runtime ThemeColors.
    pub fn from_lua_theme(lua: &crate::config::theme_manager::ThemeColors) -> Self {
        let p = |hex: &str| -> Color {
            let hex = hex.trim_start_matches('#');
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0) as f32 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0) as f32 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0) as f32 / 255.0;
            Color::from_rgb(r, g, b)
        };

        let surface0 = p(&lua.surface0);
        let surface1 = p(&lua.surface1);
        let surface2 = p(&lua.surface2);
        let base = p(&lua.base);
        let mantle = p(&lua.mantle);
        let crust = p(&lua.crust);
        let text_c = p(&lua.text);
        let sub1 = p(&lua.subtext1);
        let sub0 = p(&lua.subtext0);
        let ov2 = p(&lua.overlay2);
        let ov1 = p(&lua.overlay1);
        let ov0 = p(&lua.overlay0);
        let blue_c = p(&lua.blue);
        let purple_c = p(&lua.mauve);
        let green_c = p(&lua.green);
        let yellow_c = p(&lua.yellow);
        let red_c = p(&lua.red);
        let orange_c = p(&lua.peach);
        let teal_c = p(&lua.teal);
        let sky_c = p(&lua.sky);
        let pink_c = p(&lua.pink);
        let lav_c = p(&lua.lavender);

        let syn = build_syntax_theme(
            "Custom", text_c, base, text_c, surface0, ov2, purple_c, blue_c, yellow_c, green_c,
            orange_c, orange_c, text_c, lav_c, red_c, ov0,
        );

        Self {
            bg_primary: mantle,
            bg_secondary: mantle,
            bg_editor: base,
            bg_tab_active: surface0,
            bg_tab_inactive: mantle,
            bg_status_bar: mantle,
            bg_tab_bar: crust,
            bg_hover: surface1,
            bg_pressed: surface2,
            bg_drag_handle: surface0,
            text_primary: text_c,
            text_secondary: sub1,
            text_muted: sub0,
            text_dim: ov2,
            text_placeholder: ov1,
            border_subtle: surface1,
            border_very_subtle: surface0,
            selection: Color::from_rgba(blue_c.r, blue_c.g, blue_c.b, 0.3),
            shadow_dark: Color::from_rgba(crust.r, crust.g, crust.b, 0.5),
            shadow_light: Color::from_rgba(surface2.r, surface2.g, surface2.b, 0.08),
            editor_style: editor_style(
                base,
                text_c,
                surface0,
                surface1,
                ov2,
                base,
                blue_c,
                Color::from_rgba(blue_c.r, blue_c.g, blue_c.b, 0.12),
            ),
            syntax_theme: syn,
        }
    }
}

pub static THEME: Lazy<RwLock<ThemeColors>> = Lazy::new(|| RwLock::new(ThemeColors::default()));

/// Get a read guard to the current theme. Use as `theme().field`.
pub fn theme() -> std::sync::RwLockReadGuard<'static, ThemeColors> {
    THEME.read().unwrap()
}

/// Replace the current theme (for hot-reload / theme switching).
pub fn set_theme(t: ThemeColors) {
    let mut w = THEME.write().unwrap();
    *w = t;
}

pub fn parse_hex_color(input: &str) -> Result<Color, String> {
    let hex = input.trim().trim_start_matches('#');

    match hex.len() {
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| format!("invalid color: {input}"))?;
            let g = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| format!("invalid color: {input}"))?;
            let b = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| format!("invalid color: {input}"))?;
            Ok(Color::from_rgb8(r, g, b))
        }
        8 => {
            let r = u8::from_str_radix(&hex[0..2], 16)
                .map_err(|_| format!("invalid color: {input}"))?;
            let g = u8::from_str_radix(&hex[2..4], 16)
                .map_err(|_| format!("invalid color: {input}"))?;
            let b = u8::from_str_radix(&hex[4..6], 16)
                .map_err(|_| format!("invalid color: {input}"))?;
            let a = u8::from_str_radix(&hex[6..8], 16)
                .map_err(|_| format!("invalid color: {input}"))?;
            Ok(Color::from_rgba8(r, g, b, a as f32 / 255.0))
        }
        _ => Err(format!("expected #RRGGBB or #RRGGBBAA, got: {input}")),
    }
}

impl ThemeColors {
    pub fn set_named_color(&mut self, name: &str, color: Color) -> Result<(), String> {
        match name {
            "bg_primary" => self.bg_primary = color,
            "bg_secondary" => self.bg_secondary = color,
            "bg_editor" => self.bg_editor = color,
            "bg_tab_active" => self.bg_tab_active = color,
            "bg_tab_inactive" => self.bg_tab_inactive = color,
            "bg_status_bar" => self.bg_status_bar = color,
            "bg_tab_bar" => self.bg_tab_bar = color,
            "bg_hover" => self.bg_hover = color,
            "bg_pressed" => self.bg_pressed = color,
            "bg_drag_handle" => self.bg_drag_handle = color,
            "text_primary" => self.text_primary = color,
            "text_secondary" => self.text_secondary = color,
            "text_muted" => self.text_muted = color,
            "text_dim" => self.text_dim = color,
            "text_placeholder" => self.text_placeholder = color,
            "border_subtle" => self.border_subtle = color,
            "border_very_subtle" => self.border_very_subtle = color,
            "selection" => self.selection = color,
            "shadow_dark" => self.shadow_dark = color,
            "shadow_light" => self.shadow_light = color,
            "editor.background" => self.editor_style.background = color,
            "editor.text_color" => self.editor_style.text_color = color,
            "editor.gutter_background" => self.editor_style.gutter_background = color,
            "editor.gutter_border" => self.editor_style.gutter_border = color,
            "editor.line_number_color" => self.editor_style.line_number_color = color,
            "editor.scrollbar_background" => self.editor_style.scrollbar_background = color,
            "editor.scroller_color" => self.editor_style.scroller_color = color,
            "editor.current_line_highlight" => self.editor_style.current_line_highlight = color,
            _ => return Err(format!("unknown theme color: {name}")),
        }

        Ok(())
    }
}

/// Public alias to construct a Pinel Blueberry Dark theme (used by app startup).
pub fn pinel_blueberry_dark_theme() -> ThemeColors {
    pinel_blueberry_dark()
}

// ═══════════════════════════════════════════════════════════════════════════
// Built-in theme palettes
// ═══════════════════════════════════════════════════════════════════════════

/// List of all built-in theme names.
/// The first two entries are the "Pinel Blueberry" themes and serve as the default.
pub const BUILTIN_THEMES: &[&str] = &[
    "Pinel Blueberry Dark",
    "Pinel Blueberry Light",
    "Catppuccin Mocha",
    "Gruvbox Dark",
    "GitHub Dark",
    "Nord",
    "TokyoNight",
    "Ayu Dark",
];

/// Build a ThemeColors from a named built-in theme.
pub fn builtin_theme(name: &str) -> ThemeColors {
    match name {
        "Pinel Blueberry Dark" => pinel_blueberry_dark(),
        "Pinel Blueberry Light" => pinel_blueberry_light(),
        "Gruvbox Dark" => gruvbox_dark(),
        "GitHub Dark" => github_dark(),
        "Nord" => nord(),
        "TokyoNight" => tokyonight(),
        "Ayu Dark" => ayu_dark(),
        _ => pinel_blueberry_dark(), // fallback to Blueberry Dark
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Pinel Blueberry — Dark
// ═══════════════════════════════════════════════════════════════════════════
// Palette (dark)
//   bg      #1e1e2e   fg      #b4befe
//   red     #df4576   green   #00ffd2
//   blue    #00a9ff   yellow  #f9e2af
//   purple  #cba6f7   cyan    #89dceb
//   gray    #9399b2

fn pinel_blueberry_dark() -> ThemeColors {
    let bg = Color::from_rgb(0.118, 0.118, 0.180); // #1e1e2e
    let bg_raised = Color::from_rgb(0.149, 0.149, 0.212); // #262637 — sidebar / panels
    let bg_surface = Color::from_rgb(0.192, 0.196, 0.267); // #313244 — elevated surfaces
    let fg = Color::from_rgb(0.706, 0.745, 0.996); // #b4befe
    let red = Color::from_rgb(0.875, 0.271, 0.463); // #df4576
    let green = Color::from_rgb(0.000, 1.000, 0.824); // #00ffd2
    let blue = Color::from_rgb(0.000, 0.663, 1.000); // #00a9ff
    let yellow = Color::from_rgb(0.976, 0.886, 0.686); // #f9e2af
    let purple = Color::from_rgb(0.796, 0.651, 0.969); // #cba6f7
    let cyan = Color::from_rgb(0.537, 0.863, 0.922); // #89dceb
    let gray = Color::from_rgb(0.576, 0.600, 0.698); // #9399b2

    // Text hierarchy built from the fg / gray
    let text_primary = fg; // #b4befe — vivid, readable
    let text_secondary = Color::from_rgb(0.651, 0.678, 0.784); // #a6adc8
    let text_muted = gray; // #9399b2
    let text_dim = Color::from_rgb(0.498, 0.518, 0.612); // #7f849c
    let text_placeholder = Color::from_rgb(0.424, 0.439, 0.525); // #6c7086

    let border_subtle = Color::from_rgb(0.271, 0.278, 0.353); // #45475a
    let border_very_subtle = Color::from_rgb(0.192, 0.196, 0.267); // #313244

    let syn = build_syntax_theme(
        "Pinel Blueberry Dark",
        fg,         // foreground
        bg,         // background
        fg,         // caret
        bg_surface, // line highlight
        gray,       // comments
        purple,     // keywords
        blue,       // functions
        cyan,       // types
        green,      // strings
        yellow,     // numbers
        yellow,     // constants
        fg,         // variables
        blue,       // properties
        cyan,       // operators
        text_dim,   // punctuation
    );

    ThemeColors {
        bg_primary: bg_raised,
        bg_secondary: bg_raised,
        bg_editor: bg,
        bg_tab_active: bg_surface,
        bg_tab_inactive: bg_raised,
        bg_status_bar: bg_raised,
        bg_tab_bar: Color::from_rgb(0.094, 0.094, 0.145), // #181825 crust-like
        bg_hover: bg_surface,
        bg_pressed: Color::from_rgb(0.345, 0.357, 0.439), // #585b70
        bg_drag_handle: bg_surface,
        text_primary,
        text_secondary,
        text_muted,
        text_dim,
        text_placeholder,
        border_subtle,
        border_very_subtle,
        selection: Color::from_rgba(purple.r, purple.g, purple.b, 0.25),
        shadow_dark: Color::from_rgba(0.0, 0.0, 0.0, 0.55),
        shadow_light: Color::from_rgba(purple.r, purple.g, purple.b, 0.06),
        editor_style: editor_style(
            Color::from_rgb(0.058, 0.058, 0.090),
            Color::from_rgb(0.772, 0.800, 0.882),
            Color::from_rgb(0.082, 0.082, 0.122),
            Color::from_rgb(0.149, 0.149, 0.212),
            Color::from_rgb(0.424, 0.439, 0.525),
            Color::from_rgb(0.058, 0.058, 0.090),
            blue,
            Color::from_rgba(0.647, 0.690, 0.906, 0.12),
        ),
        syntax_theme: syn,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// Pinel Blueberry — Light
// ═══════════════════════════════════════════════════════════════════════════
// Palette (light)
//   bg      #eff1f5   fg      #4c4f69
//   red     #d20f39   green   #40a02b
//   blue    #1e66f5   yellow  #df8e1d
//   purple  #8839ef   cyan    #179299
//   gray    #7c7f93

fn pinel_blueberry_light() -> ThemeColors {
    let bg = Color::from_rgb(0.937, 0.945, 0.961); // #eff1f5
    let bg_raised = Color::from_rgb(0.894, 0.906, 0.929); // #e4e7ee
    let bg_surface = Color::from_rgb(0.847, 0.863, 0.894); // #d8dbE4
    let fg = Color::from_rgb(0.298, 0.310, 0.412); // #4c4f69
    let red = Color::from_rgb(0.824, 0.059, 0.224); // #d20f39
    let green = Color::from_rgb(0.251, 0.627, 0.169); // #40a02b
    let blue = Color::from_rgb(0.118, 0.400, 0.961); // #1e66f5
    let yellow = Color::from_rgb(0.875, 0.557, 0.114); // #df8e1d
    let purple = Color::from_rgb(0.533, 0.224, 0.937); // #8839ef
    let cyan = Color::from_rgb(0.090, 0.573, 0.600); // #179299
    let gray = Color::from_rgb(0.486, 0.498, 0.576); // #7c7f93

    let text_primary = fg;
    let text_secondary = Color::from_rgb(0.376, 0.396, 0.510); // #606481 approx
    let text_muted = gray;
    let text_dim = Color::from_rgb(0.620, 0.635, 0.715); // #9e9fb5 approx
    let text_placeholder = Color::from_rgb(0.706, 0.722, 0.796); // #b4b7c9 approx

    let border_subtle = Color::from_rgb(0.788, 0.808, 0.855); // #c9ceda
    let border_very_subtle = Color::from_rgb(0.847, 0.863, 0.894); // same as bg_surface

    let syn = build_syntax_theme(
        "Pinel Blueberry Light",
        fg,
        bg,
        fg,
        bg_surface,
        gray,
        purple,
        blue,
        cyan,
        green,
        yellow,
        yellow,
        fg,
        blue,
        cyan,
        text_dim,
    );

    ThemeColors {
        bg_primary: bg_raised,
        bg_secondary: bg_raised,
        bg_editor: bg,
        bg_tab_active: bg_surface,
        bg_tab_inactive: bg_raised,
        bg_status_bar: bg_raised,
        bg_tab_bar: Color::from_rgb(0.867, 0.882, 0.914), // slightly darker crust
        bg_hover: bg_surface,
        bg_pressed: Color::from_rgb(0.788, 0.808, 0.855),
        bg_drag_handle: bg_surface,
        text_primary,
        text_secondary,
        text_muted,
        text_dim,
        text_placeholder,
        border_subtle,
        border_very_subtle,
        selection: Color::from_rgba(purple.r, purple.g, purple.b, 0.18),
        shadow_dark: Color::from_rgba(0.298, 0.310, 0.412, 0.18),
        shadow_light: Color::from_rgba(purple.r, purple.g, purple.b, 0.05),
        editor_style: editor_style(
            Color::from_rgb(0.983, 0.985, 0.991),
            Color::from_rgb(0.565, 0.608, 0.686),
            Color::from_rgb(0.945, 0.949, 0.972),
            Color::from_rgb(0.894, 0.906, 0.929),
            Color::from_rgb(0.620, 0.635, 0.715),
            Color::from_rgb(0.983, 0.985, 0.991),
            blue,
            Color::from_rgba(0.647, 0.690, 0.906, 0.14),
        ),
        syntax_theme: syn,
    }
}

fn gruvbox_dark() -> ThemeColors {
    // Gruvbox dark palette
    let bg0 = Color::from_rgb(0.157, 0.157, 0.157); // #282828
    let bg0_h = Color::from_rgb(0.110, 0.110, 0.102); // #1d2021
    let bg1 = Color::from_rgb(0.235, 0.220, 0.208); // #3c3836
    let bg2 = Color::from_rgb(0.314, 0.298, 0.275); // #504945
    let bg3 = Color::from_rgb(0.396, 0.380, 0.357); // #665c54
    let fg0 = Color::from_rgb(0.984, 0.945, 0.831); // #fbf1c7
    let fg1 = Color::from_rgb(0.922, 0.859, 0.698); // #ebdbb2
    let fg2 = Color::from_rgb(0.835, 0.769, 0.627); // #d5c4a1
    let fg3 = Color::from_rgb(0.741, 0.682, 0.553); // #bdae93
    let fg4 = Color::from_rgb(0.659, 0.600, 0.482); // #a89984
    let red = Color::from_rgb(0.984, 0.286, 0.204); // #fb4934
    let green = Color::from_rgb(0.722, 0.733, 0.149); // #b8bb26
    let yellow = Color::from_rgb(0.980, 0.741, 0.184); // #fabd2f
    let blue = Color::from_rgb(0.514, 0.647, 0.596); // #83a598
    let purple = Color::from_rgb(0.827, 0.525, 0.608); // #d3869b
    let aqua = Color::from_rgb(0.557, 0.753, 0.486); // #8ec07c
    let orange = Color::from_rgb(0.996, 0.576, 0.027); // #fe8019

    let syn = build_syntax_theme(
        "Gruvbox", fg1, bg0, fg0, bg1, fg4,    // comments
        red,    // keywords
        green,  // functions - using aqua for distinction
        yellow, // types
        green,  // strings
        purple, // numbers
        orange, // constants
        fg1,    // variables
        blue,   // properties
        aqua,   // operators
        fg3,    // punctuation
    );

    ThemeColors {
        bg_primary: bg1,
        bg_secondary: bg0_h,
        bg_editor: bg0,
        bg_tab_active: bg1,
        bg_tab_inactive: bg0_h,
        bg_status_bar: bg0_h,
        bg_tab_bar: bg0_h,
        bg_hover: bg2,
        bg_pressed: bg3,
        bg_drag_handle: bg1,
        text_primary: fg1,
        text_secondary: fg2,
        text_muted: fg3,
        text_dim: fg4,
        text_placeholder: fg4,
        border_subtle: bg2,
        border_very_subtle: bg1,
        selection: Color::from_rgba(0.514, 0.647, 0.596, 0.3),
        shadow_dark: Color::from_rgba(0.110, 0.110, 0.102, 0.5),
        shadow_light: Color::from_rgba(0.396, 0.380, 0.357, 0.08),
        editor_style: editor_style(
            bg0,
            fg1,
            bg1,
            bg2,
            fg4,
            bg0,
            blue,
            Color::from_rgba(0.514, 0.647, 0.596, 0.14),
        ),
        syntax_theme: syn,
    }
}

fn github_dark() -> ThemeColors {
    let bg = Color::from_rgb(0.055, 0.067, 0.090); // #0d1117
    let bg1 = Color::from_rgb(0.094, 0.106, 0.133); // #161b22
    let bg2 = Color::from_rgb(0.129, 0.149, 0.180); // #21262d
    let bg3 = Color::from_rgb(0.188, 0.208, 0.235); // #30363d
    let fg = Color::from_rgb(0.890, 0.914, 0.941); // #e3ecf1 approx c9d1d9
    let fg1 = Color::from_rgb(0.788, 0.820, 0.855); // #c9d1d9
    let fg2 = Color::from_rgb(0.557, 0.592, 0.643); // #8b949e
    let fg3 = Color::from_rgb(0.329, 0.369, 0.424); // #484f58 approx
    let red = Color::from_rgb(1.0, 0.482, 0.388); // #ff7b72
    let green = Color::from_rgb(0.482, 0.780, 0.459); // #7ee787 approx
    let blue = Color::from_rgb(0.310, 0.565, 0.961); // #58a6ff
    let purple = Color::from_rgb(0.827, 0.506, 0.976); // #d2a8ff
    let orange = Color::from_rgb(0.843, 0.537, 0.204); // #d18616 approx
    let cyan = Color::from_rgb(0.463, 0.808, 0.918); // #79c0ff approx
    let yellow = Color::from_rgb(0.882, 0.776, 0.369); // #e3b341 approx

    let syn = build_syntax_theme(
        "GitHub Dark",
        fg1,
        bg,
        fg,
        bg2,
        fg2,
        red,
        purple,
        cyan,
        green,
        orange,
        orange,
        fg1,
        blue,
        red,
        fg3,
    );

    ThemeColors {
        bg_primary: bg2,
        bg_secondary: bg1,
        bg_editor: bg,
        bg_tab_active: bg2,
        bg_tab_inactive: bg1,
        bg_status_bar: bg1,
        bg_tab_bar: bg1,
        bg_hover: bg2,
        bg_pressed: bg3,
        bg_drag_handle: bg2,
        text_primary: fg1,
        text_secondary: fg,
        text_muted: fg2,
        text_dim: fg2,
        text_placeholder: fg3,
        border_subtle: bg3,
        border_very_subtle: bg2,
        selection: Color::from_rgba(0.310, 0.565, 0.961, 0.3),
        shadow_dark: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
        shadow_light: Color::from_rgba(0.188, 0.208, 0.235, 0.08),
        editor_style: editor_style(
            bg,
            fg1,
            bg1,
            bg2,
            fg2,
            bg,
            blue,
            Color::from_rgba(0.310, 0.565, 0.961, 0.12),
        ),
        syntax_theme: syn,
    }
}

fn nord() -> ThemeColors {
    let polar0 = Color::from_rgb(0.180, 0.204, 0.251); // #2e3440
    let polar1 = Color::from_rgb(0.231, 0.259, 0.322); // #3b4252
    let polar2 = Color::from_rgb(0.263, 0.298, 0.369); // #434c5e
    let polar3 = Color::from_rgb(0.298, 0.337, 0.416); // #4c566a
    let snow0 = Color::from_rgb(0.847, 0.871, 0.914); // #d8dee9
    let snow1 = Color::from_rgb(0.898, 0.914, 0.941); // #e5e9f0
    let snow2 = Color::from_rgb(0.929, 0.945, 0.969); // #eceff4
    let frost0 = Color::from_rgb(0.557, 0.737, 0.733); // #8fbcbb
    let frost1 = Color::from_rgb(0.533, 0.753, 0.816); // #88c0d0
    let frost2 = Color::from_rgb(0.506, 0.631, 0.757); // #81a1c1
    let frost3 = Color::from_rgb(0.369, 0.506, 0.675); // #5e81ac
    let aurora_r = Color::from_rgb(0.749, 0.380, 0.416); // #bf616a
    let aurora_o = Color::from_rgb(0.816, 0.529, 0.439); // #d08770
    let aurora_y = Color::from_rgb(0.922, 0.796, 0.545); // #ebcb8b
    let aurora_g = Color::from_rgb(0.639, 0.745, 0.549); // #a3be8c
    let aurora_p = Color::from_rgb(0.706, 0.557, 0.678); // #b48ead

    let syn = build_syntax_theme(
        "Nord", snow0, polar0, snow1, polar1, polar3, frost2, frost1, frost0, aurora_g, aurora_p,
        aurora_o, snow0, frost3, frost2, polar3,
    );

    ThemeColors {
        bg_primary: polar1,
        bg_secondary: polar0,
        bg_editor: polar0,
        bg_tab_active: polar1,
        bg_tab_inactive: polar0,
        bg_status_bar: polar0,
        bg_tab_bar: polar0,
        bg_hover: polar2,
        bg_pressed: polar3,
        bg_drag_handle: polar1,
        text_primary: snow0,
        text_secondary: snow1,
        text_muted: snow2,
        text_dim: polar3,
        text_placeholder: polar3,
        border_subtle: polar2,
        border_very_subtle: polar1,
        selection: Color::from_rgba(0.533, 0.753, 0.816, 0.3),
        shadow_dark: Color::from_rgba(0.180, 0.204, 0.251, 0.5),
        shadow_light: Color::from_rgba(0.298, 0.337, 0.416, 0.08),
        editor_style: editor_style(
            polar0,
            snow0,
            polar1,
            polar2,
            polar3,
            polar0,
            frost2,
            Color::from_rgba(0.533, 0.753, 0.816, 0.12),
        ),
        syntax_theme: syn,
    }
}

fn tokyonight() -> ThemeColors {
    let bg = Color::from_rgb(0.102, 0.110, 0.176); // #1a1b2e approx
    let bg_dark = Color::from_rgb(0.063, 0.071, 0.129); // #16161e
    let bg1 = Color::from_rgb(0.145, 0.153, 0.224); // #24283b
    let bg2 = Color::from_rgb(0.200, 0.212, 0.302); // #292e42 approx
    let fg = Color::from_rgb(0.757, 0.827, 0.922); // #c0caf5
    let fg_dark = Color::from_rgb(0.627, 0.686, 0.761); // #a9b1d6
    let comment = Color::from_rgb(0.337, 0.376, 0.502); // #565f89
    let red = Color::from_rgb(0.969, 0.471, 0.518); // #f7768e
    let green = Color::from_rgb(0.451, 0.839, 0.506); // #73daca approx 9ece6a
    let green2 = Color::from_rgb(0.620, 0.808, 0.416); // #9ece6a
    let blue = Color::from_rgb(0.486, 0.631, 0.984); // #7aa2f7
    let blue2 = Color::from_rgb(0.478, 0.718, 0.957); // #7dcfff
    let purple = Color::from_rgb(0.733, 0.518, 0.969); // #bb9af7
    #[allow(clippy::approx_constant)] // 0.318 is an intentional color value, not FRAC_1_PI
    let orange = Color::from_rgb(1.0, 0.608, 0.318); // #ff9e64

    let yellow = Color::from_rgb(0.878, 0.769, 0.459); // #e0af68
    let cyan = Color::from_rgb(0.486, 0.863, 0.871); // #7dcfff approx

    let syn = build_syntax_theme(
        "TokyoNight",
        fg,
        bg,
        fg,
        bg1,
        comment,
        purple,
        blue,
        blue2,
        green2,
        orange,
        orange,
        fg,
        cyan,
        red,
        comment,
    );

    ThemeColors {
        bg_primary: bg1,
        bg_secondary: bg_dark,
        bg_editor: bg,
        bg_tab_active: bg1,
        bg_tab_inactive: bg_dark,
        bg_status_bar: bg_dark,
        bg_tab_bar: bg_dark,
        bg_hover: bg2,
        bg_pressed: bg2,
        bg_drag_handle: bg1,
        text_primary: fg,
        text_secondary: fg_dark,
        text_muted: fg_dark,
        text_dim: comment,
        text_placeholder: comment,
        border_subtle: bg2,
        border_very_subtle: bg1,
        selection: Color::from_rgba(0.486, 0.631, 0.984, 0.3),
        shadow_dark: Color::from_rgba(0.063, 0.071, 0.129, 0.5),
        shadow_light: Color::from_rgba(0.200, 0.212, 0.302, 0.08),
        editor_style: editor_style(
            bg,
            fg,
            bg1,
            bg2,
            fg_dark,
            bg,
            blue,
            Color::from_rgba(0.486, 0.631, 0.984, 0.12),
        ),
        syntax_theme: syn,
    }
}

fn ayu_dark() -> ThemeColors {
    let bg = Color::from_rgb(0.051, 0.071, 0.098); // #0d1117 approx #0a0e14
    let bg1 = Color::from_rgb(0.078, 0.098, 0.133); // #131721 approx
    let bg2 = Color::from_rgb(0.110, 0.133, 0.169); // #1c2027 approx
    let bg3 = Color::from_rgb(0.180, 0.200, 0.235); // #2d3640 approx
    let fg = Color::from_rgb(0.710, 0.773, 0.827); // #b3b1ad approx #bfbdb6
    let fg2 = Color::from_rgb(0.561, 0.612, 0.659); // #acb6bf approx
    let comment = Color::from_rgb(0.345, 0.396, 0.459); // #626a73 approx
    let red = Color::from_rgb(0.965, 0.333, 0.357); // #f07178
    let green = Color::from_rgb(0.667, 0.808, 0.357); // #aad94c
    let orange = Color::from_rgb(1.0, 0.702, 0.333); // #ffb454
    let yellow = Color::from_rgb(0.898, 0.745, 0.412); // #e6b450 approx
    let blue = Color::from_rgb(0.224, 0.651, 0.925); // #39bae6
    let purple = Color::from_rgb(0.827, 0.639, 0.925); // #d2a6ff
    let cyan = Color::from_rgb(0.588, 0.875, 0.816); // #95e6cb

    let syn = build_syntax_theme(
        "Ayu Dark", fg, bg, fg, bg2, comment, orange, blue, cyan, green, purple, orange, fg,
        yellow, red, comment,
    );

    ThemeColors {
        bg_primary: bg2,
        bg_secondary: bg1,
        bg_editor: bg,
        bg_tab_active: bg2,
        bg_tab_inactive: bg1,
        bg_status_bar: bg1,
        bg_tab_bar: bg1,
        bg_hover: bg2,
        bg_pressed: bg3,
        bg_drag_handle: bg2,
        text_primary: fg,
        text_secondary: fg,
        text_muted: fg2,
        text_dim: comment,
        text_placeholder: comment,
        border_subtle: bg3,
        border_very_subtle: bg2,
        selection: Color::from_rgba(0.224, 0.651, 0.925, 0.3),
        shadow_dark: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
        shadow_light: Color::from_rgba(0.180, 0.200, 0.235, 0.08),
        editor_style: editor_style(
            bg,
            fg,
            bg1,
            bg2,
            fg2,
            bg,
            blue,
            Color::from_rgba(0.224, 0.651, 0.925, 0.12),
        ),
        syntax_theme: syn,
    }
}

/// Helper to build a syntax theme from key colors.
fn build_syntax_theme(
    name: &str,
    fg: Color,
    bg: Color,
    caret: Color,
    line_hl: Color,
    comment_c: Color,
    keyword_c: Color,
    function_c: Color,
    type_c: Color,
    string_c: Color,
    number_c: Color,
    constant_c: Color,
    variable_c: Color,
    property_c: Color,
    operator_c: Color,
    punctuation_c: Color,
) -> SynTheme {
    let none = FontStyle::empty();
    let italic = FontStyle::ITALIC;
    let bold = FontStyle::BOLD;

    let scopes = vec![
        scope_item(
            "comment, comment.line, comment.block, punctuation.definition.comment",
            comment_c,
            italic,
        ),
        scope_item(
            "keyword, keyword.control, keyword.operator.logical, storage.type, storage.modifier",
            keyword_c,
            none,
        ),
        scope_item(
            "entity.name.function, support.function, meta.function-call",
            function_c,
            none,
        ),
        scope_item(
            "entity.name.type, entity.name.class, support.type, support.class",
            type_c,
            none,
        ),
        scope_item(
            "string, string.quoted, punctuation.definition.string",
            string_c,
            none,
        ),
        scope_item(
            "constant.numeric, constant.numeric.integer, constant.numeric.float",
            number_c,
            none,
        ),
        scope_item(
            "constant.language, constant.language.boolean",
            constant_c,
            italic,
        ),
        scope_item("constant.other, variable.other.constant", constant_c, none),
        scope_item(
            "variable, variable.other, variable.parameter",
            variable_c,
            none,
        ),
        scope_item(
            "variable.other.property, variable.other.member, support.variable.property",
            property_c,
            none,
        ),
        scope_item(
            "keyword.operator, keyword.operator.assignment, punctuation.accessor",
            operator_c,
            none,
        ),
        scope_item(
            "punctuation, punctuation.section, punctuation.separator, meta.brace",
            punctuation_c,
            none,
        ),
        scope_item(
            "entity.name.tag, punctuation.definition.tag",
            keyword_c,
            none,
        ),
        scope_item("entity.other.attribute-name", type_c, italic),
        scope_item(
            "entity.name.namespace, entity.name.module",
            property_c,
            none,
        ),
        scope_item(
            "entity.name.macro, support.function.macro",
            function_c,
            bold,
        ),
        scope_item(
            "storage.modifier.lifetime, entity.name.lifetime",
            operator_c,
            italic,
        ),
        scope_item("constant.character.escape", constant_c, none),
        scope_item("string.regexp", constant_c, none),
        scope_item(
            "meta.decorator, meta.annotation, punctuation.decorator",
            constant_c,
            italic,
        ),
        scope_item("markup.heading, entity.name.section", function_c, bold),
        scope_item("markup.bold", fg, bold),
        scope_item("markup.italic", fg, italic),
        scope_item("markup.underline.link, string.other.link", property_c, none),
        scope_item("markup.inserted", string_c, none),
        scope_item("markup.deleted", operator_c, none),
        scope_item("markup.changed", type_c, none),
        scope_item("invalid, invalid.illegal", operator_c, none),
    ];

    SynTheme {
        name: Some(name.to_string()),
        author: None,
        settings: ThemeSettings {
            foreground: Some(to_syn(fg)),
            background: Some(to_syn(bg)),
            caret: Some(to_syn(caret)),
            line_highlight: Some(to_syn(line_hl)),
            selection: Some(SynColor {
                r: (fg.r * 255.0) as u8,
                g: (fg.g * 255.0) as u8,
                b: (fg.b * 255.0) as u8,
                a: 77,
            }),
            ..ThemeSettings::default()
        },
        scopes,
    }
}
