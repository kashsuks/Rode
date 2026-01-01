use crate::setup::app::CatEditorApp;
use eframe::egui;

pub fn handle_normal_mode_input(app: &mut CatEditorApp, input: &egui::InputState) {
    if let Some(motion_char) = app.pending_motion {
        handle_pending_motion(app, input, motion_char);
        return;
    }

    if input.key_pressed(egui::Key::H) {
        move_left(app);
        app.saved_column = None;
    } else if input.key_pressed(egui::Key::J) {
        move_down(app);
    } else if input.key_pressed(egui::Key::L) {
        move_right(app);
        app.saved_column = None;
    } else if input.key_pressed(egui::Key::K) {
        move_up(app);
    } else if input.key_pressed(egui::Key::W) {
        move_word_forward(app, false);
        app.saved_column = None;
    } else if input.key_pressed(egui::Key::B) {
        move_word_backward(app, false);
        app.saved_column = None;
    } else if input.key_pressed(egui::Key::E) {
        move_word_end(app, false);
        app.saved_column = None;
    } else if input.key_pressed(egui::Key::Num0) {
        move_to_line_start(app);
        app.saved_column = None;
    } else if input.key_pressed(egui::Key::Minus) && input.modifiers.shift {
        move_to_first_non_blank(app);
        app.saved_column = None;
    } else if input.key_pressed(egui::Key::Num4) && input.modifiers.shift {
        move_to_line_end(app);
        app.saved_column = None;
    } else if input.key_pressed(egui::Key::G) {
        if input.modifiers.shift {
            move_to_last_line(app);
            app.saved_column = None;
        }
    } else if input.key_pressed(egui::Key::CloseBracket) && input.modifiers.shift {
        move_to_next_paragraph(app);
        app.saved_column = None;
    } else if input.key_pressed(egui::Key::OpenBracket) && input.modifiers.shift {
        move_to_prev_paragraph(app);
        app.saved_column = None;
    } else if input.modifiers.ctrl && input.key_pressed(egui::Key::F) {
        move_page_down(app);
    } else if input.modifiers.ctrl && input.key_pressed(egui::Key::B) {
        move_page_up(app);
    } else if input.modifiers.ctrl && input.key_pressed(egui::Key::D) {
        move_half_page_down(app);
    } else if input.modifiers.ctrl && input.key_pressed(egui::Key::U) {
        move_half_page_up(app);
    }

    for event in &input.events {
        if let egui::Event::Text(text) = event {
            handle_text_command(app, text);
        }
    }
}

fn handle_text_command(app: &mut CatEditorApp, text: &str) {
    match text {
        "g" => {
            if let Some('g') = app.pending_motion {
                move_to_first_line(app);
                app.saved_column = None;
                app.pending_motion = None;
            } else {
                app.pending_motion = Some('g')
            }
        }
        "f" => app.pending_motion = Some('f'),
        "t" => app.pending_motion = Some('t'),
        "F" => app.pending_motion = Some('F'),
        "T" => app.pending_motion = Some('T'),
        "Z" => app.pending_motion = Some('Z'),
        _ => app.pending_motion = None,
    }
}

fn handle_pending_motion(app: &mut CatEditorApp, input: &egui::InputState, motion: char) {
    for event in &input.events {
        if let egui::Event::Text(text) = event {
            match motion {
                'g' => {
                    match text.as_str() {
                        "g" => {
                            move_to_first_line(app);
                            app.saved_column = None;
                        }
                        "j" => move_down(app),
                        "k" => move_up(app),
                        "e" => {
                            move_word_end_backward(app, false);
                            app.saved_column = None;
                        }
                        "E" => {
                            move_word_end_backward(app, true);
                            app.saved_column = None;
                        }
                        "d" => {},
                        "D" => {},
                        "_" => {
                            move_to_last_non_blank(app);
                            app.saved_column = None;
                        }
                        _ => {}
                    }
                }
                'f' => {
                    find_char_forward(app, text.chars().next().unwrap_or(' '));
                    app.saved_column = None;
                }
                't' => {
                    find_char_to_forward(app, text.chars().next().unwrap_or(' '));
                    app.saved_column = None;
                }
                'F' => {
                    find_char_backward(app, text.chars().next().unwrap_or(' '));
                    app.saved_column = None;
                }
                'T' => {
                    find_char_to_backward(app, text.chars().next().unwrap_or(' '));
                    app.saved_column = None;
                }
                'z' => {
                    match text.as_str() {
                        "z" => {},
                        "t" => {},
                        "b" => {},
                        _ => {}
                    }
                }
                _ => {}
            }
            app.pending_motion = None;
        }
    }
}

fn get_current_line_and_column(app: &CatEditorApp) -> (usize, usize) {
    let lines: Vec<&str> = app.text.lines().collect();
    let mut current_line = 0;
    let mut char_count = 0;

    for (i, line) in lines.iter().enumerate() {
        let line_len = line.chars().count();
        if char_count + line_len >= app.cursor_pos {
            current_line = i;
            break;
        }
        char_count += line.len() + 1;
    }

    let col = app.cursor_pos.saturating_sub(char_count);
    (current_line, col)
}

