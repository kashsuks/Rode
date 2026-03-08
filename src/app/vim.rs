use super::*;
use iced_code_editor::{ArrowDirection, Message as EditorMessage};

impl App {
    pub(super) fn vim_refresh_cursor_style(&mut self) {
        // With iced-code-editor, vim normal mode removes focus from
        // the canvas so the user cannot type. Insert mode restores it.
        match self.vim_mode {
            VimMode::Normal => {
                if let Some(idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get_mut(idx) {
                        if let TabKind::Editor { ref mut code_editor, .. } = tab.kind {
                            code_editor.lose_focus();
                        }
                    }
                }
            }
            VimMode::Insert => {
                if let Some(idx) = self.active_tab {
                    if let Some(tab) = self.tabs.get_mut(idx) {
                        if let TabKind::Editor { ref code_editor, .. } = tab.kind {
                            code_editor.request_focus();
                        }
                    }
                }
            }
        }
    }

    pub(super) fn handle_vim_key(&mut self, key: crate::message::VimKey) -> iced::Task<Message> {
        if !self.vim_context_active() {
            return iced::Task::none();
        }

        match self.vim_mode {
            VimMode::Insert => {
                if matches!(key, crate::message::VimKey::Escape) {
                    self.vim_mode = VimMode::Normal;
                    self.vim_pending.clear();
                    self.vim_count.clear();
                    self.vim_refresh_cursor_style();
                }
                iced::Task::none()
            }
            VimMode::Normal => {
                match key {
                    crate::message::VimKey::Escape => {
                        self.vim_pending.clear();
                        self.vim_count.clear();
                    }
                    crate::message::VimKey::Ctrl(ch) => {
                        self.vim_pending.clear();
                        return self.vim_apply_ctrl_motion(ch);
                    }
                    crate::message::VimKey::Char(ch) => {
                        return self.vim_handle_char(ch);
                    }
                    crate::message::VimKey::Enter | crate::message::VimKey::Backspace => {}
                }
                iced::Task::none()
            }
        }
    }

    pub(super) fn vim_context_active(&self) -> bool {
        self.active_tab.is_some()
            && !self.settings_open
            && !self.command_palette.open
            && !self.fuzzy_finder.open
            && !self.file_finder_visible
            && !self.search_visible
            && !self.command_input.open
    }

    fn vim_handle_char(&mut self, ch: char) -> iced::Task<Message> {
        if ch.is_ascii_digit() && self.vim_pending.is_empty() {
            if ch == '0' && self.vim_count.is_empty() {
                return self.vim_send_editor_msg(EditorMessage::Home(false));
            } else {
                self.vim_count.push(ch);
            }
            return iced::Task::none();
        }

        if !self.vim_pending.is_empty() {
            let pending = self.vim_pending.clone();
            self.vim_pending.clear();
            return self.vim_dispatch_pending(&pending, ch);
        }

        match ch {
            'i' => {
                self.vim_mode = VimMode::Insert;
                self.vim_refresh_cursor_style();
                iced::Task::none()
            }
            'a' => {
                // 'a' in vim: move right one char, then insert
                let task = self.vim_send_editor_msg(
                    EditorMessage::ArrowKey(ArrowDirection::Right, false),
                );
                self.vim_mode = VimMode::Insert;
                self.vim_refresh_cursor_style();
                task
            }
            'A' => {
                let task = self.vim_send_editor_msg(EditorMessage::End(false));
                self.vim_mode = VimMode::Insert;
                self.vim_refresh_cursor_style();
                task
            }
            'I' => {
                let task = self.vim_send_editor_msg(EditorMessage::Home(false));
                self.vim_mode = VimMode::Insert;
                self.vim_refresh_cursor_style();
                task
            }
            'o' => {
                // Open new line below
                let t1 = self.vim_send_editor_msg(EditorMessage::End(false));
                let t2 = self.vim_send_editor_msg(EditorMessage::Enter);
                self.vim_mode = VimMode::Insert;
                self.vim_refresh_cursor_style();
                iced::Task::batch([t1, t2])
            }
            'O' => {
                // Open new line above
                let t1 = self.vim_send_editor_msg(EditorMessage::Home(false));
                let t2 = self.vim_send_editor_msg(EditorMessage::Enter);
                let t3 = self.vim_send_editor_msg(
                    EditorMessage::ArrowKey(ArrowDirection::Up, false),
                );
                self.vim_mode = VimMode::Insert;
                self.vim_refresh_cursor_style();
                iced::Task::batch([t1, t2, t3])
            }
            'h' => self.vim_repeat_motion(ArrowDirection::Left),
            'j' => self.vim_repeat_motion(ArrowDirection::Down),
            'k' => self.vim_repeat_motion(ArrowDirection::Up),
            'l' => self.vim_repeat_motion(ArrowDirection::Right),
            'w' | 'W' => self.vim_word_motion_forward(ch == 'W'),
            'e' | 'E' => self.vim_word_motion_end(ch == 'E'),
            'b' | 'B' => self.vim_word_motion_backward(ch == 'B'),
            '%' => self.vim_match_pair(),
            '^' => self.vim_move_first_nonblank(),
            '$' => self.vim_send_editor_msg(EditorMessage::End(false)),
            'G' => self.vim_goto_end_or_line(),
            'x' => {
                // Delete char under cursor
                self.vim_count.clear();
                self.vim_send_editor_msg(EditorMessage::Delete)
            }
            'H' | 'M' | 'L' => {
                // Screen-relative motions - limited support, just use
                // page-level navigation
                self.vim_count.clear();
                iced::Task::none()
            }
            '{' => self.vim_move_paragraph_prev(),
            '}' => self.vim_move_paragraph_next(),
            ';' => self.vim_repeat_last_find(false),
            ',' => self.vim_repeat_last_find(true),
            'd' => {
                self.vim_pending.push('d');
                iced::Task::none()
            }
            'f' | 'F' | 't' | 'T' | 'g' | 'z' => {
                self.vim_pending.push(ch);
                iced::Task::none()
            }
            _ => {
                self.vim_count.clear();
                iced::Task::none()
            }
        }
    }

