use std::error::Error;

use sdl3::{event::Event, keyboard::Keycode, pixels::Color, render::{Canvas, FRect, TextureCreator, TextureQuery}, ttf::{Font, FontStyle, Sdl3TtfContext}, video::{Window, WindowContext}, VideoSubsystem};

use crate::{editor::{command::Command, cursor::Cursor, draw, inputstate::InputState, textrope::TextRope, windowstate::WindowState, TextAlignment}, vector::Vector2D};

const DEFAULT_FONT_PATH: &str = r"C:\Windows\Fonts\consola.ttf";
const DEFAULT_FONT_SIZE: f32 = 24.0;
const MAX_FONT_SIZE: f32 = 126.0;
const MIN_FONT_SIZE: f32 = 12.0;
const FONT_ZOOM_INCREMENT: f32 = 2.0;
const DEFAULT_FONT_STYLE: FontStyle = FontStyle::NORMAL;
const DEFAULT_BACKGROUND_COLOR: Color = Color::RGB(20, 20, 20);
pub const DEFAULT_FONT_COLOR: Color = Color::RGB(180, 225, 225);
const DEFAULT_FONT_SELECT_COLOR: Color = Color::RGB(80, 80, 80);
const DEFAULT_TEXT_PADDING: u32 = 16;
const DEFAULT_LINE_PADDING: u32 = 2;
const TAB_SPACE_COUNT: u32 = 4;
const TAB_SPACE_STRING: &str = "    ";

pub struct TextBox<'a> {
    active: bool,
    text: TextRope,
    window: WindowState,
    cursor: Cursor,
    font: Font<'a>,
    font_size: f32,

    background_color: Color,
    font_color: Color,
    font_select_color: Color,

    /// Context
    video_subsystem: &'a VideoSubsystem,
    ttf_context: &'a Sdl3TtfContext,
}

impl <'a> TextBox<'a> {
    pub fn build(
        pos: Vector2D,
        window_width: u32,
        window_height: u32,
        background_color: Option<Color>,
        video_subsystem: &'a VideoSubsystem,
        ttf_context: &'a Sdl3TtfContext,
    ) -> Result<Self, Box<dyn Error>> {
        let mut default_font = ttf_context.load_font(DEFAULT_FONT_PATH, DEFAULT_FONT_SIZE)?;
        default_font.set_style(DEFAULT_FONT_STYLE);
        let (text_width, text_height) = default_font.size_of_char('|')?;

        let window = WindowState::new(
            pos,
            window_width,
            window_height,
            text_width,
            text_height,
            DEFAULT_TEXT_PADDING,
            DEFAULT_LINE_PADDING,
        );

        Ok(
            Self {
                active: false,
                text: TextRope::new(),
                window,
                cursor: Cursor::new(),
                font: default_font,
                font_size: DEFAULT_FONT_SIZE,

                background_color: background_color.unwrap_or(DEFAULT_BACKGROUND_COLOR),
                font_color: DEFAULT_FONT_COLOR,
                font_select_color: DEFAULT_FONT_SELECT_COLOR,

                video_subsystem,
                ttf_context,
            }
        )
    }
}

impl <'a> TextBox<'a> {
    pub fn execute_cmd(&mut self, cmd: Command) {
        match cmd {
            Command::JUMP(col, line) => self.cursor.snap_to_pos(col, line, &self.text, &mut self.window),
            _ => {},
        }
    }
}

