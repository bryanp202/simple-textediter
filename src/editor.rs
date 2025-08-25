mod draw;
mod cursor;
mod windowstate;
pub mod rope;

use windowstate::WindowState;
use cursor::Cursor;
use std::{error::Error, ffi::CString, path::PathBuf, str::FromStr, sync::{Arc, Mutex}};

use sdl3::{dialog::{show_open_file_dialog, show_save_file_dialog, DialogFileFilter}, event::{Event, WindowEvent}, keyboard::Keycode, mouse::{MouseButton}, pixels::Color, render::{Canvas, TextureCreator, TextureQuery}, sys::{clipboard::SDL_SetClipboardText, keyboard::{SDL_GetModState, SDL_StartTextInput, SDL_StopTextInput}, keycode::{SDL_KMOD_CTRL, SDL_KMOD_NUM}}, ttf::{Font, FontStyle, Sdl3TtfContext}, video::{Window, WindowContext}, EventPump, VideoSubsystem};

use crate::{editor::rope::TextRope, vector::Vector2D};

const DEFAULT_FONT_PATH: &str = "C:\\Windows\\Fonts\\consola.ttf";
const DEFAULT_FONT_SIZE: f32 = 56.0;
const DEFAULT_FONT_STYLE: FontStyle = FontStyle::NORMAL;
const DEFAULT_BACKGROUND_COLOR: Color = Color::RGB(20, 20, 20);
const DEFAULT_FONT_COLOR: Color = Color::RGB(180, 225, 225);
const DEFAULT_FONT_SELECT_COLOR: Color = Color::RGB(80, 80, 80);
const DEFAULT_TEXT_PADDING: u32 = 16;
const DEFAULT_LINE_PADDING: u32 = 2;
const TAB_SPACE_COUNT: u32 = 2;
const TAB_SPACE_STRING: &str = "  ";

#[allow(dead_code)]
pub enum TextAlignment {
    LEFT,
    RIGHT,
    CENTER,
}

pub struct Editor <'a> {
    // State
    quit: bool,
    text: TextRope,
    backgroud_color: Color,
    font_color: Color,
    font_select_color: Color,
    alignment: TextAlignment,
    cursor: Cursor,

    // Data
    font: Font<'a>,
    window: WindowState,
    open_file_paths: Arc<Mutex<Vec<PathBuf>>>,
    save_file_paths: Arc<Mutex<Vec<PathBuf>>>,
    // Handlers
    context: EditorContext,
}

pub struct EditorContext {
    video_subsystem: VideoSubsystem,
    ttf_context: Sdl3TtfContext,
    events: EventPump,
    canvas: Canvas<Window>,
    texture_creater: TextureCreator<WindowContext>,
}