    fn vim_dispatch_pending(&mut self, pending: &str, ch: char) -> iced::Task<Message> {
        match pending {
            "g" => match ch {
                'g' => {
                    self.vim_count.clear();
                    self.vim_send_editor_msg(EditorMessage::CtrlHome)
                }
                _ => {
                    self.vim_count.clear();
                    iced::Task::none()
                }
            },
            "z" => {
                self.vim_count.clear();
                iced::Task::none()
            }
            "d" => match ch {
                'd' => self.vim_delete_line(),
                'w' => self.vim_delete_word(),
                _ => {
                    self.vim_count.clear();
                    iced::Task::none()
                }
            },
            "f" => self.vim_find_char(ch, false, false),
            "t" => self.vim_find_char(ch, false, true),
            "F" => self.vim_find_char(ch, true, false),
            "T" => self.vim_find_char(ch, true, true),
            _ => {
                self.vim_count.clear();
                iced::Task::none()
            }
        }
    }

    fn vim_take_count(&mut self) -> usize {
        if self.vim_count.is_empty() {
            1
        } else {
            let parsed = self.vim_count.parse::<usize>().unwrap_or(1);
            self.vim_count.clear();
            parsed.max(1)
        }
    }

    /// Send a message to the active tab's CodeEditor and return the resulting Task.
    fn vim_send_editor_msg(&mut self, msg: EditorMessage) -> iced::Task<Message> {
        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get_mut(idx) {
                if let TabKind::Editor {
                    ref mut code_editor,
                    ref mut buffer,
                    ..
                } = tab.kind
                {
                    let task = code_editor.update(&msg);
                    buffer.set_text(&code_editor.content());
                    self.lsp
                        .change_document(tab.path.clone(), code_editor.content());
                    return task.map(Message::CodeEditorEvent);
                }
            }
        }
        iced::Task::none()
    }

    fn vim_repeat_motion(&mut self, dir: ArrowDirection) -> iced::Task<Message> {
        let count = self.vim_take_count();
        let mut tasks = Vec::with_capacity(count);
        for _ in 0..count {
            tasks.push(self.vim_send_editor_msg(
                EditorMessage::ArrowKey(dir, false),
            ));
        }
        iced::Task::batch(tasks)
    }

    fn vim_goto_end_or_line(&mut self) -> iced::Task<Message> {
        let count = self.vim_take_count();
        if count == 1 && self.vim_count.is_empty() {
            // G with no count = end of file
            self.vim_send_editor_msg(EditorMessage::CtrlEnd)
        } else {
            // Ngg = go to line N (approximate via CtrlHome + N-1 ArrowDown)
            let mut tasks = vec![self.vim_send_editor_msg(EditorMessage::CtrlHome)];
            for _ in 0..count.saturating_sub(1) {
                tasks.push(self.vim_send_editor_msg(
                    EditorMessage::ArrowKey(ArrowDirection::Down, false),
                ));
            }
            iced::Task::batch(tasks)
        }
    }

    fn vim_apply_ctrl_motion(&mut self, ch: char) -> iced::Task<Message> {
        match ch {
            'f' => {
                self.vim_count.clear();
                self.vim_send_editor_msg(EditorMessage::PageDown)
            }
            'b' => {
                self.vim_count.clear();
                self.vim_send_editor_msg(EditorMessage::PageUp)
            }
            'd' => {
                // Half page down: approximate with multiple ArrowDown
                let half = 30;
                let mut tasks = Vec::with_capacity(half);
                for _ in 0..half {
                    tasks.push(self.vim_send_editor_msg(
                        EditorMessage::ArrowKey(ArrowDirection::Down, false),
                    ));
                }
                iced::Task::batch(tasks)
            }
            'u' => {
                // Half page up
                let half = 30;
                let mut tasks = Vec::with_capacity(half);
                for _ in 0..half {
                    tasks.push(self.vim_send_editor_msg(
                        EditorMessage::ArrowKey(ArrowDirection::Up, false),
                    ));
                }
                iced::Task::batch(tasks)
            }
            _ => iced::Task::none(),
        }
    }

    // --- Word motions --- //

    fn vim_content_text(&self) -> Option<String> {
        let idx = self.active_tab?;
        let tab = self.tabs.get(idx)?;
        if let TabKind::Editor { ref code_editor, .. } = tab.kind {
            Some(code_editor.content())
        } else {
            None
        }
    }

    fn vim_word_motion_forward(&mut self, big: bool) -> iced::Task<Message> {
        let count = self.vim_take_count();
        let Some(text) = self.vim_content_text() else {
            return iced::Task::none();
        };
        let lines: Vec<&str> = text.split('\n').collect();
        let mut idx = position_to_index(&lines, self.cursor_line, self.cursor_col);
        for _ in 0..count {
            idx = next_word_start(&text, idx, big);
        }
        let (target_line, target_col) = index_to_position(&lines, idx);
        self.vim_goto_position(target_line, target_col)
    }

    fn vim_word_motion_end(&mut self, big: bool) -> iced::Task<Message> {
        let count = self.vim_take_count();
        let Some(text) = self.vim_content_text() else {
            return iced::Task::none();
        };
        let lines: Vec<&str> = text.split('\n').collect();
        let mut idx = position_to_index(&lines, self.cursor_line, self.cursor_col);
        for _ in 0..count {
            idx = next_word_end(&text, idx, big);
        }
        let (target_line, target_col) = index_to_position(&lines, idx);
        self.vim_goto_position(target_line, target_col)
    }

    fn vim_word_motion_backward(&mut self, big: bool) -> iced::Task<Message> {
        let count = self.vim_take_count();
        let Some(text) = self.vim_content_text() else {
            return iced::Task::none();
        };
        let lines: Vec<&str> = text.split('\n').collect();
        let mut idx = position_to_index(&lines, self.cursor_line, self.cursor_col);
        for _ in 0..count {
            idx = prev_word_start(&text, idx, big);
        }
        let (target_line, target_col) = index_to_position(&lines, idx);
        self.vim_goto_position(target_line, target_col)
    }

    fn vim_match_pair(&mut self) -> iced::Task<Message> {
        self.vim_count.clear();
        let Some(text) = self.vim_content_text() else {
            return iced::Task::none();
        };
        let lines: Vec<&str> = text.split('\n').collect();
        let idx = position_to_index(&lines, self.cursor_line, self.cursor_col);
        if let Some(target) = match_pair_index(&text, idx) {
            let (l, c) = index_to_position(&lines, target);
            self.vim_goto_position(l, c)
        } else {
            iced::Task::none()
        }
    }

    fn vim_move_first_nonblank(&mut self) -> iced::Task<Message> {
        self.vim_count.clear();
        let Some(text) = self.vim_content_text() else {
            return iced::Task::none();
        };
        let lines: Vec<&str> = text.split('\n').collect();
        let line_idx = self.cursor_line.saturating_sub(1).min(lines.len().saturating_sub(1));
        if let Some(line) = lines.get(line_idx) {
            let col = line
                .chars()
                .position(|c| !c.is_whitespace())
                .map(|i| i + 1)
                .unwrap_or(1);
            self.vim_goto_position(self.cursor_line, col)
        } else {
            iced::Task::none()
        }
    }

    fn vim_move_paragraph_next(&mut self) -> iced::Task<Message> {
        self.vim_count.clear();
        let Some(text) = self.vim_content_text() else {
            return iced::Task::none();
        };
        let lines: Vec<&str> = text.split('\n').collect();
        let mut i = self.cursor_line;
        while i < lines.len() && !lines[i.saturating_sub(1)].trim().is_empty() {
            i += 1;
        }
        while i < lines.len() && lines[i.saturating_sub(1)].trim().is_empty() {
            i += 1;
        }
        self.vim_goto_position(i.min(lines.len()).max(1), 1)
    }

    fn vim_move_paragraph_prev(&mut self) -> iced::Task<Message> {
        self.vim_count.clear();
        let Some(text) = self.vim_content_text() else {
            return iced::Task::none();
        };
        let lines: Vec<&str> = text.split('\n').collect();
        let mut i = self.cursor_line.saturating_sub(1);
        while i > 0 && lines[i.saturating_sub(1)].trim().is_empty() {
            i = i.saturating_sub(1);
        }
        while i > 0 && !lines[i.saturating_sub(1)].trim().is_empty() {
            i = i.saturating_sub(1);
        }
        self.vim_goto_position(i.max(1), 1)
    }

    /// Move cursor to an absolute position using CtrlHome + arrow keys.
    fn vim_goto_position(&mut self, target_line: usize, target_col: usize) -> iced::Task<Message> {
        let mut tasks = vec![self.vim_send_editor_msg(EditorMessage::CtrlHome)];
        let line_moves = target_line.saturating_sub(1);
        for _ in 0..line_moves {
            tasks.push(self.vim_send_editor_msg(
                EditorMessage::ArrowKey(ArrowDirection::Down, false),
            ));
        }
        // Use Home to ensure we're at column 1, then move to the target column
        tasks.push(self.vim_send_editor_msg(EditorMessage::Home(false)));
        let col_moves = target_col.saturating_sub(1);
        for _ in 0..col_moves {
            tasks.push(self.vim_send_editor_msg(
                EditorMessage::ArrowKey(ArrowDirection::Right, false),
            ));
        }
        self.cursor_line = target_line;
        self.cursor_col = target_col;
        iced::Task::batch(tasks)
    }

    // --- Find char motions --- //

    fn vim_find_char(
        &mut self,
        ch: char,
        backward: bool,
        till: bool,
    ) -> iced::Task<Message> {
        self.vim_last_find = Some(VimFindState {
            kind: match (backward, till) {
                (false, false) => VimFindKind::ForwardTo,
                (false, true) => VimFindKind::ForwardTill,
                (true, false) => VimFindKind::BackwardTo,
                (true, true) => VimFindKind::BackwardTill,
            },
            needle: ch,
        });
        let count = self.vim_take_count();
        let Some(text) = self.vim_content_text() else {
            return iced::Task::none();
        };
        let lines: Vec<&str> = text.split('\n').collect();
        let line_idx = self.cursor_line.saturating_sub(1).min(lines.len().saturating_sub(1));
        let Some(line) = lines.get(line_idx) else {
            return iced::Task::none();
        };
        let chars: Vec<char> = line.chars().collect();
        let cur = self.cursor_col.saturating_sub(1).min(chars.len());

        let mut result_col = None;
        if !backward {
            let mut found_count = 0;
            for (pos, &c) in chars.iter().enumerate().skip(cur.saturating_add(1)) {
                if c == ch {
                    found_count += 1;
                    if found_count == count {
                        result_col = Some(if till { pos.saturating_sub(1) } else { pos });
                        break;
                    }
                }
            }
        } else {
            let mut found_count = 0;
            for i in (0..cur).rev() {
                if chars[i] == ch {
                    found_count += 1;
                    if found_count == count {
                        result_col = Some(
                            if till { (i + 1).min(chars.len()) } else { i },
                        );
                        break;
                    }
                }
            }
        }

        if let Some(col) = result_col {
            self.vim_goto_position(self.cursor_line, col + 1)
        } else {
            iced::Task::none()
        }
    }

    fn vim_repeat_last_find(&mut self, reverse: bool) -> iced::Task<Message> {
        if let Some(last) = self.vim_last_find {
            let (backward, till) = if reverse {
                match last.kind {
                    VimFindKind::ForwardTo => (true, false),
                    VimFindKind::ForwardTill => (true, true),
                    VimFindKind::BackwardTo => (false, false),
                    VimFindKind::BackwardTill => (false, true),
                }
            } else {
                match last.kind {
                    VimFindKind::ForwardTo => (false, false),
                    VimFindKind::ForwardTill => (false, true),
                    VimFindKind::BackwardTo => (true, false),
                    VimFindKind::BackwardTill => (true, true),
                }
            };
            self.vim_find_char(last.needle, backward, till)
        } else {
            iced::Task::none()
        }
    }

    // --- Delete operations --- //

    fn vim_delete_line(&mut self) -> iced::Task<Message> {
        self.vim_count.clear();
        // Select entire current line and delete it
        let t1 = self.vim_send_editor_msg(EditorMessage::Home(false));
        // Select to end of line
        let t2 = self.vim_send_editor_msg(EditorMessage::End(true));
        // Delete the selection
        let t3 = self.vim_send_editor_msg(EditorMessage::Backspace);
        // Delete the remaining newline
        let t4 = self.vim_send_editor_msg(EditorMessage::Backspace);
        iced::Task::batch([t1, t2, t3, t4])
    }

    fn vim_delete_word(&mut self) -> iced::Task<Message> {
        self.vim_count.clear();
        // Approximate: select word forward with shift+right arrows then delete
        let Some(text) = self.vim_content_text() else {
            return iced::Task::none();
        };
        let lines: Vec<&str> = text.split('\n').collect();
        let idx = position_to_index(&lines, self.cursor_line, self.cursor_col);
        let end = next_word_start(&text, idx, false);
        let chars_to_select = end.saturating_sub(idx);

        let mut tasks = Vec::with_capacity(chars_to_select + 1);
        for _ in 0..chars_to_select {
            tasks.push(self.vim_send_editor_msg(
                EditorMessage::ArrowKey(ArrowDirection::Right, true),
            ));
        }
        tasks.push(self.vim_send_editor_msg(EditorMessage::Backspace));
        iced::Task::batch(tasks)
    }
}

