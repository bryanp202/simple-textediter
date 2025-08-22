pub struct WindowState {
    start_line: usize,
    line_count: usize,
    line_char_count: usize,
}

impl WindowState {
    pub fn new(window_width: u32, window_height: u32, text_width: u32, text_height: u32) -> Self {
        let mut new_window_state = Self::default();
        new_window_state.resize(window_width, window_height, text_width, text_height);
        new_window_state
    }

    pub fn resize(&mut self, window_width: u32, window_height: u32, text_width: u32, text_height: u32) {
        self.line_count = window_height.div_ceil(text_height) as usize;
        self.line_char_count = window_width.div_ceil(text_width) as usize;
    }

    pub fn get_first_line(&self) -> usize {
        self.start_line
    }

    pub fn lines(&self) -> usize {
        self.line_count
    }

    pub fn chars(&self) -> usize {
        self.line_char_count
    }
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            start_line: 0,
            line_count: 0,
            line_char_count: 0,
        }
    }
}