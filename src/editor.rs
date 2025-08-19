mod draw;
mod cursor;
pub mod rope;

use cursor::Cursor;
use std::{error::Error, ffi::CString, str::FromStr};

use sdl3::{event::{Event, WindowEvent}, keyboard::Keycode, pixels::Color,
    rect::Rect, render::{Canvas, FPoint, TextureCreator, TextureQuery},
    sys::{clipboard::SDL_SetClipboardText, events::SDL_WindowEvent, keyboard::{SDL_GetModState, SDL_StartTextInput, SDL_StopTextInput}, keycode::SDL_KMOD_CTRL}, ttf::{Font, FontStyle, Sdl3TtfContext},
    video::{Window, WindowContext}, EventPump, Sdl, VideoSubsystem};

use crate::vector::Vector2D;

const DEFAULT_FONT_PATH: &str = "C:\\Windows\\Fonts\\consola.ttf";
const DEFAULT_FONT_SIZE: f32 = 32.0;
const DEFAULT_FONT_STYLE: FontStyle = FontStyle::NORMAL;
const DEFAULT_BACKGROUND_COLOR: Color = Color::BLACK;
const DEFAULT_FONT_COLOR: Color = Color::WHITE;
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
    text: String,
    backgroud_color: Color,
    font_color: Color,
    text_padding: u32,
    line_padding: u32,
    alignment: TextAlignment,
    cursor: Cursor,

    // Data
    font: Font<'a>,

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
            text: String::new(),
            font: default_font,
            text_padding: DEFAULT_TEXT_PADDING,
            line_padding: DEFAULT_LINE_PADDING,
            alignment: TextAlignment::LEFT,
            cursor: Cursor::new(DEFAULT_FONT_SIZE)
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
                Event::Window { win_event: WindowEvent::Resized(..), .. } | Event::Window { win_event: WindowEvent::PixelSizeChanged(..), ..} => {
                    self.render_text = true;
                }
                Event::KeyDown { keycode: Some(Keycode::Backspace), .. } => {
                    self.text.pop();
                    self.render_text = true;
                },
                Event::KeyDown { keycode: Some(Keycode::Return), .. } => {
                    self.cursor.move_y(1);
                    self.text.push('\n');
                    self.render_text = true;
                }
                Event::KeyDown { keycode: Some(Keycode::C), ..}
                if unsafe {SDL_GetModState()} & SDL_KMOD_CTRL > 0 => {
                    let raw_text = CString::from_str(&self.text)?;
                    unsafe {SDL_SetClipboardText(raw_text.as_ptr()); }
                }
                Event::KeyDown { keycode: Some(Keycode::V), ..}
                if unsafe { SDL_GetModState() } & SDL_KMOD_CTRL > 0 => {
                    let clipboard_text = self.context.video_subsystem.clipboard().clipboard_text()?;
                    self.text.push_str(clipboard_text.as_str());
                    self.render_text = true;
                },
                Event::TextInput { text, .. } => {
                    self.text.push_str(text.as_str());
                    self.render_text = true;
                },
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

        for line_text in self.text.lines() {
            let trimmed_text = line_text.trim();
            let text_to_render = if trimmed_text.len() == 0 { " " } else { trimmed_text };

            let surface = self
                .font
                .render(text_to_render)
                .blended(self.font_color)
                .map_err(|err| format!("On line: {:?}: {}", text_to_render, err))?;
            let texture = self
                .context.texture_creater
                .create_texture_from_surface(&surface)?;

            let TextureQuery {width, height, .. } = texture.query();

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
    fn load_font(&mut self, font_path: &str, point_size: f32, style: FontStyle) -> Result<(), Box<dyn Error>> {
        self.font = self
            .context.ttf_context
            .load_font(font_path, point_size)?;
        self.font.set_style(style);
        Ok(())
    }
}