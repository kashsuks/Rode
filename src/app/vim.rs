use super::*;
use iced::widget::text_editor::{Action, Cursor, Motion, Position};

const VIEWPORT_LINES: usize = 60;

impl App {
    pub(super) fn vim_refresh_cursor_style(&mut self) {
        match self.vim_mode {
            VimMode::Normal => self.vim_apply_block_cursor(),
            VimMode::Insert => self.vim_clear_block_cursor(),
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
                    self.vim_apply_block_cursor();
                }
                iced::Task::none()
            }
            VimMode::Normal => {
                match key {
                    crate::message::VimKey::Escape => {
                        self.vim_pending.clear();
                        self.vim_count.clear();
                        self.vim_apply_block_cursor();
                    }
                    crate::message::VimKey::Ctrl(ch) => {
                        self.vim_pending.clear();
                        self.vim_apply_ctrl_motion(ch);
                    }
                    crate::message::VimKey::Char(ch) => {
                        self.vim_handle_char(ch);
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

    fn vim_handle_char(&mut self, ch: char) {
        if ch.is_ascii_digit() && self.vim_pending.is_empty() {
            if ch == '0' && self.vim_count.is_empty() {
                self.vim_move_line_start();
            } else {
                self.vim_count.push(ch);
            }
            return;
        }

        if !self.vim_pending.is_empty() {
            let pending = self.vim_pending.clone();
            self.vim_pending.clear();
            self.vim_dispatch_pending(&pending, ch);
            return;
        }

        match ch {
            'i' => {
                self.vim_mode = VimMode::Insert;
                self.vim_clear_block_cursor();
            }
            'h' => self.vim_apply_move(Motion::Left),
            'j' => self.vim_apply_move(Motion::Down),
            'k' => self.vim_apply_move(Motion::Up),
            'l' => self.vim_apply_move(Motion::Right),
            'w' | 'W' => self.vim_move_word_start_forward(ch == 'W'),
            'e' | 'E' => self.vim_move_word_end_forward(ch == 'E'),
            'b' | 'B' => self.vim_move_word_start_backward(ch == 'B'),
            '%' => self.vim_match_pair(),
            '^' => self.vim_move_first_nonblank(),
            '$' => self.vim_move_line_end(),
            'G' => self.vim_goto_line_or_end(),
            'H' => self.vim_move_to_screen_top(),
            'M' => self.vim_move_to_screen_middle(),
            'L' => self.vim_move_to_screen_bottom(),
            '{' => self.vim_move_paragraph_prev(),
            '}' => self.vim_move_paragraph_next(),
            ';' => self.vim_repeat_last_find(false),
            ',' => self.vim_repeat_last_find(true),
            'd' => {
                if !self.vim_delete_selected_range() {
                    self.vim_count.clear();
                }
            }
            'f' | 'F' | 't' | 'T' | 'g' | 'z' => {
                self.vim_pending.push(ch);
            }
            _ => {
                self.vim_count.clear();
            }
        }
    }

    fn vim_dispatch_pending(&mut self, pending: &str, ch: char) {
        match pending {
            "g" => match ch {
                'j' => self.vim_apply_move(Motion::Down),
                'k' => self.vim_apply_move(Motion::Up),
                'g' => {
                    let count = self.vim_take_count();
                    self.vim_goto_line(count);
                }
                'e' => self.vim_move_word_end_backward(false),
                'E' => self.vim_move_word_end_backward(true),
                '_' => self.vim_move_last_nonblank(),
                'd' => self.vim_goto_declaration(true),
                'D' => self.vim_goto_declaration(false),
                _ => {}
            },
            "z" => match ch {
                'z' => self.vim_center_cursor(),
                't' => self.vim_cursor_top(),
                'b' => self.vim_cursor_bottom(),
                _ => {}
            },
            "f" => self.vim_find_forward(ch, false),
            "t" => self.vim_find_forward(ch, true),
            "F" => self.vim_find_backward(ch, false),
            "T" => self.vim_find_backward(ch, true),
            _ => {}
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

    fn vim_apply_move(&mut self, motion: Motion) {
        let count = self.vim_take_count();
        for _ in 0..count {
            self.vim_apply_action(Action::Move(motion));
        }
    }

    fn vim_apply_ctrl_motion(&mut self, ch: char) {
        match ch {
            'e' => self.vim_scroll_only(1),
            'y' => self.vim_scroll_only(-1),
            'f' => self.vim_page_down(),
            'b' => self.vim_page_up(),
            'd' => self.vim_half_page_down(),
            'u' => self.vim_half_page_up(),
            _ => {}
        }
    }

    fn vim_apply_action(&mut self, action: Action) {
        if let Some(idx) = self.active_tab {
            let mut next_cursor: Option<(usize, usize)> = None;
            if let Some(tab) = self.tabs.get_mut(idx) {
                if let TabKind::Editor {
                    ref mut content,
                    ref mut scroll_line,
                    ..
                } = tab.kind
                {
                    let cursor = content.cursor();
                    content.move_to(Cursor {
                        position: cursor.position,
                        selection: None,
                    });
                    content.perform(action);
                    apply_block_cursor_on_content(content, self.vim_mode == VimMode::Normal);
                    let cursor = content.cursor().position;
                    let cl = cursor.line + 1;
                    let cc = cursor.column + 1;
                    *scroll_line = ensure_cursor_visible(cl, *scroll_line, content.line_count());
                    next_cursor = Some((cl, cc));
                }
            }
            if let Some((line, col)) = next_cursor {
                self.cursor_line = line;
                self.cursor_col = col;
            }
        }
    }

    fn vim_move_to(&mut self, line_1: usize, col_1: usize) {
        if let Some(idx) = self.active_tab {
            let mut next_cursor: Option<(usize, usize)> = None;
            if let Some(tab) = self.tabs.get_mut(idx) {
                if let TabKind::Editor {
                    ref mut content,
                    ref mut scroll_line,
                    ..
                } = tab.kind
                {
                    let line_0 = line_1.saturating_sub(1).min(content.line_count().saturating_sub(1));
                    let line_text = content
                        .line(line_0)
                        .map(|l| l.text.to_string())
                        .unwrap_or_default();
                    let max_col = line_text.chars().count();
                    let col_0 = col_1.saturating_sub(1).min(max_col);

                    content.move_to(Cursor {
                        position: Position {
                            line: line_0,
                            column: col_0,
                        },
                        selection: None,
                    });
                    apply_block_cursor_on_content(content, self.vim_mode == VimMode::Normal);
                    *scroll_line = ensure_cursor_visible(line_0 + 1, *scroll_line, content.line_count());
                    next_cursor = Some((line_0 + 1, col_0 + 1));
                }
            }
            if let Some((line, col)) = next_cursor {
                self.cursor_line = line;
                self.cursor_col = col;
            }
        }
    }

    fn vim_snapshot(&self) -> Option<(String, usize, usize)> {
        let idx = self.active_tab?;
        let tab = self.tabs.get(idx)?;
        if let TabKind::Editor { content, .. } = &tab.kind {
            Some((content.text(), self.cursor_line, self.cursor_col))
        } else {
            None
        }
    }

    fn vim_lines(&self) -> Option<Vec<String>> {
        let (text, _, _) = self.vim_snapshot()?;
        Some(text.split('\n').map(ToString::to_string).collect())
    }

    fn vim_move_line_start(&mut self) {
        self.vim_move_to(self.cursor_line, 1);
    }

    fn vim_move_first_nonblank(&mut self) {
        if let Some(lines) = self.vim_lines() {
            if let Some(line) = lines.get(self.cursor_line.saturating_sub(1)) {
                let col = line
                    .chars()
                    .position(|c| !c.is_whitespace())
                    .map(|i| i + 1)
                    .unwrap_or(1);
                self.vim_move_to(self.cursor_line, col);
            }
        }
    }

    fn vim_move_line_end(&mut self) {
        if let Some(lines) = self.vim_lines() {
            if let Some(line) = lines.get(self.cursor_line.saturating_sub(1)) {
                self.vim_move_to(self.cursor_line, line.chars().count() + 1);
            }
        }
    }

    fn vim_move_last_nonblank(&mut self) {
        if let Some(lines) = self.vim_lines() {
            if let Some(line) = lines.get(self.cursor_line.saturating_sub(1)) {
                let mut idx = None;
                for (i, c) in line.chars().enumerate() {
                    if !c.is_whitespace() {
                        idx = Some(i);
                    }
                }
                self.vim_move_to(self.cursor_line, idx.map(|i| i + 1).unwrap_or(1));
            }
        }
    }

    fn vim_goto_line_or_end(&mut self) {
        let count = self.vim_take_count();
        if count == 1 {
            if let Some((_, _, _)) = self.vim_snapshot() {
                let total = self.total_lines();
                self.vim_goto_line(total);
            }
        } else {
            self.vim_goto_line(count);
        }
    }

    fn vim_goto_line(&mut self, line_1: usize) {
        self.vim_move_to(line_1.max(1).min(self.total_lines()), self.cursor_col);
    }

    fn total_lines(&self) -> usize {
        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get(idx) {
                if let TabKind::Editor { content, .. } = &tab.kind {
                    return content.line_count().max(1);
                }
            }
        }
        1
    }

    fn vim_move_word_start_forward(&mut self, big: bool) {
        let count = self.vim_take_count();
        if let Some((text, line, col)) = self.vim_snapshot() {
            let lines = text.split('\n').collect::<Vec<_>>();
            let mut idx = position_to_index(&lines, line, col);
            for _ in 0..count {
                idx = next_word_start(&text, idx, big);
            }
            let (l, c) = index_to_position(&lines, idx);
            self.vim_move_to(l, c);
        }
    }

    fn vim_move_word_end_forward(&mut self, big: bool) {
        let count = self.vim_take_count();
        if let Some((text, line, col)) = self.vim_snapshot() {
            let lines = text.split('\n').collect::<Vec<_>>();
            let mut idx = position_to_index(&lines, line, col);
            for _ in 0..count {
                idx = next_word_end(&text, idx, big);
            }
            let (l, c) = index_to_position(&lines, idx);
            self.vim_move_to(l, c);
        }
    }

    fn vim_move_word_start_backward(&mut self, big: bool) {
        let count = self.vim_take_count();
        if let Some((text, line, col)) = self.vim_snapshot() {
            let lines = text.split('\n').collect::<Vec<_>>();
            let mut idx = position_to_index(&lines, line, col);
            for _ in 0..count {
                idx = prev_word_start(&text, idx, big);
            }
            let (l, c) = index_to_position(&lines, idx);
            self.vim_move_to(l, c);
        }
    }

    fn vim_move_word_end_backward(&mut self, big: bool) {
        let count = self.vim_take_count();
        if let Some((text, line, col)) = self.vim_snapshot() {
            let lines = text.split('\n').collect::<Vec<_>>();
            let mut idx = position_to_index(&lines, line, col);
            for _ in 0..count {
                idx = prev_word_end(&text, idx, big);
            }
            let (l, c) = index_to_position(&lines, idx);
            self.vim_move_to(l, c);
        }
    }

    fn vim_match_pair(&mut self) {
        if let Some((text, line, col)) = self.vim_snapshot() {
            let lines = text.split('\n').collect::<Vec<_>>();
            let idx = position_to_index(&lines, line, col);
            if let Some(target) = match_pair_index(&text, idx) {
                let (l, c) = index_to_position(&lines, target);
                self.vim_move_to(l, c);
            }
        }
    }

    fn vim_find_forward(&mut self, ch: char, till: bool) {
        self.vim_last_find = Some(VimFindState {
            kind: if till {
                VimFindKind::ForwardTill
            } else {
                VimFindKind::ForwardTo
            },
            needle: ch,
        });
        let count = self.vim_take_count();
        for _ in 0..count {
            self.vim_find_step(ch, VimFindKind::ForwardTo, till);
        }
    }

    fn vim_find_backward(&mut self, ch: char, till: bool) {
        self.vim_last_find = Some(VimFindState {
            kind: if till {
                VimFindKind::BackwardTill
            } else {
                VimFindKind::BackwardTo
            },
            needle: ch,
        });
        let count = self.vim_take_count();
        for _ in 0..count {
            self.vim_find_step(ch, VimFindKind::BackwardTo, till);
        }
    }

    fn vim_find_step(&mut self, ch: char, dir: VimFindKind, till: bool) {
        if let Some(lines) = self.vim_lines() {
            let line_idx = self.cursor_line.saturating_sub(1);
            if let Some(line) = lines.get(line_idx) {
                let chars: Vec<char> = line.chars().collect();
                let cur = self.cursor_col.saturating_sub(1).min(chars.len());
                match dir {
                    VimFindKind::ForwardTo | VimFindKind::ForwardTill => {
                        if let Some(pos) = chars.iter().skip(cur.saturating_add(1)).position(|c| *c == ch) {
                            let found = cur.saturating_add(1) + pos;
                            let dest = if till { found.saturating_sub(1) } else { found };
                            self.vim_move_to(self.cursor_line, dest + 1);
                        }
                    }
                    VimFindKind::BackwardTo | VimFindKind::BackwardTill => {
                        let mut found = None;
                        for i in (0..cur).rev() {
                            if chars[i] == ch {
                                found = Some(i);
                                break;
                            }
                        }
                        if let Some(found) = found {
                            let dest = if till { (found + 1).min(chars.len()) } else { found };
                            self.vim_move_to(self.cursor_line, dest + 1);
                        }
                    }
                }
            }
        }
    }

    fn vim_repeat_last_find(&mut self, reverse: bool) {
        if let Some(last) = self.vim_last_find {
            let kind = if reverse {
                reverse_find(last.kind)
            } else {
                last.kind
            };
            match kind {
                VimFindKind::ForwardTo => self.vim_find_step(last.needle, VimFindKind::ForwardTo, false),
                VimFindKind::ForwardTill => self.vim_find_step(last.needle, VimFindKind::ForwardTo, true),
                VimFindKind::BackwardTo => self.vim_find_step(last.needle, VimFindKind::BackwardTo, false),
                VimFindKind::BackwardTill => self.vim_find_step(last.needle, VimFindKind::BackwardTo, true),
            }
        }
    }

    fn vim_move_paragraph_next(&mut self) {
        if let Some(lines) = self.vim_lines() {
            let mut i = self.cursor_line;
            while i < lines.len() && !lines[i.saturating_sub(1)].trim().is_empty() {
                i += 1;
            }
            while i < lines.len() && lines[i.saturating_sub(1)].trim().is_empty() {
                i += 1;
            }
            self.vim_move_to(i.min(lines.len()).max(1), 1);
        }
    }

    fn vim_move_paragraph_prev(&mut self) {
        if let Some(lines) = self.vim_lines() {
            let mut i = self.cursor_line.saturating_sub(1);
            while i > 0 && lines[i.saturating_sub(1)].trim().is_empty() {
                i = i.saturating_sub(1);
            }
            while i > 0 && !lines[i.saturating_sub(1)].trim().is_empty() {
                i = i.saturating_sub(1);
            }
            self.vim_move_to(i.max(1), 1);
        }
    }

    fn vim_goto_declaration(&mut self, local: bool) {
        if let Some((text, line, col)) = self.vim_snapshot() {
            let lines = text.split('\n').collect::<Vec<_>>();
            let idx = position_to_index(&lines, line, col);
            let word = word_at_index(&text, idx);
            if word.is_empty() {
                return;
            }
            let patterns = [
                format!("let {word}"),
                format!("const {word}"),
                format!("fn {word}"),
                format!("def {word}"),
                format!("class {word}"),
                format!("var {word}"),
            ];
            let hay = if local { &text[..idx.min(text.len())] } else { &text };
            let mut found = None;
            for pat in patterns {
                if let Some(pos) = hay.rfind(&pat) {
                    found = Some(pos);
                    break;
                }
            }
            if let Some(pos) = found {
                let char_pos = hay[..pos].chars().count();
                let (l, c) = index_to_position(&lines, char_pos);
                self.vim_move_to(l, c);
            }
        }
    }

    fn vim_move_to_screen_top(&mut self) {
        let line = self.current_scroll_line();
        self.vim_move_to(line, self.cursor_col);
    }

    fn vim_move_to_screen_middle(&mut self) {
        let line = self.current_scroll_line().saturating_add(VIEWPORT_LINES / 2);
        self.vim_move_to(line.min(self.total_lines()), self.cursor_col);
    }

    fn vim_move_to_screen_bottom(&mut self) {
        let line = self
            .current_scroll_line()
            .saturating_add(VIEWPORT_LINES.saturating_sub(1));
        self.vim_move_to(line.min(self.total_lines()), self.cursor_col);
    }

    fn current_scroll_line(&self) -> usize {
        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get(idx) {
                if let TabKind::Editor { scroll_line, .. } = tab.kind {
                    return scroll_line;
                }
            }
        }
        1
    }

    fn vim_scroll_only(&mut self, delta: i32) {
        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get_mut(idx) {
                if let TabKind::Editor {
                    ref content,
                    ref mut scroll_line,
                    ..
                } = tab.kind
                {
                    let total = content.line_count().max(1);
                    let max_start = total.saturating_sub(VIEWPORT_LINES - 1).max(1);
                    let next = if delta > 0 {
                        scroll_line.saturating_add(delta as usize)
                    } else {
                        scroll_line.saturating_sub(delta.unsigned_abs() as usize)
                    };
                    *scroll_line = next.clamp(1, max_start);
                }
            }
        }
    }

    fn vim_page_down(&mut self) {
        self.vim_scroll_only(VIEWPORT_LINES as i32);
        let top = self.current_scroll_line();
        self.vim_move_to(top, self.cursor_col);
    }

    fn vim_page_up(&mut self) {
        self.vim_scroll_only(-(VIEWPORT_LINES as i32));
        let bottom = self
            .current_scroll_line()
            .saturating_add(VIEWPORT_LINES.saturating_sub(1));
        self.vim_move_to(bottom.min(self.total_lines()), self.cursor_col);
    }

    fn vim_half_page_down(&mut self) {
        let half = (VIEWPORT_LINES / 2) as i32;
        self.vim_scroll_only(half);
        let line = self.cursor_line.saturating_add(half as usize).min(self.total_lines());
        self.vim_move_to(line, self.cursor_col);
    }

    fn vim_half_page_up(&mut self) {
        let half = (VIEWPORT_LINES / 2) as i32;
        self.vim_scroll_only(-half);
        let line = self.cursor_line.saturating_sub(half as usize).max(1);
        self.vim_move_to(line, self.cursor_col);
    }

    fn vim_center_cursor(&mut self) {
        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get_mut(idx) {
                if let TabKind::Editor {
                    ref content,
                    ref mut scroll_line,
                    ..
                } = tab.kind
                {
                    let total = content.line_count().max(1);
                    let max_start = total.saturating_sub(VIEWPORT_LINES - 1).max(1);
                    *scroll_line = self
                        .cursor_line
                        .saturating_sub(VIEWPORT_LINES / 2)
                        .clamp(1, max_start);
                }
            }
        }
    }

    fn vim_cursor_top(&mut self) {
        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get_mut(idx) {
                if let TabKind::Editor {
                    ref mut scroll_line, ..
                } = tab.kind
                {
                    *scroll_line = self.cursor_line.max(1);
                }
            }
        }
    }