impl <'a> Editor<'a> {
    pub fn build(video_subsystem: VideoSubsystem, ttf_context: Sdl3TtfContext, events: EventPump, window: Window) -> Result<Self, Box<dyn Error>> {
        let mut default_font = ttf_context.load_font(DEFAULT_FONT_PATH, DEFAULT_FONT_SIZE)?;
        default_font.set_style(DEFAULT_FONT_STYLE);

        unsafe { SDL_StartTextInput(window.raw()); }

        let canvas = window.into_canvas();
        let texture_creater = canvas.texture_creator();
        let (window_width, window_height) = canvas.window().size();
        let (text_width, text_height) = default_font.size_of_char('|')?;

        let new_editor = Self {
            context : EditorContext {
                video_subsystem,
                ttf_context,
                events,
                canvas,
                texture_creater,
            },
            backgroud_color: DEFAULT_BACKGROUND_COLOR,
            font_color: DEFAULT_FONT_COLOR,
            font_select_color: DEFAULT_FONT_SELECT_COLOR,
            quit: false,
            text: TextRope::new(),
            font: default_font,
            alignment: TextAlignment::LEFT,
            cursor: Cursor::new(),
            window: WindowState::new(window_width, window_height, text_width, text_height, DEFAULT_TEXT_PADDING, DEFAULT_LINE_PADDING),
            open_file_paths: Arc::new(Mutex::new(Vec::new())),
            save_file_paths: Arc::new(Mutex::new(Vec::new())),
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
                Event::Window { win_event: WindowEvent::Resized(w_w, w_h), .. } |
                Event::Window { win_event: WindowEvent::PixelSizeChanged(w_w, w_h), ..} => {
                    let (text_width, text_height) = self.font.size_of_char('|')?;
                    self.window.resize(
                        w_w as u32, w_h as u32, text_width, text_height,
                    );
                }
                Event::KeyDown { keycode: Some(Keycode::Home), .. } => {
                    let y = if unsafe {SDL_GetModState()} & SDL_KMOD_CTRL > 0 {
                        0
                    } else {
                        self.cursor.pos().y
                    };
                    self.cursor.jump_to(0, y, &mut self.window, &self.text);
                },
                Event::KeyDown { keycode: Some(Keycode::Delete), .. } => Self::delete_text(
                    &mut self.text,
                    &mut self.cursor,
                    &mut self.window,
                ),
                Event::KeyDown { keycode: Some(Keycode::Backspace), .. } => Self::remove_text(
                    &mut self.text,
                    &mut self.cursor,
                    1,
                    &mut self.window,
                ),
                Event::KeyDown { keycode: Some(Keycode::Return), .. } | Event::KeyDown { keycode: Some(Keycode::KpEnter), .. } => Self::return_text(
                    &mut self.text,
                    &mut self.cursor,
                    &mut self.window,
                ),
                Event::KeyDown { keycode: Some(Keycode::Tab), .. } => Self::tab_text(
                    &mut self.text,
                    &mut self.cursor,
                    &mut self.window,
                ),
                Event::KeyDown { keycode: Some(Keycode::Up), .. } => {
                    self.cursor.shift_y(-1, &self.text, &mut self.window);
                },
                Event::KeyDown { keycode: Some(Keycode::Kp8), .. }
                if unsafe { SDL_GetModState() } & SDL_KMOD_NUM == 0 => {
                    self.cursor.shift_y(-1, &self.text, &mut self.window);
                },
                Event::KeyDown { keycode: Some(Keycode::Down), .. } => {
                    self.cursor.shift_y(1, &self.text, &mut self.window);
                },
                Event::KeyDown { keycode: Some(Keycode::Kp2), .. }
                if unsafe { SDL_GetModState() } & SDL_KMOD_NUM == 0 => {
                    self.cursor.shift_y(1, &self.text, &mut self.window);
                },
                Event::KeyDown { keycode: Some(Keycode::Left), .. } => {
                    self.cursor.shift_x(-1, &self.text, &mut self.window);
                },
                Event::KeyDown { keycode: Some(Keycode::Kp4), .. }
                if unsafe { SDL_GetModState() } & SDL_KMOD_NUM == 0 => {
                    self.cursor.shift_x(-1, &self.text, &mut self.window);
                },
                Event::KeyDown { keycode: Some(Keycode::Right), .. } => {
                    self.cursor.shift_x(1, &self.text, &mut self.window);
                },
                Event::KeyDown { keycode: Some(Keycode::Kp6), .. }
                if unsafe { SDL_GetModState() } & SDL_KMOD_NUM == 0 => {
                    self.cursor.shift_x(1, &self.text, &mut self.window);
                },
                Event::KeyDown { keycode: Some(Keycode::A), ..}
                if unsafe {SDL_GetModState()} & SDL_KMOD_CTRL > 0 => self.cursor.select_all(&mut self.window, &self.text),
                Event::KeyDown { keycode: Some(Keycode::C), ..}
                if unsafe {SDL_GetModState()} & SDL_KMOD_CTRL > 0 => {
                    let selected_text = Self::get_selected_text(&self.cursor, &self.text);
                    let raw_text = CString::from_str(&selected_text)?;
                    unsafe {SDL_SetClipboardText(raw_text.as_ptr()); }
                },
                Event::KeyDown { keycode: Some(Keycode::V), ..}
                if unsafe { SDL_GetModState() } & SDL_KMOD_CTRL > 0 => {
                    let clipboard_text = self.context.video_subsystem.clipboard().clipboard_text()?;
                    let normalized_clipboard_text = clipboard_text.replace("\r\n", "\n");
                    Self::insert_text(
                        &mut self.text,
                        &mut self.cursor,
                        &normalized_clipboard_text,
                        &mut self.window,
                    );
                },
                Event::KeyDown { keycode: Some(Keycode::O), .. }
                if unsafe { SDL_GetModState() } & SDL_KMOD_CTRL > 0 => {
                    let filters = [
                        DialogFileFilter {
                            name: "Text Document (*.txt)",
                            pattern: "txt",
                        },
                        DialogFileFilter {
                            name: "All Files (*.*)",
                            pattern: "*",
                        },
                    ];
                    let file_path_ref = self.open_file_paths.clone();
                    show_open_file_dialog(
                        &filters,
                        None::<PathBuf>,
                        true,
                        self.context.canvas.window(),
                        Box::new(move |result, _| {
                            let Ok(file_paths) = result else { return };
                            let mut open_file_paths = file_path_ref.lock().unwrap_or_else(|mut err| {
                                **err.get_mut() = vec![];
                                file_path_ref.clear_poison();
                                err.into_inner()
                            });
                            open_file_paths.extend_from_slice(&file_paths);
                        }),
                        ).map_err(|err| err.to_string())?;
                },
                Event::KeyDown { keycode: Some(Keycode::S), .. }
                if unsafe { SDL_GetModState() } & SDL_KMOD_CTRL > 0 => {
                    let filters = [
                        DialogFileFilter {
                            name: "Text Document (*.txt)",
                            pattern: "txt",
                        },
                        DialogFileFilter {
                            name: "All Files (*.*)",
                            pattern: "*",
                        },
                    ];
                    let file_path_ref = self.save_file_paths.clone();
                    show_save_file_dialog(
                        &filters,
                        None::<PathBuf>,
                        self.context.canvas.window(),
                        Box::new(move |result, _| {
                            let Ok(mut file_paths) = result else { return };
                            let mut open_file_paths = file_path_ref.lock().unwrap_or_else(|mut err| {
                                **err.get_mut() = vec![];
                                file_path_ref.clear_poison();
                                err.into_inner()
                            });
                            for file_path in file_paths.iter_mut() {
                                if file_path.extension() == None {
                                    file_path.set_extension("txt");
                                }
                            }
                            open_file_paths.extend_from_slice(&file_paths);
                        }),
                        ).map_err(|err| err.to_string())?;
                },
                Event::TextInput { text: input_text, .. } => Self::insert_text(
                    &mut self.text,
                    &mut self.cursor,
                    &input_text,
                    &mut self.window,
                ),
                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x: click_x, y: click_y, clicks, .. } => Self::left_click(
                    click_x,
                    click_y,
                    clicks,
                    &mut self.text,
                    &mut self.cursor,
                    &mut self.window,
                ),
                Event::MouseButtonUp { mouse_btn: MouseButton::Left, .. } => {
                    Self::release_left_click(&mut self.cursor);
                }
                Event::MouseWheel { y, .. } => {
                    if y > 0.0 {
                        self.window.scroll_up(y as usize);
                    } else {
                        let max_lines = self.text.line_count();
                        self.window.scroll_down((-y) as usize, max_lines);
                    }
                },
                Event::MouseMotion { x, y, .. } => Self::move_mouse(
                    x,
                    y,
                    &mut self.text,
                    &mut self.cursor,
                    &mut self.window,
                ),
                _ => {},
            }
        }

        Ok(())
    }

    pub fn render(&mut self) -> Result<(), Box<dyn Error>> {
        if !self.window.check_render() {
            return Ok(());
        }

        self.context.canvas.set_draw_color(self.backgroud_color);
        self.context.canvas.clear();


        let (text_padding, line_padding) = self.window.get_padding();
        let mut start_y = text_padding;
        let (screen_w, _) = self.context.canvas.window().size();

        for (line_num, line_text) in self.text.lines().enumerate().skip(self.window.get_first_line()).take(self.window.lines()) {
            let focused_text = line_text.chars().skip(self.window.get_first_char()).take(self.window.chars()).collect::<String>();
            draw::selection_box(
                &mut self.context.canvas,
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
            let texture = self
                .context.texture_creater
                .create_texture_from_surface(&surface)?;

            let TextureQuery {width, .. } = texture.query();
            let (_, height) = self.font.size_of_char('|')?;

            let target = draw::text_target_aligned(&self.alignment, text_padding, start_y, width, height, screen_w);
            self.context.canvas.copy(&texture, None, Some(target.into()))?;

            start_y += height + line_padding;
        }

        self.cursor.draw(&mut self.context.canvas, &self.window)?;

        self.context.canvas.present();

        Ok(())
    }

    pub fn update(&mut self) {
        if self.cursor.update() {
            self.window.set_render_flag();
        }

        self.check_open_files();
        self.check_save_files();
    }

    pub fn close(self) {
        unsafe {SDL_StopTextInput(self.context.canvas.window().raw()); }
    }
}