// --- Helper functions (preserved from original) --- //

fn is_word_char(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

fn position_to_index(lines: &[&str], line_1: usize, col_1: usize) -> usize {
    let mut idx = 0usize;
    let line_idx = line_1.saturating_sub(1).min(lines.len().saturating_sub(1));
    for line in &lines[..line_idx] {
        idx += line.chars().count() + 1;
    }
    let line_len = lines.get(line_idx).map(|l| l.chars().count()).unwrap_or(0);
    idx + col_1.saturating_sub(1).min(line_len)
}

fn index_to_position(lines: &[&str], mut idx: usize) -> (usize, usize) {
    for (i, line) in lines.iter().enumerate() {
        let len = line.chars().count();
        if idx <= len {
            return (i + 1, idx + 1);
        }
        idx = idx.saturating_sub(len + 1);
    }
    let last = lines.len().max(1);
    let col = lines.last().map(|l| l.chars().count() + 1).unwrap_or(1);
    (last, col)
}

fn next_word_start(text: &str, idx: usize, big: bool) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let mut i = idx.min(chars.len());
    while i < chars.len() && if big { !chars[i].is_whitespace() } else { is_word_char(chars[i]) } {
        i += 1;
    }
    while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
    }
    while i < chars.len()
        && !chars[i].is_whitespace()
        && !big
        && !is_word_char(chars[i])
    {
        i += 1;
    }
    i
}

