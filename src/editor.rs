mod draw;
mod cursor;
mod windowstate;
pub mod rope;

use windowstate::WindowState;
use cursor::Cursor;
use std::{error::Error, ffi::CString, str::FromStr};

use sdl3::{event::{Event, WindowEvent}, keyboard::Keycode, pixels::Color,
    rect::Rect, render::{Canvas, FPoint, TextureCreator, TextureQuery},
    sys::{clipboard::SDL_SetClipboardText, events::SDL_WindowEvent, keyboard::{SDL_GetModState, SDL_StartTextInput, SDL_StopTextInput}, keycode::SDL_KMOD_CTRL}, ttf::{Font, FontStyle, Sdl3TtfContext},
    video::{Window, WindowContext}, EventPump, Sdl, VideoSubsystem};

use crate::{editor::rope::TextRope, vector::Vector2D};

const DEFAULT_FONT_PATH: &str = "C:\\Windows\\Fonts\\consola.ttf";
const DEFAULT_FONT_SIZE: f32 = 32.0;
const DEFAULT_FONT_STYLE: FontStyle = FontStyle::NORMAL;
const DEFAULT_BACKGROUND_COLOR: Color = Color::RGB(20, 20, 20);
const DEFAULT_FONT_COLOR: Color = Color::RGB(180, 225, 225);
const DEFAULT_TEXT_PADDING: u32 = 16;
const DEFAULT_LINE_PADDING: u32 = 2;

pub enum TextAlignment {
    LEFT,
    RIGHT,
    CENTER,
}

pub struct Editor <'a> {
    // State
    quit: bool,
    render_text: bool,
    text: TextRope,
    backgroud_color: Color,
    font_color: Color,
    text_padding: u32,
    line_padding: u32,
    alignment: TextAlignment,
    cursor: Cursor,

    // Data
    font: Font<'a>,
    window: WindowState,
    // Handlers
    context: EditorContext,
}

pub struct EditorContext {
    sdl_context: Sdl,
    video_subsystem: VideoSubsystem,
    ttf_context: Sdl3TtfContext,
    events: EventPump,
    canvas: Canvas<Window>,
    texture_creater: TextureCreator<WindowContext>,
}

impl <'a> Editor<'a> {
    pub fn build(sdl_context: Sdl, video_subsystem: VideoSubsystem, ttf_context: Sdl3TtfContext, events: EventPump, window: Window) -> Result<Self, Box<dyn Error>> {
        let mut default_font = ttf_context.load_font(DEFAULT_FONT_PATH, DEFAULT_FONT_SIZE)?;
        default_font.set_style(DEFAULT_FONT_STYLE);

        unsafe { SDL_StartTextInput(window.raw()); }

        let canvas = window.into_canvas();
        let texture_creater = canvas.texture_creator();
        let (window_width, window_height) = canvas.window().size();
        let (text_width, text_height) = default_font.size_of_char('|')?;

        let new_editor = Self {
            context : EditorContext { sdl_context,
                video_subsystem,
                ttf_context,
                events,
                canvas,
                texture_creater,
            },
            backgroud_color: DEFAULT_BACKGROUND_COLOR,
            font_color: DEFAULT_FONT_COLOR,
            render_text: false,
            quit: false,
            text: TextRope::new(),
            font: default_font,
            text_padding: DEFAULT_TEXT_PADDING,
            line_padding: DEFAULT_LINE_PADDING,
            alignment: TextAlignment::LEFT,
            cursor: Cursor::new(text_width, text_height),
            window: WindowState::new(window_width, window_height, text_width, text_height),
        };

        Ok(new_editor)
    }

    pub fn should_quit(&self) -> bool {
        self.quit
    }