impl <'a> Editor<'a> {
    pub fn open_file(&mut self, file_path: String) {
        let data = std::fs::read_to_string(file_path).unwrap_or_else(|_| String::new());
        
        let normalized_data = data.replace("\r\n", "\n");
        self.text = TextRope::new().append(&normalized_data);
        self.cursor.jump_to(0, 0, &mut self.window, &self.text);
    }

    fn check_open_files(&mut self) {
        let mut open_file_paths = self.open_file_paths.lock().unwrap_or_else(|mut err| {
            **err.get_mut() = vec![];
            self.open_file_paths.clear_poison();
            err.into_inner()
        });
        while let Some(file_path) = open_file_paths.pop() {
            let data = std::fs::read_to_string(file_path).unwrap_or_else(|_| String::new());
            
            let normalized_data = data.replace("\r\n", "\n");
            self.text = TextRope::new().append(&normalized_data);
            self.cursor.jump_to(0, 0, &mut self.window, &self.text);
        }
    }

    fn check_save_files(&mut self) {
        let mut save_file_paths = self.save_file_paths.lock().unwrap_or_else(|mut err| {
            **err.get_mut() = vec![];
            self.open_file_paths.clear_poison();
            err.into_inner()
        });
        while let Some(file_path) = save_file_paths.pop() {
            _ = std::fs::write(file_path, Self::export(&self.text));
        }
    }