fn move_left(app: &mut CatEditorApp) {
    if app.cursor_pos > 0 {
        app.cursor_pos -= 1;
    }
}

fn move_right(app: &mut CatEditorApp) {
    let max = app.text.chars().count();
    if app.cursor_pos < max {
        app.cursor_pos += 1;
    }
}

fn move_up(app: &mut CatEditorApp) {
    let lines: Vec<&str> = app.text.lines().collect();
    let (current_line, current_col) = get_current_line_and_column(app);

    if app.saved_column.is_none() {
        app.saved_column = Some(current_col);
    }

    if current_line > 0 {
        let target_col = app.saved_column.unwrap_or(current_col);
        let prev_line_len = lines[current_line - 1].chars().count();
        let new_col = target_col.min(prev_line_len);

        let char_count: usize = lines.iter().take(current_line - 1).map(|l| l.len() + 1).sum();
        app.cursor_pos = char_count + new_col;
    }
}

fn move_down(app: &mut CatEditorApp) {
    let lines: Vec<&str> = app.text.lines().collect();
    if lines.is_empty() {
        return;
    }

    let (current_line, current_col) = get_current_line_and_column(app);

    if app.saved_column.is_none() {
        app.saved_column = Some(current_col);
    }

    if current_line + 1 < lines.len() {
        let target_col = app.saved_column.unwrap_or(current_col);
        let next_len = lines[current_line + 1].chars().count();
        let new_col = target_col.min(next_len);

        let next_line_start: usize = lines
            .iter()
            .take(current_line + 1)
            .map(|l| l.chars().count() + 1)
            .sum();
        
        app.cursor_pos = next_line_start + new_col;
    }
}

fn move_word_forward(app: &mut CatEditorApp, _with_punct: bool) {
    let chars: Vec<char> = app.text.chars().collect();
    let mut pos = app.cursor_pos;

    while pos < chars.len() && !chars[pos].is_whitespace() {
        pos += 1
    }

    while pos < chars.len() && chars[pos].is_whitespace() {
        pos += 1
    }

    app.cursor_pos = pos;
}

fn move_word_backward(app: &mut CatEditorApp, _with_punct: bool) {
    if app.cursor_pos == 0 {
        return;
    }

    let chars: Vec<char> = app.text.chars().collect();
    let mut pos = app.cursor_pos - 1;

    while pos > 0 && chars[pos].is_whitespace() {
        pos -= 1;
    }

    while pos > 0 && !chars[pos].is_whitespace() {
        pos -= 1;
    }

    if pos > 0 {
        pos += 1;
    }

    app.cursor_pos = pos;
}

fn move_word_end(app: &mut CatEditorApp, _with_punct: bool) {
    let chars: Vec<char> = app.text.chars().collect();
    let mut pos = app.cursor_pos + 1;

    while pos < chars.len() && chars[pos].is_whitespace() {
        pos += 1;
    }

    while pos < chars.len() && !chars[pos].is_whitespace() {
        pos += 1;
    }

    if pos > 0 {
        pos -= 1;
    }

    app.cursor_pos = pos;
}

fn move_word_end_backward(app: &mut CatEditorApp, _with_punct: bool) {
    if app.cursor_pos == 0 {
        return;
    }

    let chars: Vec<char> = app.text.chars().collect();
    let mut pos = app.cursor_pos - 1;

    while pos > 0 && chars[pos].is_whitespace() {
        pos -= 1;
    }

    while pos > 0 && !chars[pos].is_whitespace() {
        pos -= 1;
    }

    app.cursor_pos = pos;
}

fn move_to_line_start(app: &mut CatEditorApp) {
    let lines: Vec<&str> = app.text.lines().collect();
    let mut char_count = 0;

    for line in lines.iter() {
        if char_count + line.len() >= app.cursor_pos {
            app.cursor_pos = char_count;
            return;
        }
        char_count += line.len() + 1;
    }
}

fn move_to_line_end(app: &mut CatEditorApp) {
    let lines: Vec<&str> = app.text.lines().collect();
    let mut char_count = 0;

    for line in lines.iter() {
        if char_count + line.len() >= app.cursor_pos {
            app.cursor_pos = char_count + line.len();
            return;
        }
        char_count += line.len() + 1;
    }
}

fn move_to_first_non_blank(app: &mut CatEditorApp) {
    let lines: Vec<&str> = app.text.lines().collect();
    let mut char_count = 0;

    for line in lines.iter() {
        if char_count + line.len() >= app.cursor_pos {
            let first_non_blank = line.chars().position(|c| !c.is_whitespace()).unwrap_or(0);
            app.cursor_pos = char_count + first_non_blank;
            return;
        }

        char_count += line.len() + 1;
    }
}

fn move_to_last_non_blank(app: &mut CatEditorApp) {
    let lines: Vec<&str> = app.text.lines().collect();
    let mut char_count = 0;

    for line in lines.iter() {
        if char_count + line.len() >= app.cursor_pos {
            let last_non_blank = line.trim_end().len();
            app.cursor_pos = char_count + last_non_blank;
            return;
        }
        char_count += line.len() + 1;
    }
}