impl <'a> TextBox<'a> {
    pub fn handle_input(&mut self, event: Event, input: &InputState) -> Result<(), Box<dyn Error>> {
        match event {
            // Keyboard input
            Event::KeyDown { keycode: Some(Keycode::Home), .. } => self.cursor.home(&input, &self.text, &mut self.window),
            Event::KeyDown { keycode: Some(Keycode::Delete), .. } => self.delete_text(),
            Event::KeyDown { keycode: Some(Keycode::Backspace), .. } => self.remove_text(1),
            Event::KeyDown { keycode: Some(Keycode::Return), .. } => self.return_text(),
            Event::KeyDown { keycode: Some(Keycode::Tab), .. } => self.tab_text(),
            Event::KeyDown { keycode: Some(Keycode::Up), .. } => self.cursor.shift_y(-1, &input, &self.text, &mut self.window),
            Event::KeyDown { keycode: Some(Keycode::Down), .. } => self.cursor.shift_y(1, &input, &self.text, &mut self.window),
            Event::KeyDown { keycode: Some(Keycode::Left), .. } => self.cursor.shift_x(-1, &input, &self.text, &mut self.window),
            Event::KeyDown { keycode: Some(Keycode::Right), .. } => self.cursor.shift_x(1, &input, &self.text, &mut self.window),
            Event::TextInput { text, .. } => self.insert_text(text),

            // Keyboard commands
            Event::KeyDown { keycode: Some(Keycode::A), .. }
            if input.keyboard.ctrl_down() => self.cursor.select_all(&self.text, &mut self.window),
            Event::KeyDown { keycode: Some(Keycode::C), .. }
            if input.keyboard.ctrl_down() => self.copy_selected_text()?,
            Event::KeyDown { keycode: Some(Keycode::X), .. }
            if input.keyboard.ctrl_down() => self.cut_text()?,
            Event::KeyDown { keycode: Some(Keycode::V), .. }
            if input.keyboard.ctrl_down() => self.paste_text()?,
            Event::KeyDown { keycode: Some(Keycode::Z), .. }
            if input.keyboard.ctrl_down() => self.undo_action(),
            Event::KeyDown { keycode: Some(Keycode::Y), .. }
            if input.keyboard.ctrl_down() => self.redo_action(),
            Event::KeyDown { keycode: Some(Keycode::D), .. }
            if input.keyboard.ctrl_down() => self.cursor.select_around_cursor(&self.text, &mut self.window),

            // Mouse Events
            Event::MouseWheel { y, .. } => self.scroll(y),
            Event::MouseMotion { x, y, .. } => self.move_mouse(x, y, &input),
            Event::MouseButtonDown { clicks, x, y, .. } => self.left_click(x, y, clicks),
            
            _ => {},
        }

        Ok(())
    }

    pub fn activate(&mut self) {
        self.active = true;
    }

    pub fn deactivate(&mut self) {
        self.active = false;
    }

    pub fn update(&mut self) {
        if self.active {
            self.cursor.update(&mut self.window);
        }
    }

    pub fn should_render(&mut self) -> bool {
        self.window.check_render()
    }

    pub fn resize(&mut self, pos: Vector2D, width: i32, height: i32) {
        self.window.resize(pos, width, height);
    }

    pub fn draw(&mut self, canvas: &mut Canvas<Window>, texture_creator: &TextureCreator<WindowContext>) -> Result<(), Box<dyn Error>> {
        canvas.set_draw_color(self.background_color);
        let (x, y) = self.window.get_pos().into();
        let (screen_w, screen_h) = self.window.get_window_dim();
        canvas.fill_rect(FRect::new(x as f32, y as f32, screen_w as f32, screen_h as f32))?;


        let (text_padding, line_padding) = self.window.get_padding();
        let pos = self.window.pos();
        let mut start_y = text_padding + pos.y;
        let (_, height) = self.window.get_text_dim();
        let height = height as u32;

        for (line_num, line_text) in self.text.lines().enumerate().skip(self.window.get_first_line()).take(self.window.lines()) {
            let focused_text = line_text.chars().skip(self.window.get_first_char()).take(self.window.chars()).collect::<String>();
            draw::selection_box(
                canvas,
                &self.cursor,
                &self.window,
                line_num,
                focused_text.chars().count(),
                self.font_select_color,
            )?;

            let text_to_render = if focused_text.len() != 0 {
                focused_text
            } else {
                String::from(" ")
            };

            let surface = self
                .font
                .render(&text_to_render)
                .blended(self.font_color)
                .map_err(|err| format!("On line: {:?}: {}", text_to_render, err))?;
            let texture = texture_creator
                .create_texture_from_surface(&surface)?;

            let TextureQuery {width, .. } = texture.query();

            let target = draw::text_target_aligned(
                &TextAlignment::LEFT,
                text_padding,
                pos.x,
                start_y,
                width,
                height,
                screen_w
            );
            canvas.copy(&texture, None, Some(target.into()))?;

            start_y += height + line_padding;
        }
        self.cursor.draw(self.active, canvas, &self.window)?;

        Ok(())
    }