fn next_word_end(text: &str, idx: usize, big: bool) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let mut i = idx.min(chars.len());
    while i < chars.len() && chars[i].is_whitespace() {
        i += 1;
    }
    while i < chars.len()
        && if big {
            !chars[i].is_whitespace()
        } else {
            is_word_char(chars[i])
        }
    {
        i += 1;
    }
    i.saturating_sub(1)
}

fn prev_word_start(text: &str, idx: usize, big: bool) -> usize {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return 0;
    }
    let mut i = idx.saturating_sub(1).min(chars.len().saturating_sub(1));
    while i > 0 && chars[i].is_whitespace() {
        i -= 1;
    }
    while i > 0
        && if big {
            !chars[i - 1].is_whitespace()
        } else {
            is_word_char(chars[i - 1])
        }
    {
        i -= 1;
    }
    i
}

fn match_pair_index(text: &str, idx: usize) -> Option<usize> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return None;
    }
    let i = idx.min(chars.len().saturating_sub(1));
    let ch = chars[i];
    let (open, close, forward) = match ch {
        '(' => ('(', ')', true),
        '[' => ('[', ']', true),
        '{' => ('{', '}', true),
        ')' => ('(', ')', false),
        ']' => ('[', ']', false),
        '}' => ('{', '}', false),
        _ => return None,
    };
    let mut depth = 0i32;
    if forward {
        for (j, c) in chars.iter().enumerate().skip(i) {
            if *c == open {
                depth += 1;
            } else if *c == close {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
        }
    } else {
        for j in (0..=i).rev() {
            let c = chars[j];
            if c == close {
                depth += 1;
            } else if c == open {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
        }
    }
    None
}