    pub fn handle_input(&mut self) -> Result<(), Box<dyn Error>> {
        for event in self.context.events.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => self.quit = true,
                Event::Window { win_event: WindowEvent::Resized(w_w, w_h), .. }
                    | Event::Window { win_event: WindowEvent::PixelSizeChanged(w_w, w_h), ..} => {
                        let (text_width, text_height) = self.font.size_of_char('|')?;
                    self.window.resize(w_w as u32, w_h as u32, text_width, text_height);
                    self.render_text = true;
                }
                Event::KeyDown { keycode: Some(Keycode::Delete), .. } => Self::delete_text(
                    &mut self.text,
                    &mut self.cursor,
                    &mut self.render_text
                ),
                Event::KeyDown { keycode: Some(Keycode::Backspace), .. } => Self::remove_text(
                    &mut self.text,
                    &mut self.cursor,
                    &mut self.render_text,
                    1
                ),
                Event::KeyDown { keycode: Some(Keycode::Return), .. } => Self::return_text(
                    &mut self.text,
                    &mut self.cursor,
                    &mut self.render_text
                ),
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    self.cursor.shift_y(-1, &self.text);
                    self.render_text = true;
                },
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    self.cursor.shift_y(1, &self.text);
                    self.render_text = true;
                },
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    self.cursor.shift_x(-1, &self.text);
                    self.render_text = true;
                },
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    self.cursor.shift_x(1, &self.text);
                    self.render_text = true;
                },
                Event::KeyDown { keycode: Some(Keycode::C), ..}
                if unsafe {SDL_GetModState()} & SDL_KMOD_CTRL > 0 => {
                    let raw_text = CString::from_str(self.text.chars().collect::<String>().as_str())?;
                    unsafe {SDL_SetClipboardText(raw_text.as_ptr()); }
                }
                Event::KeyDown { keycode: Some(Keycode::V), ..}
                if unsafe { SDL_GetModState() } & SDL_KMOD_CTRL > 0 => {
                    let clipboard_text = self.context.video_subsystem.clipboard().clipboard_text()?;
                    Self::insert_text(&mut self.text, &mut self.cursor, &mut self.render_text, &clipboard_text);
                },
                Event::TextInput { text: input_text, .. } => Self::insert_text(
                    &mut self.text,
                    &mut self.cursor,
                    &mut self.render_text,
                    &input_text
                ),
                _ => {},
            }
        }

        Ok(())
    }

    pub fn render(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.render_text {
            return Ok(());
        }
        self.render_text = false;

        self.context.canvas.set_draw_color(self.backgroud_color);
        self.context.canvas.clear();

        let mut start_y = self.text_padding;
        let (screen_w, _) = self.context.canvas.window().size();

        for line_text in self.text.lines().skip(self.window.get_first_line()).take(self.window.lines()) {
            let trimmed_text = line_text.trim();
            let text_to_render = if trimmed_text.len() != 0 {
                trimmed_text.chars().take(self.window.chars()).collect::<String>()
            } else {
                String::from(" ")
            };

            let surface = self
                .font
                .render(&text_to_render)
                .blended(self.font_color)
                .map_err(|err| format!("On line: {:?}: {}", text_to_render, err))?;
            let texture = self
                .context.texture_creater
                .create_texture_from_surface(&surface)?;

            let TextureQuery {width, .. } = texture.query();
            let (_, height) = self.font.size_of_char('|')?;

            let target = draw::text_target_aligned(&self.alignment, self.text_padding, start_y, width, height, screen_w);
            self.context.canvas.copy(&texture, None, Some(target.into()))?;

            start_y += height + self.line_padding;
        }

        self.cursor.draw(&mut self.context.canvas, self.text_padding, self.line_padding)?;

        self.context.canvas.present();

        Ok(())
    }

    pub fn update(&mut self) {
        if self.cursor.update() {
            self.render_text = true;
        }
    }

    pub fn close(self) {
        unsafe {SDL_StopTextInput(self.context.canvas.window().raw()); }
    }
}

impl <'a> Editor<'a> {
    fn delete_text(text: &mut TextRope, cursor: &mut Cursor, render_text: &mut bool) {
        let Vector2D {x, y} = cursor.pos();
        let line_index = text.get_line_index(y as usize);
        let index = line_index + x as usize;
        if index == text.len() {
            return;
        }

        let old_text = std::mem::take(text);
        *text = old_text.remove(index, 1);
        *render_text = true;
    }

    fn remove_text(text: &mut TextRope, cursor: &mut Cursor, render_text: &mut bool, amt: usize) {
        let Vector2D {x, y} = cursor.pos();
        let line_index = text.get_line_index(y as usize);
        let Some(index) = (line_index + x as usize).checked_sub(amt) else {
            return;
        };
        cursor.shift_x(-(amt as isize), text);

        let old_text = std::mem::take(text);
        *text = old_text.remove(index, amt);
        *render_text = true;
    }

    fn insert_text(text: &mut TextRope, cursor: &mut Cursor, render_text: &mut bool, text_chunk: &str) {
        let Vector2D {x, y} = cursor.pos();
        let line_index = text.get_line_index(y as usize);
        let index = line_index + x as usize;

        let old_text = std::mem::take(text);
        *text = old_text.insert(index, text_chunk);
        let text_len = text_chunk.chars().count() as isize;
        cursor.shift_x(text_len, text);
        *render_text = true;
    }

    fn return_text(text: &mut TextRope, cursor: &mut Cursor, render_text: &mut bool) {
        let Vector2D {x, y} = cursor.pos();
        let line_index = text.get_line_index(y as usize);
        let index = line_index + x as usize;

        let old_text = std::mem::take(text);
        *text = old_text.insert(index, "\n");
        cursor.ret();
        *render_text = true;
    }
}

impl <'a> Editor<'a> {
    fn load_font(&mut self, font_path: &str, point_size: f32, style: FontStyle) -> Result<(), Box<dyn Error>> {
        self.font = self
            .context.ttf_context
            .load_font(font_path, point_size)?;
        self.font.set_style(style);
        Ok(())
    }
}