    fn export(text: &TextRope) -> String {
        let raw_text = text.chars().collect::<String>();
        let windows_text = raw_text.replace("\n", "\r\n");
        windows_text
    }

    fn replace_selected_text(
        text: &mut TextRope,
        cursor: &mut Cursor,
        window: &mut WindowState,
        select_pos: Vector2D,
        replace_text: &str,
    ) {
        let cursor_pos = cursor.pos();
        let select_start = Self::calculate_index_from_pos(text, select_pos);
        let current_index = Self::calculate_index_from_pos(text, cursor_pos);
        let (index, Vector2D{x: jump_x, y: jump_y}) = if select_start <= current_index {
            (select_start, select_pos)
        } else {
            (current_index, cursor_pos)
        };
        let len = select_start.abs_diff(current_index);

        let old_text = std::mem::take(text);
        *text = old_text.remove(index, len).insert(index, replace_text);
        cursor.jump_to(jump_x, jump_y, window, text);
        cursor.shift_x(replace_text.len() as isize, text, window);
    }

    fn delete_text(text: &mut TextRope, cursor: &mut Cursor, window: &mut WindowState) {
        if let Some(select_pos) =  cursor.select_start_pos() {
            return Self::replace_selected_text(text, cursor, window, select_pos, "");
        }

        let Vector2D {x, y} = cursor.pos();
        let line_index = text.get_line_index(y as usize);
        let index = line_index + x as usize;
        if index == text.len() {
            return;
        }

        let old_text = std::mem::take(text);
        *text = old_text.remove(index, 1);
        cursor.reset_blink();
        cursor.jump_to(x, y, window, text);
    }

