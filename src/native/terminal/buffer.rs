use vte::ansi::{ClearMode, Handler, LineClearMode, Processor};

const MAX_SCROLLBACK_LINES: usize = 2_000;

pub(super) struct TerminalBuffer {
    processor: Processor,
    screen: Screen,
}

impl TerminalBuffer {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            processor: Processor::new(),
            screen: Screen::new(cols, rows),
        }
    }

    pub fn advance(&mut self, bytes: &[u8]) {
        self.processor.advance(&mut self.screen, bytes);
    }

    pub fn resize(&mut self, cols: usize, rows: usize) {
        self.screen.cols = cols.max(1);
        self.screen.rows = rows.max(1);
    }

    pub fn text(&self) -> String {
        self.screen.text()
    }
}

struct Screen {
    lines: Vec<Vec<char>>,
    cursor_row: usize,
    cursor_col: usize,
    saved_cursor: Option<(usize, usize)>,
    cols: usize,
    rows: usize,
}

impl Screen {
    fn new(cols: usize, rows: usize) -> Self {
        Self {
            lines: vec![Vec::new()],
            cursor_row: 0,
            cursor_col: 0,
            saved_cursor: None,
            cols: cols.max(1),
            rows: rows.max(1),
        }
    }

    fn current_line(&mut self) -> &mut Vec<char> {
        self.ensure_row(self.cursor_row);
        &mut self.lines[self.cursor_row]
    }

    fn ensure_row(&mut self, row: usize) {
        if self.lines.len() <= row {
            self.lines.resize_with(row + 1, Vec::new);
        }
    }

    fn put(&mut self, ch: char) {
        if self.cursor_col >= self.cols {
            self.cursor_col = 0;
            self.cursor_row += 1;
            self.ensure_row(self.cursor_row);
        }
        let col = self.cursor_col;
        let line = self.current_line();
        if line.len() < col {
            line.resize(col, ' ');
        }
        if line.len() == col {
            line.push(ch);
        } else {
            line[col] = ch;
        }
        self.cursor_col += 1;
    }

    fn linefeed(&mut self) {
        self.cursor_row += 1;
        self.ensure_row(self.cursor_row);
        self.trim_scrollback();
    }

    fn trim_scrollback(&mut self) {
        let max_lines = MAX_SCROLLBACK_LINES.max(self.rows);
        if self.lines.len() <= max_lines {
            return;
        }
        let remove = self.lines.len() - max_lines;
        self.lines.drain(..remove);
        self.cursor_row = self.cursor_row.saturating_sub(remove);
        self.saved_cursor = self
            .saved_cursor
            .map(|(row, col)| (row.saturating_sub(remove), col));
    }

    fn viewport_top(&self) -> usize {
        self.lines.len().saturating_sub(self.rows)
    }

    fn text(&self) -> String {
        let mut output = String::new();
        for (index, line) in self.lines.iter().enumerate() {
            let end = line
                .iter()
                .rposition(|character| *character != ' ')
                .map_or(0, |index| index + 1);
            if index > 0 {
                output.push('\n');
            }
            output.extend(line[..end].iter());
        }
        output
    }

    fn clear_line_range(&mut self, start: usize, end: usize) {
        let line = self.current_line();
        let end = end.min(line.len());
        let start = start.min(end);
        for character in &mut line[start..end] {
            *character = ' ';
        }
    }
}

impl Handler for Screen {
    fn input(&mut self, character: char) {
        self.put(character);
    }

    fn goto(&mut self, line: i32, col: usize) {
        self.cursor_row = self.viewport_top() + line.max(0) as usize;
        self.cursor_col = col;
        self.ensure_row(self.cursor_row);
    }

    fn goto_line(&mut self, line: i32) {
        self.cursor_row = self.viewport_top() + line.max(0) as usize;
        self.ensure_row(self.cursor_row);
    }

    fn goto_col(&mut self, col: usize) {
        self.cursor_col = col;
    }

    fn move_up(&mut self, rows: usize) {
        self.cursor_row = self.cursor_row.saturating_sub(rows);
    }

    fn move_down(&mut self, rows: usize) {
        self.cursor_row += rows;
        self.ensure_row(self.cursor_row);
    }

    fn move_forward(&mut self, cols: usize) {
        self.cursor_col = self.cursor_col.saturating_add(cols);
    }

    fn move_backward(&mut self, cols: usize) {
        self.cursor_col = self.cursor_col.saturating_sub(cols);
    }