    pub fn set_text(&mut self, text_data: String) {
        let old_text = std::mem::take(&mut self.text);
        let total_len = old_text.len();
        let jump_pos = Vector2D::new(0, 0);
        self.text = old_text.replace(0, total_len, text_data, jump_pos, &mut self.cursor, &mut self.window);
    }

    pub fn export(&self) -> String {
        self.text.chars().collect()
    }

    pub fn extract_text(&mut self) -> String {
        let contents = self.export();
        let len = self.text.len();
        let old_text = std::mem::take(&mut self.text);
        self.text = old_text.remove(0, len, &mut self.cursor, &mut self.window);
        contents
    }

    pub fn click_in_window(&self, x: f32, y: f32) -> bool {
        self.window.is_in_screen_bound(x.ceil() as u32, y.ceil() as u32)
    }

    pub fn height_of_one_line(&self) -> u32 {
        let (_, text_height) = self.window.get_text_dim();
        let (text_padding, line_padding) = self.window.get_padding();
        text_height as u32 + line_padding + text_padding * 2
    }

    pub fn enlarge_text(&mut self) -> Result<(), Box<dyn Error>> {
        self.font_size = (self.font_size + FONT_ZOOM_INCREMENT).min(MAX_FONT_SIZE);
        self.font = load_font(&self.ttf_context, DEFAULT_FONT_PATH, self.font_size, DEFAULT_FONT_STYLE)?;
        let (text_width, text_height) = self.font.size_of_char('|')?;
        self.window.resize_text(text_width, text_height);
        Ok(())
    }

    pub fn shrink_text(&mut self) -> Result<(), Box<dyn Error>> {
        self.font_size = (self.font_size - FONT_ZOOM_INCREMENT).max(MIN_FONT_SIZE);
        self.font = load_font(&self.ttf_context, DEFAULT_FONT_PATH, self.font_size, DEFAULT_FONT_STYLE)?;
        let (text_width, text_height) = self.font.size_of_char('|')?;
        self.window.resize_text(text_width, text_height);
        Ok(())
    }
}

impl <'a> TextBox<'a> {
    fn replace_selected_text(&mut self, select_pos: Vector2D, replace_text: String) {
        let cursor_pos = self.cursor.pos();
        let select_start = calculate_index_from_pos(&mut self.text, select_pos);
        let current_index = calculate_index_from_pos(&mut self.text, cursor_pos);
        let (index, jump_pos) = if select_start <= current_index {
            (select_start, select_pos)
        } else {
            (current_index, cursor_pos)
        };
        let replace_len = select_start.abs_diff(current_index);

        let old_text = std::mem::take(&mut self.text);
        self.text = old_text.replace(index, replace_len, replace_text, jump_pos, &mut self.cursor, &mut self.window);
    }

    fn insert_text(&mut self, text_chunk: String) {
        if let Some(select_pos) = self.cursor.select_start_pos() {
            return self.replace_selected_text(select_pos, text_chunk);
        }

        let index = calculate_index_from_pos(&self.text, self.cursor.pos());
        let old_text = std::mem::take(&mut self.text);
        self.text = old_text.insert(index, text_chunk, &mut self.cursor, &mut self.window);
    }

    fn delete_text(&mut self) {
        if let Some(select_pos) =  self.cursor.select_start_pos() {
            return self.replace_selected_text(select_pos, String::from(""));
        }

        let Vector2D {x, y} = self.cursor.pos();
        let line_index = self.text.get_line_index(y as usize);
        let index = line_index + x as usize;
        if index == self.text.len() {
            return;
        }

        let old_text = std::mem::take(&mut self.text);
        self.text = old_text.delete(index, 1, &mut self.cursor, &mut self.window);
    }

    fn remove_text(&mut self, amt: usize) {
        if let Some(select_pos) =  self.cursor.select_start_pos() {
            return self.replace_selected_text(select_pos, String::from(""));
        }

        let index = calculate_index_from_pos(&self.text, self.cursor.pos());
        let Some(shift_index) = index.checked_sub(amt) else {
            return;
        };

        let old_text = std::mem::take(&mut self.text);
        self.text = old_text.remove(shift_index, amt, &mut self.cursor, &mut self.window);
    }