    fn remove_text(text: &mut TextRope, cursor: &mut Cursor, amt: usize, window: &mut WindowState) {
        if let Some(select_pos) =  cursor.select_start_pos() {
            return Self::replace_selected_text(text, cursor, window, select_pos, "");
        }

        let index = Self::calculate_index_from_pos(text, cursor.pos());
        let Some(shift_index) = index.checked_sub(amt) else {
            return;
        };
        cursor.shift_x(-(amt as isize), text, window);

        let old_text = std::mem::take(text);
        *text = old_text.remove(shift_index, amt);
    }

    fn insert_text(text: &mut TextRope, cursor: &mut Cursor, text_chunk: &str, window: &mut WindowState) {
        if let Some(select_pos) =  cursor.select_start_pos() {
            return Self::replace_selected_text(text, cursor, window, select_pos, text_chunk);
        }

        let index = Self::calculate_index_from_pos(text, cursor.pos());
        let old_text = std::mem::take(text);
        *text = old_text.insert(index, text_chunk);

        let text_len = text_chunk.chars().filter(|&c| c != '\n').count() as isize;

        cursor.shift_x(text_len, text, window);
    }

    fn return_text(text: &mut TextRope, cursor: &mut Cursor, window: &mut WindowState) {
        if let Some(select_pos) =  cursor.select_start_pos() {
            return Self::replace_selected_text(text, cursor, window, select_pos, "\n");
        }
        let index = Self::calculate_index_from_pos(text, cursor.pos());

        let old_text = std::mem::take(text);
        *text = old_text.insert(index, "\n");
        cursor.ret(window, text);
    }

    fn tab_text(text: &mut TextRope, cursor: &mut Cursor, window: &mut WindowState) {
        let pos @ Vector2D {x, ..} = cursor.pos();
        let index = Self::calculate_index_from_pos(text, pos);
        let spaces = TAB_SPACE_COUNT - x % TAB_SPACE_COUNT;
        let insert_spaces = &TAB_SPACE_STRING[..spaces as usize];
        let old_text = std::mem::take(text);

        if let Some(select_pos) =  cursor.select_start_pos() {
            return Self::replace_selected_text(text, cursor, window, select_pos, insert_spaces);
        }
        
        *text = old_text.insert(index, insert_spaces);
        cursor.shift_x(spaces as isize, text, window);
    }

    fn left_click(
        click_x: f32,
        click_y: f32,
        clicks: u8,
        text: &mut TextRope,
        cursor: &mut Cursor,
        window: &mut WindowState,
    ) {
        cursor.left_click_press(click_x, click_y, clicks, text, window);
    }

    fn move_mouse(
        click_x: f32,
        click_y: f32,
        text: &mut TextRope,
        cursor: &mut Cursor,
        window: &mut WindowState,
    ) {
        cursor.mouse_move(click_x, click_y, text, window)
    }

    fn release_left_click(cursor: &mut Cursor) {
        cursor.left_click_release();
    }

    fn calculate_index_from_pos(text: &TextRope, pos: Vector2D) -> usize {
        let Vector2D {x, y} = pos;
        let line_index = text.get_line_index(y as usize);
        line_index + x as usize
    }

    fn get_selected_text(cursor: &Cursor, text: &TextRope) -> String {
        let Some(select_pos) = cursor.select_start_pos() else {
            return String::new();
        };
        let cursor_pos = cursor.pos();
        let select_start = Self::calculate_index_from_pos(text, select_pos);
        let current_index = Self::calculate_index_from_pos(text, cursor_pos);
        let len = select_start.abs_diff(current_index);
        let index = select_start.min(current_index);

        text.chars().skip(index).take(len).collect()
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