    fn vim_cursor_bottom(&mut self) {
        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get_mut(idx) {
                if let TabKind::Editor {
                    ref content,
                    ref mut scroll_line,
                    ..
                } = tab.kind
                {
                    let total = content.line_count().max(1);
                    let max_start = total.saturating_sub(VIEWPORT_LINES - 1).max(1);
                    *scroll_line = self
                        .cursor_line
                        .saturating_sub(VIEWPORT_LINES.saturating_sub(1))
                        .clamp(1, max_start);
                }
            }
        }
    }

    fn vim_apply_block_cursor(&mut self) {
        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get_mut(idx) {
                if let TabKind::Editor { ref mut content, .. } = tab.kind {
                    apply_block_cursor_on_content(content, true);
                    let cursor = content.cursor().position;
                    self.cursor_line = cursor.line + 1;
                    self.cursor_col = cursor.column + 1;
                }
            }
        }
    }

    fn vim_clear_block_cursor(&mut self) {
        if let Some(idx) = self.active_tab {
            if let Some(tab) = self.tabs.get_mut(idx) {
                if let TabKind::Editor { ref mut content, .. } = tab.kind {
                    let cursor = content.cursor();
                    content.move_to(Cursor {
                        position: cursor.position,
                        selection: None,
                    });
                    let cursor = content.cursor().position;
                    self.cursor_line = cursor.line + 1;
                    self.cursor_col = cursor.column + 1;
                }
            }
        }
    }

    fn vim_delete_selected_range(&mut self) -> bool {
        let Some(idx) = self.active_tab else {
            return false;
        };

        let mut lsp_update: Option<(std::path::PathBuf, String)> = None;

        if let Some(tab) = self.tabs.get_mut(idx) {
            if let TabKind::Editor {
                ref mut content,
                ref mut buffer,
                ref mut modified,
                ref mut scroll_line,
            } = tab.kind
            {
                let cursor = content.cursor();
                let Some(anchor) = cursor.selection else {
                    return false;
                };
                let head = cursor.position;
                if head == anchor {
                    return false;
                }

                let text = content.text();
                let lines: Vec<&str> = text.split('\n').collect();
                let a = position_to_index(&lines, anchor.line + 1, anchor.column + 1);
                let b = position_to_index(&lines, head.line + 1, head.column + 1);
                let start = a.min(b);
                let end = a.max(b);
                if start >= end {
                    return false;
                }

                let start_byte = char_to_byte_index(&text, start);
                let end_byte = char_to_byte_index(&text, end);
                let mut new_text = text;
                new_text.replace_range(start_byte..end_byte, "");

                *content = iced::widget::text_editor::Content::with_text(&new_text);
                buffer.set_text(&new_text);
                *modified = true;

                let new_lines: Vec<&str> = new_text.split('\n').collect();
                let (line_1, col_1) = index_to_position(&new_lines, start);
                content.move_to(Cursor {
                    position: Position {
                        line: line_1.saturating_sub(1),
                        column: col_1.saturating_sub(1),
                    },
                    selection: None,
                });
                apply_block_cursor_on_content(content, self.vim_mode == VimMode::Normal);

                let pos = content.cursor().position;
                self.cursor_line = pos.line + 1;
                self.cursor_col = pos.column + 1;
                *scroll_line = ensure_cursor_visible(self.cursor_line, *scroll_line, content.line_count());

                lsp_update = Some((tab.path.clone(), new_text));
            }
        }

        if let Some((path, text)) = lsp_update {
            self.lsp.change_document(path, text);
        }

        true
    }
}

