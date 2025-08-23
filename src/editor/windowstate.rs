use crate::editor::rope::TextRope;

pub struct WindowState {
    start_line: usize,
    start_char: usize,
    line_count: usize,
    line_char_count: usize,
}

impl WindowState {
    pub fn new(window_width: u32, window_height: u32, text_width: u32, text_height: u32, text_pad: u32, line_pad: u32) -> Self {
        let mut new_window_state = Self::default();
        new_window_state.resize(window_width, window_height, text_width, text_height, text_pad, line_pad);
        new_window_state
    }

    pub fn resize(&mut self, window_width: u32, window_height: u32, text_width: u32, text_height: u32, text_pad: u32, line_pad: u32) {
        let window_height = window_height.saturating_sub(text_pad);
        let window_width = window_width.saturating_sub(text_pad);
        let text_height = text_height + line_pad;
        self.line_count = (window_height / text_height) as usize;
        self.line_char_count = (window_width / text_width) as usize;
    }

    pub fn get_first_line(&self) -> usize {
        self.start_line
    }

    pub fn get_first_char(&self) -> usize {
        self.start_char
    }

    pub fn lines(&self) -> usize {
        self.line_count
    }

    pub fn chars(&self) -> usize {
        self.line_char_count
    }

    pub fn scroll_up(&mut self, distance: usize) {
        self.start_line = self.start_line.saturating_sub(distance);
    }

    pub fn scroll_down(&mut self, distance: usize, max_line_count: usize) {
        self.start_line = (self.start_line + distance).min(max_line_count.saturating_sub(self.line_count));
    }

    pub fn adjust_focus(&mut self, x: usize, y: usize, text_data: &TextRope) {
        let new_char_start = if x < self.start_char {
            x
        } else if x + 1 >= self.start_char + self.line_char_count {
            x + 1 - self.line_char_count
        } else {
            self.start_char
        };

        let new_line_start = if y < self.start_line {
            y
        } else if y + 1 >= self.start_line + self.line_count {
            y + 1 - self.line_count
        } else {
            self.start_line
        };

        self.start_char = new_char_start.min(text_data.lines().nth(y).unwrap().chars().count().saturating_sub(self.line_char_count / 4));
        self.start_line = new_line_start.min(text_data.line_count().saturating_sub(self.line_count));
    }
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            start_line: 0,
            start_char: 0,
            line_count: 0,
            line_char_count: 0,
        }
    }
}