use std::{error::Error, ffi::CString, str::FromStr};

use sdl3::{event::{Event, WindowEvent}, keyboard::Keycode, pixels::Color,
    rect::Rect, render::{Canvas, TextureCreator, TextureQuery},
    sys::{clipboard::SDL_SetClipboardText, events::SDL_WindowEvent, keyboard::{SDL_GetModState, SDL_StartTextInput, SDL_StopTextInput}, keycode::SDL_KMOD_CTRL}, ttf::{Font, FontStyle, Sdl3TtfContext},
    video::{Window, WindowContext}, EventPump, Sdl, VideoSubsystem};

const DEFAULT_FONT_PATH: &str = "C:\\Windows\\Fonts\\NotoSansJP-VF.ttf"; //"C:\\Windows\\Fonts\\\"noto sans\".ttf";
const DEFAULT_FONT_SIZE: f32 = 32.0;
const DEFAULT_FONT_STYLE: FontStyle = FontStyle::NORMAL;
const DEFAULT_BACKGROUND_COLOR: Color = Color::BLACK;
const DEFAULT_FONT_COLOR: Color = Color::WHITE;
const DEFAULT_TEXT_PADDING: u32 = 32;

pub struct Editor <'a> {
    // State
    quit: bool,
    render_text: bool,
    text: String,
    backgroud_color: Color,
    font_color: Color,
    text_padding: u32,

    // Data
    font: Font<'a>,

    // Handlers
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
            sdl_context,
            video_subsystem,
            ttf_context,
            events,
            canvas,
            texture_creater,
            backgroud_color: DEFAULT_BACKGROUND_COLOR,
            font_color: DEFAULT_FONT_COLOR,
            render_text: false,
            quit: false,
            text: String::new(),
            font: default_font,
            text_padding: DEFAULT_TEXT_PADDING,
        };

        Ok(new_editor)
    }

    pub fn should_quit(&self) -> bool {
        self.quit
    }

    pub fn handle_input(&mut self) -> Result<(), Box<dyn Error>> {
        for event in self.events.poll_iter() {
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
                    let clipboard_text = self.video_subsystem.clipboard().clipboard_text()?;
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

        self.canvas.set_draw_color(self.backgroud_color);
        self.canvas.clear();

        for (line_number, line_text) in self.text.lines().enumerate() {
            let text_to_render = if line_text.len() == 0 { " " } else { line_text };

            let surface = self
                .font
                .render(text_to_render)
                .blended(self.font_color)?;
            let texture = self
                .texture_creater
                .create_texture_from_surface(&surface)?;

            let TextureQuery {width, height, .. } = texture.query();

            let target = self.text_rect_line(width, height, line_number);
            self.canvas.copy(&texture, None, Some(target.into()))?;
        }
        self.canvas.present();

        Ok(())
    }

    pub fn close(self) {
        unsafe {SDL_StopTextInput(self.canvas.window().raw()); }
    }
}

impl <'a> Editor<'a> {
    fn load_font(&mut self, font_path: &str, point_size: f32, style: FontStyle) -> Result<(), Box<dyn Error>> {
        self.font = self
            .ttf_context
            .load_font(font_path, point_size)?;
        self.font.set_style(style);
        Ok(())
    }

    fn scaled_centered_text_rect(&self, text_width: u32, text_height: u32) -> Rect {
        let (window_width, window_height) = self.canvas.window().size();
        let padded_width = window_width - self.text_padding;
        let padded_height = window_height - self.text_padding;

        let wr = text_width as f32 / padded_width as f32;
        let hr = text_height as f32 / padded_height as f32;

        let (w, h) = if wr > 1f32 || hr > 1f32 {
            if wr > hr {
                let h = (text_height as f32 / wr) as i32;
                (padded_width as i32, h)
            } else {
                let w = (text_width as f32 / hr) as i32;
                (w, padded_height as i32)
            }
        } else {
            (text_width as i32, text_height as i32)
        };

        let cx = (window_width as i32 - w) / 2;
        let cy = (window_height as i32 - h) / 2;
        Rect::new(cx as i32, cy as i32, w as u32, h as u32)
    }

    fn text_rect_line(&self, text_width: u32, text_height: u32, line_number: usize) -> Rect {
        let (window_width, window_height) = self.canvas.window().size();
        let padded_width = window_width - self.text_padding;
        let padded_height = window_height - self.text_padding;

        let wr = text_width as f32 / padded_width as f32;
        let hr = text_height as f32 / padded_height as f32;

        let (w, h) = if wr > 1f32 || hr > 1f32 {
            if wr > hr {
                let h = (text_height as f32 / wr) as i32;
                (padded_width as i32, h)
            } else {
                let w = (text_width as f32 / hr) as i32;
                (w, padded_height as i32)
            }
        } else {
            (text_width as i32, text_height as i32)
        };

        let line_number = line_number as i32;
        let cx = (window_width as i32 - w) / 2;
        let cy = (window_height as i32 - h) / 10 * line_number;
        Rect::new(cx as i32, cy as i32, w as u32, h as u32)
    }
}