fn ensure_cursor_visible(cursor_line: usize, scroll_line: usize, total_lines: usize) -> usize {
    let max_start = total_lines.saturating_sub(VIEWPORT_LINES - 1).max(1);
    if cursor_line < scroll_line {
        cursor_line
    } else {
        let bottom = scroll_line.saturating_add(VIEWPORT_LINES - 1);
        if cursor_line > bottom {
            cursor_line.saturating_sub(VIEWPORT_LINES - 1).clamp(1, max_start)
        } else {
            scroll_line.clamp(1, max_start)
        }
    }
}

fn reverse_find(kind: VimFindKind) -> VimFindKind {
    match kind {
        VimFindKind::ForwardTo => VimFindKind::BackwardTo,
        VimFindKind::ForwardTill => VimFindKind::BackwardTill,
        VimFindKind::BackwardTo => VimFindKind::ForwardTo,
        VimFindKind::BackwardTill => VimFindKind::ForwardTill,
    }
}

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

fn prev_word_end(text: &str, idx: usize, big: bool) -> usize {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return 0;
    }
    let mut i = idx.saturating_sub(1).min(chars.len().saturating_sub(1));
    while i > 0 && chars[i].is_whitespace() {
        i -= 1;
    }
    if big {
        i
    } else {
        while i + 1 < chars.len() && is_word_char(chars[i + 1]) {
            i += 1;
        }
        i
    }
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