    fn move_down_and_cr(&mut self, rows: usize) {
        self.move_down(rows);
        self.carriage_return();
    }

    fn move_up_and_cr(&mut self, rows: usize) {
        self.move_up(rows);
        self.carriage_return();
    }

    fn put_tab(&mut self, count: u16) {
        for _ in 0..count {
            let spaces = 8 - self.cursor_col % 8;
            for _ in 0..spaces {
                self.put(' ');
            }
        }
    }

    fn backspace(&mut self) {
        self.cursor_col = self.cursor_col.saturating_sub(1);
    }

    fn carriage_return(&mut self) {
        self.cursor_col = 0;
    }

    fn linefeed(&mut self) {
        Screen::linefeed(self);
    }

    fn newline(&mut self) {
        self.carriage_return();
        Screen::linefeed(self);
    }

    fn insert_blank(&mut self, count: usize) {
        let col = self.cursor_col;
        let line = self.current_line();
        let col = col.min(line.len());
        line.splice(col..col, std::iter::repeat_n(' ', count));
    }

    fn erase_chars(&mut self, count: usize) {
        self.clear_line_range(self.cursor_col, self.cursor_col.saturating_add(count));
    }

    fn delete_chars(&mut self, count: usize) {
        let col = self.cursor_col;
        let line = self.current_line();
        let end = col.saturating_add(count).min(line.len());
        if col < end {
            line.drain(col..end);
        }
    }

    fn insert_blank_lines(&mut self, count: usize) {
        let row = self.cursor_row.min(self.lines.len());
        self.lines
            .splice(row..row, std::iter::repeat_with(Vec::new).take(count));
        self.trim_scrollback();
    }

    fn delete_lines(&mut self, count: usize) {
        let end = self.cursor_row.saturating_add(count).min(self.lines.len());
        if self.cursor_row < end {
            self.lines.drain(self.cursor_row..end);
        }
        if self.lines.is_empty() {
            self.lines.push(Vec::new());
        }
        self.cursor_row = self.cursor_row.min(self.lines.len() - 1);
    }

    fn save_cursor_position(&mut self) {
        self.saved_cursor = Some((self.cursor_row, self.cursor_col));
    }

    fn restore_cursor_position(&mut self) {
        if let Some((row, col)) = self.saved_cursor {
            self.cursor_row = row;
            self.cursor_col = col;
            self.ensure_row(row);
        }
    }

    fn clear_line(&mut self, mode: LineClearMode) {
        let len = self.current_line().len();
        match mode {
            LineClearMode::Right => self.clear_line_range(self.cursor_col, len),
            LineClearMode::Left => self.clear_line_range(0, self.cursor_col.saturating_add(1)),
            LineClearMode::All => self.current_line().clear(),
        }
    }

    fn clear_screen(&mut self, mode: ClearMode) {
        match mode {
            ClearMode::All => {
                self.lines.clear();
                self.lines.push(Vec::new());
                self.cursor_row = 0;
                self.cursor_col = 0;
            }
            ClearMode::Saved => {
                let top = self.viewport_top();
                self.lines.drain(..top);
                self.cursor_row = self.cursor_row.saturating_sub(top);
            }
            ClearMode::Below => {
                self.clear_line(LineClearMode::Right);
                self.lines.truncate(self.cursor_row + 1);
            }
            ClearMode::Above => {
                for line in &mut self.lines[..self.cursor_row] {
                    line.clear();
                }
                self.clear_line(LineClearMode::Left);
            }
        }
    }

    fn reset_state(&mut self) {
        let cols = self.cols;
        let rows = self.rows;
        *self = Self::new(cols, rows);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vt_output_strips_styles_and_applies_carriage_return_updates() {
        let mut buffer = TerminalBuffer::new(80, 24);
        buffer.advance(b"\x1b[32mready\x1b[0m\r\nprogress 1\rprogress 2");
        assert_eq!(buffer.text(), "ready\nprogress 2");
    }

    #[test]
    fn vt_cursor_controls_replace_screen_content() {
        let mut buffer = TerminalBuffer::new(80, 24);
        buffer.advance(b"hello\r\nworld\x1b[1A\rHi");
        assert_eq!(buffer.text(), "Hillo\nworld");
    }

    #[test]
    fn scrollback_has_a_hard_line_bound() {
        let mut buffer = TerminalBuffer::new(80, 24);
        for _ in 0..2_100 {
            buffer.advance(b"line\n");
        }
        assert!(buffer.text().lines().count() <= MAX_SCROLLBACK_LINES);
    }
}