    fn return_text(&mut self) {
        if let Some(select_pos) = self.cursor.select_start_pos() {
            return self.replace_selected_text(select_pos, String::from("\n"));
        }
        let index = calculate_index_from_pos(&self.text, self.cursor.pos());

        let old_text = std::mem::take(&mut self.text);
        self.text = old_text.push_and_insert(index, String::from("\n"), &mut self.cursor, &mut self.window);
    }

    fn tab_text(&mut self) {
        let pos @ Vector2D {x, ..}: Vector2D = self.cursor.pos();
        let index = calculate_index_from_pos(&self.text, pos);
        let spaces = TAB_SPACE_COUNT - x % TAB_SPACE_COUNT;
        let insert_spaces = String::from(&TAB_SPACE_STRING[..spaces as usize]);

        if let Some(select_pos) =  self.cursor.select_start_pos() {
            return self.replace_selected_text(select_pos, insert_spaces);
        }

        let old_text = std::mem::take(&mut self.text);
        self.text = old_text.push_and_insert(index, insert_spaces, &mut self.cursor, &mut self.window);
    }

    fn copy_selected_text(&self) -> Result<(), Box<dyn Error>> {
        let selected_text = self.get_selected_text();
        self.video_subsystem.clipboard().set_clipboard_text(&selected_text)?;
        Ok(())
    }

    fn paste_text(&mut self) -> Result<(), Box<dyn Error>> {
        let clipboard_text = self.video_subsystem.clipboard().clipboard_text()?;
        let normalized_clipboard_text = clipboard_text.replace("\r\n", "\n");
        if let Some(select_pos) =  self.cursor.select_start_pos() {
            self.replace_selected_text(select_pos, normalized_clipboard_text);
            return Ok(());
        }

        let index = calculate_index_from_pos(&self.text, self.cursor.pos());
        let old_text = std::mem::take(&mut self.text);
        self.text = old_text.push_and_insert(index, normalized_clipboard_text, &mut self.cursor, &mut self.window);

        Ok(())
    }

    fn cut_text(&mut self) -> Result<(), Box<dyn Error>> {
        let selected_text = self.get_selected_text();
        self.video_subsystem.clipboard().set_clipboard_text(&selected_text)?;
        if let Some(select_pos) = self.cursor.select_start_pos() {
            self.replace_selected_text(select_pos, String::from(""));
        }

        Ok(())
    }

    fn get_selected_text(&self) -> String {
        let Some(select_pos) = self.cursor.select_start_pos() else {
            return String::new();
        };
        let cursor_pos = self.cursor.pos();
        let select_start = calculate_index_from_pos(&self.text, select_pos);
        let current_index = calculate_index_from_pos(&self.text, cursor_pos);
        let len = select_start.abs_diff(current_index);
        let index = select_start.min(current_index);

        self.text.chars().skip(index).take(len).collect()
    }

    fn undo_action(&mut self) {
        let old_text = std::mem::take(&mut self.text);
        self.text = old_text.undo(&mut self.cursor, &mut self.window);
    }

    fn redo_action(&mut self) {
        let old_text = std::mem::take(&mut self.text);
        self.text = old_text.redo(&mut self.cursor, &mut self.window);
    }

    fn scroll(&mut self, amt: f32) {
        if amt > 0.0 {
            self.window.scroll_up(amt as usize);
        } else {
            let max_lines = self.text.line_count();
            self.window.scroll_down((-amt) as usize, max_lines);
        }
    }

    fn move_mouse(&mut self, mouse_x: f32, mouse_y: f32, input: &InputState) {
        self.cursor.mouse_move(mouse_x, mouse_y, input, &mut self.text, &mut self.window);
    }

    fn left_click(&mut self, click_x: f32, click_y: f32, clicks: u8) {
        self.cursor.left_click_press(click_x, click_y, clicks, &self.text, &mut self.window);
    }
}

fn calculate_index_from_pos(text: &TextRope, pos: Vector2D) -> usize {
    let Vector2D {x, y} = pos;
    let line_index = text.get_line_index(y as usize);
    line_index + x as usize
}

fn load_font<'a>(ttf_context: &Sdl3TtfContext, font_path: &str, point_size: f32, style: FontStyle) -> Result<Font<'a>, Box<dyn Error>> {
    let mut font = ttf_context.load_font(font_path, point_size)?;
    font.set_style(style);
    Ok(font)
}