fn word_at_index(text: &str, idx: usize) -> String {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return String::new();
    }
    let i = idx.min(chars.len().saturating_sub(1));
    if !is_word_char(chars[i]) {
        return String::new();
    }
    let mut start = i;
    while start > 0 && is_word_char(chars[start - 1]) {
        start -= 1;
    }
    let mut end = i;
    while end + 1 < chars.len() && is_word_char(chars[end + 1]) {
        end += 1;
    }
    chars[start..=end].iter().collect()
}

fn char_to_byte_index(text: &str, char_idx: usize) -> usize {
    if char_idx == 0 {
        return 0;
    }
    text.char_indices()
        .nth(char_idx)
        .map(|(i, _)| i)
        .unwrap_or(text.len())
}

fn apply_block_cursor_on_content(content: &mut iced::widget::text_editor::Content, enabled: bool) {
    let cursor = content.cursor();
    let line = cursor.position.line;
    let mut col = cursor.position.column;

    let line_text = content
        .line(line)
        .map(|l| l.text.to_string())
        .unwrap_or_default();
    let len = line_text.chars().count();

    if !enabled || len == 0 {
        content.move_to(Cursor {
            position: Position { line, column: col.min(len) },
            selection: None,
        });
        return;
    }

    if col >= len {
        col = len.saturating_sub(1);
    }

    content.move_to(Cursor {
        position: Position { line, column: col },
        selection: Some(Position {
            line,
            column: (col + 1).min(len),
        }),
    });
}