fn move_to_first_line(app: &mut CatEditorApp) {
    app.cursor_pos = 0;
}

fn move_to_last_line(app: &mut CatEditorApp) {
    app.cursor_pos = app.text.len();
}

fn move_to_next_paragraph(app: &mut CatEditorApp) {
    let lines: Vec<&str> = app.text.lines().collect();
    let mut char_count = 0;
    let mut current_line = 0;

    for (i, line) in lines.iter().enumerate() {
        if char_count + line.len() >= app.cursor_pos {
            current_line = i;
            break;
        }
        char_count += line.len() + 1;
    }

    let mut found = false;
    for i in (current_line + 1)..lines.len() {
        if lines[i].trim().is_empty() {
            found = true;
        } else if found {
            char_count = lines.iter().take(i).map(|l| l.len() + 1).sum();
            app.cursor_pos = char_count;
            return;
        }
    }

    app.cursor_pos = app.text.len();
}

fn move_to_prev_paragraph(app: &mut CatEditorApp) {
    let lines: Vec<&str> = app.text.lines().collect();
    let mut char_count = 0;
    let mut current_line = 0;

    for (i, line) in lines.iter().enumerate() {
        if char_count + line.len() >= app.cursor_pos {
            current_line = i;
            break;
        }

        char_count += line.len() + 1;
    }

    let mut found = false;
    for i in (0..current_line).rev() {
        if lines[i].trim().is_empty() {
            found = true;
        } else if found {
            char_count = lines.iter().take(i).map(|l| l.len() + 1).sum();
            app.cursor_pos = char_count;
            return;
        }
    }

    app.cursor_pos = 0;
}

fn move_page_down(app: &mut CatEditorApp) {
    let lines: Vec<&str> = app.text.lines().collect();
    let page_size = 20;
    let mut current_line = 0;
    let mut char_count = 0;

    for (i, line) in lines.iter().enumerate() {
        if char_count + line.len() >= app.cursor_pos {
            current_line = i;
            break;
        }
        char_count += line.len() + 1;
    }

    let target_line = (current_line + page_size).min(lines.len() - 1);
    char_count = lines.iter().take(target_line).map(|l| l.len() + 1).sum();
    app.cursor_pos = char_count;
}

fn move_page_up(app: &mut CatEditorApp) {
    let lines: Vec<&str> = app.text.lines().collect();
    let page_size = 20;
    let mut current_line = 0;
    let mut char_count = 0;

    for (i, line) in lines.iter().enumerate() {
        if char_count + line.len() >= app.cursor_pos {
            current_line = i;
            break;
        }

        char_count += lines.len() + 1;
    }

    let target_line = current_line.saturating_sub(page_size);
    char_count = lines.iter().take(target_line).map(|l| l.len() + 1).sum();
    app.cursor_pos = char_count;
}

fn move_half_page_down(app: &mut CatEditorApp) {
    let lines: Vec<&str> = app.text.lines().collect();
    let page_size = 10;
    let mut current_line = 0;
    let mut char_count = 0;

    for (i, line) in lines.iter().enumerate() {
            if char_count + line.len() >= app.cursor_pos {
                current_line = i;
                break;
            }

            char_count += line.len() + 1;
    }

    let target_line = (current_line + page_size).min(lines.len() - 1);
    char_count = lines.iter().take(target_line).map(|l| l.len() + 1).sum();
    app.cursor_pos = char_count;
}

fn move_half_page_up(app: &mut CatEditorApp) {
    let lines: Vec<&str> = app.text.lines().collect();
    let page_size = 10;
    let mut current_line: usize = 0;
    let mut char_count = 0;

    for (i, line) in lines.iter().enumerate() {
        if char_count + line.len() >= app.cursor_pos {
            current_line = i;
            break;
        }

        char_count += line.len() + 1;
    }

    let target_line = current_line.saturating_sub(page_size);
    char_count = lines.iter().take(target_line).map(|l| l.len() + 1).sum();
    app.cursor_pos = char_count;
}

fn find_char_forward(app: &mut CatEditorApp, target: char) {
    let chars: Vec<char> = app.text.chars().collect();
    for i in (app.cursor_pos + 1)..chars.len() {
        if chars[i] == target {
            app.cursor_pos = i;
            return;
        }
    }
}

fn find_char_to_forward(app: &mut CatEditorApp, target: char) {
    let chars: Vec<char> = app.text.chars().collect();
    for i in (app.cursor_pos + 1)..chars.len() {
        if chars[i] == target {
            app.cursor_pos = i - 1;
            return;
        }
    }
}

fn find_char_backward(app: &mut CatEditorApp, target: char) {
    let chars: Vec<char> = app.text.chars().collect();
    for i in (0..app.cursor_pos).rev() {
        if chars[i] == target {
            app.cursor_pos = i;
            return;
        }
    }
}

fn find_char_to_backward(app: &mut CatEditorApp, target: char) {
    let chars: Vec<char> = app.text.chars().collect();
    for i in (0..app.cursor_pos).rev() {
        if chars[i] == target {
            app.cursor_pos = i + 1;
            return
        }
    }
}