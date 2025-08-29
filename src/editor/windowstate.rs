use crate::{editor::textrope::TextRope, vector::Vector2D};

pub struct WindowState {
    start_line: usize,
    start_char: usize,
    line_count: usize,
    line_char_count: usize,

    text_padding: u32,
    line_padding: u32,
    should_render: bool,

    text_width: f32,
    text_height: f32,
    window_width: u32,
    window_height: u32,
    pos: Vector2D,
}

impl WindowState {
    const SCROLL_FACTOR: usize = 8;
    pub fn new(pos: Vector2D, window_width: u32, window_height: u32, text_width: u32, text_height: u32, text_padding: u32, line_padding: u32) -> Self {
        let mut new_window_state = Self {
            pos,
            text_padding,
            line_padding,
            window_width,
            window_height,
            ..Default::default()
        };
        new_window_state.resize_text(text_width, text_height);
        new_window_state
    }

    pub fn check_render(&mut self) -> bool {
        let should_render = self.should_render;
        self.should_render = false;
        should_render
    }

    pub fn set_render_flag(&mut self) {
        self.should_render = true;
    }

    pub fn get_pos(&self) -> Vector2D {
        self.pos
    }

    /// Returns (width, height)
    pub fn get_text_dim(&self) -> (f32, f32) {
        (self.text_width, self.text_height)
    }

    /// returns (width, height)
    pub fn get_window_dim(&self) -> (u32, u32) {
        (self.window_width, self.window_height)
    }

    pub fn pos(&self) -> Vector2D {
        self.pos
    }

    #[allow(dead_code)]
    pub fn resize(&mut self, window_width: i32, window_height: i32, ) {
        self.window_height = window_height as u32;
        self.window_width = window_width as u32;
    }

    pub fn resize_text(&mut self, text_width: u32, text_height: u32) {
        self.text_height = text_height as f32;
        self.text_width = text_width as f32;

        let window_height = self.window_height.saturating_sub(self.text_padding);
        let window_width = self.window_width.saturating_sub(self.text_padding);
        let text_height = text_height + self.line_padding;
        self.line_count = (window_height / text_height) as usize - 1;
        self.line_char_count = (window_width / text_width) as usize;
        self.should_render = true;
    }

    pub fn in_screen_bound(&self, x: u32, y: u32) -> Option<Vector2D> {
        let window_first_char = self.start_char as u32;
        let window_first_line = self.start_line as u32;
        let window_char_len = self.line_char_count as u32;
        let window_line_len = self.line_count as u32;

        let shifted_x = if x < window_first_char || x >= window_char_len + window_first_char {
            return None;
        } else {
            x - window_first_char
        };
        let shifted_y = if y < window_first_line || y >= window_line_len + window_first_line {
            return None;
        } else {
            y - window_first_line
        };
        Some(Vector2D {x: shifted_x, y: shifted_y })
    }

    pub fn is_in_screen_bound(&self, x: u32, y: u32) -> bool {
        let (w, h) = self.get_window_dim();
        let (start_x, start_y) = self.get_pos().into();

        x >= start_x && x < (start_x + w) && y >= start_y && y < (start_y + h)
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

    /// Returns (text_padding, line_padding)
    pub fn get_padding(&self) -> (u32, u32) {
        (self.text_padding, self.line_padding)
    }

    pub fn scroll_up(&mut self, distance: usize) {
        self.start_line = self.start_line.saturating_sub(distance * Self::SCROLL_FACTOR);
        self.should_render = true;
    }

    pub fn scroll_down(&mut self, distance: usize, max_line_count: usize) {
        self.start_line = (self.start_line + distance * Self::SCROLL_FACTOR).min(max_line_count.saturating_sub(self.line_count));
        self.should_render = true;
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
        self.should_render = true;
    }
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            start_line: 0,
            start_char: 0,
            line_count: 0,
            line_char_count: 0,
            text_padding: 0,
            line_padding: 0,
            should_render: false,
            text_height: 0.0,
            text_width: 0.0,
            pos: Vector2D::default(),
            window_height: 0,
            window_width: 0,
        }
    }
}