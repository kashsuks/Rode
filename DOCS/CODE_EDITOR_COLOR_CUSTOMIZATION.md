# Code Editor Color Customization

This project uses two separate theme layers for editing:

- App chrome colors: panels, tabs, status bar, sidebars in `/Users/ksukshavasi/Pinel/src/theme.rs`
- Actual `iced-code-editor` canvas colors: editor background, gutter, line numbers, scrollbar, current-line highlight in `/Users/ksukshavasi/Pinel/src/theme.rs`

## What changed

The real editor widget no longer uses `iced-code-editor`'s default Tokyo Night palette.

Pinel now stores an explicit `editor_style` inside `ThemeColors` and applies it to every editor through:

- `/Users/ksukshavasi/Pinel/src/theme.rs`
- `/Users/ksukshavasi/Pinel/src/app.rs`
- `/Users/ksukshavasi/Pinel/src/app/update.rs`
- `/Users/ksukshavasi/Pinel/src/app/commands.rs`

That means:

- `Pinel Blueberry Dark` uses a dedicated dark editor canvas palette
- `Pinel Blueberry Light` uses a dedicated light editor canvas palette
- switching themes updates already-open editors
- new editors inherit the active editor canvas palette automatically

## The important type

The actual editor canvas is controlled by `iced_code_editor::theme::Style`.

Current Pinel alias:

```rust
use iced_code_editor::theme::Style as CodeEditorStyle;
```

Its fields are:

```rust
pub struct Style {
    pub background: Color,
    pub text_color: Color,
    pub gutter_background: Color,
    pub gutter_border: Color,
    pub line_number_color: Color,
    pub scrollbar_background: Color,
    pub scroller_color: Color,
    pub current_line_highlight: Color,
}
```

## Where to edit colors

### 1. Change Blueberry Dark editor colors

Edit the `editor_style: editor_style(...)` block in:

- `/Users/ksukshavasi/Pinel/src/theme.rs`

Inside `fn pinel_blueberry_dark() -> ThemeColors`.

Current shape:

```rust
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
```

### 2. Change Blueberry Light editor colors

Edit the `editor_style: editor_style(...)` block in:

- `/Users/ksukshavasi/Pinel/src/theme.rs`

Inside `fn pinel_blueberry_light() -> ThemeColors`.

Current shape:

```rust
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
```

### 3. Change the default mapping for custom themes

If a user loads `theme.lua`, Pinel builds the editor canvas palette in:

- `/Users/ksukshavasi/Pinel/src/theme.rs`

Inside:

```rust
impl ThemeColors {
    pub fn from_lua_theme(...) -> Self
```

That is the place to change how `base`, `surface0`, `surface1`, `overlay2`, and `blue` map into the actual editor widget.

## What each field controls

- `background`: main code area background
- `text_color`: fallback text color used by the widget
- `gutter_background`: line-number column background
- `gutter_border`: divider between gutter and code area
- `line_number_color`: line-number text
- `scrollbar_background`: scrollbar track background
- `scroller_color`: scrollbar thumb color
- `current_line_highlight`: highlight behind the active line

## Recommended workflow

1. Edit the palette block in `/Users/ksukshavasi/Pinel/src/theme.rs`
2. Run `cargo check`
3. Launch the app and compare against the intended light/dark reference
4. Tune opacity values first for `current_line_highlight`
5. Tune `gutter_background` and `gutter_border` second, because those control most of the editor's visual separation

## Applying a completely custom editor palette

If you want a fully named palette instead of inline values, create a helper in `/Users/ksukshavasi/Pinel/src/theme.rs`.

Example:

```rust
fn my_editor_style() -> CodeEditorStyle {
    editor_style(
        Color::from_rgb(0.10, 0.11, 0.13),
        Color::from_rgb(0.86, 0.88, 0.91),
        Color::from_rgb(0.08, 0.09, 0.11),
        Color::from_rgb(0.16, 0.18, 0.22),
        Color::from_rgb(0.48, 0.51, 0.57),
        Color::from_rgb(0.10, 0.11, 0.13),
        Color::from_rgb(0.35, 0.63, 0.96),
        Color::from_rgba(0.35, 0.63, 0.96, 0.12),
    )
}
```

Then assign it in the theme:

```rust
ThemeColors {
    // ...
    editor_style: my_editor_style(),
    syntax_theme: syn,
}
```

## How the style gets applied

Pinel applies the active `editor_style` through:

- `/Users/ksukshavasi/Pinel/src/app.rs`

```rust
pub(super) fn configured_code_editor(&self, content: &str, syntax: &str) -> CodeEditor {
    let mut editor = iced_code_editor::CodeEditor::new(content, syntax);
    editor.set_theme(theme().editor_style);
    // ...
    editor
}
```

Existing tabs are refreshed on theme change through:

- `/Users/ksukshavasi/Pinel/src/app.rs`

```rust
pub(super) fn apply_editor_theme_to_tabs(&mut self) {
    let editor_style = theme().editor_style;

    for tab in &mut self.tabs {
        if let TabKind::Editor { code_editor, .. } = &mut tab.kind {
            code_editor.set_theme(editor_style);
        }
    }
}
```

## If you want to change more than colors

Color changes are handled in Pinel.

If you want to change:

- completion row spacing
- hover padding
- overlay border radius
- overlay layout behavior
- internal drawing details of the editor canvas

then you likely need to patch or fork `iced-code-editor`, because those are owned by the crate, not by Pinel's local theme model.
