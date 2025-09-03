mod draw;
mod cursor;
mod windowstate;
mod textrope;
mod inputstate;
mod textbox;
mod command;

use std::{error::Error, path::PathBuf, sync::{Arc, Mutex}};

use sdl3::{dialog::{show_open_file_dialog, show_save_file_dialog, DialogFileFilter}, event::{Event, WindowEvent}, get_error, keyboard::Keycode, mouse::MouseButton, pixels::Color, render::{Canvas, TextureCreator}, sys::{keyboard::{SDL_GetModState, SDL_StartTextInput, SDL_StopTextInput}, keycode::SDL_KMOD_CTRL}, ttf::Sdl3TtfContext, video::{Window, WindowContext}, EventPump, VideoSubsystem};

use crate::{editor::{command::CommandState, inputstate::InputState}, vector::Vector2D};
use crate::editor::textbox::TextBox;
use crate::editor::command::Command;

const DEFAULT_TEXT_POS: Vector2D = Vector2D { x: 0, y: 0};
const DEFAULT_CONSOLE_POS: Vector2D = Vector2D {x: 0, y: 500};

#[allow(dead_code)]
pub enum TextAlignment {
    LEFT,
    RIGHT,
    CENTER,
}

pub enum Component {
    TEXT, CONSOLE,
}

pub struct State<'a> {
    quit: bool,
    input: InputState,
    text: TextBox<'a>,
    console: TextBox<'a>,
    active_component: Component,
    command_state: CommandState,
    open_file_paths: Arc<Mutex<Vec<PathBuf>>>,
    save_file_paths: Arc<Mutex<Vec<PathBuf>>>,
}

impl <'a> State<'a> {
    fn switch_to_text(&mut self) {
        self.console.deactivate();
        self.text.activate();
        self.active_component = Component::TEXT;
    }

    fn switch_to_console(&mut self) {
        self.text.deactivate();
        self.console.activate();
        self.active_component = Component::CONSOLE;
    }
}

pub struct Editor<'a> {
    // State
    state: State<'a>,
    // Handles and such
    context: EditorContext<'a>,
}


#[allow(dead_code)]
pub struct EditorContext<'a> {
    video_subsystem: &'a VideoSubsystem,
    ttf_context: &'a Sdl3TtfContext,
    events: &'a mut EventPump,
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
}

impl <'a> Editor<'a> {
    pub fn build(video_subsystem: &'a VideoSubsystem, ttf_context: &'a Sdl3TtfContext, events: &'a mut EventPump, window: Window) -> Result<Self, Box<dyn Error>> {
        unsafe { SDL_StartTextInput(window.raw()); }

        let (window_width, window_height) = window.size();
        let canvas = window.into_canvas();
        let texture_creator = canvas.texture_creator();

        let mut new_editor = Self {
            context: EditorContext {
                video_subsystem,
                ttf_context,
                events,
                canvas,
                texture_creator,
            },
            state: State {
                quit: false,
                text: TextBox::build(
                    DEFAULT_TEXT_POS,
                    window_width,
                    window_height - 100,
                    None,
                    video_subsystem,
                    ttf_context
                )?,
                console: TextBox::build(
                    DEFAULT_CONSOLE_POS,
                    window_width,
                    window_height,
                    Some(Color::RGB(20, 20, 60)),
                    video_subsystem,
                    ttf_context
                )?,
                active_component: Component::TEXT,
                input: InputState::default(),
                command_state: CommandState::new(),
                open_file_paths: Arc::new(Mutex::new(Vec::new())),
                save_file_paths: Arc::new(Mutex::new(Vec::new())),
            },
        };
        new_editor.state.text.activate();

        Ok(new_editor)
    }

    pub fn should_quit(&self) -> bool {
        self.state.quit
    }

    pub fn handle_input(&mut self) -> Result<(), Box<dyn Error>> {
        for event in self.context.events.poll_iter() {
            match &event {
                // Window control
                Event::Quit { .. } => self.state.quit = true,
                Event::KeyUp { keycode: Some(Keycode::W), .. } if self.state.input.keyboard.ctrl_down() => self.state.quit = true,
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    match self.state.active_component {
                        Component::CONSOLE => self.state.switch_to_text(),
                        Component::TEXT => self.state.switch_to_console(),
                    }
                },
                Event::Window { win_event: WindowEvent::Resized(w_w, w_h), .. } |
                Event::Window { win_event: WindowEvent::PixelSizeChanged(w_w, w_h), ..} => {
                    Self::realign_textboxes(&mut self.state.text, &mut self.state.console, *w_w, *w_h);
                },

                // Keyboard state
                Event::KeyDown { keycode: Some(Keycode::LShift), .. } |
                Event::KeyDown { keycode: Some(Keycode::RShift), .. } => self.state.input.keyboard.press_shift(),
                Event::KeyUp { keycode: Some(Keycode::LShift), .. } |
                Event::KeyUp { keycode: Some(Keycode::RShift), .. } => self.state.input.keyboard.release_shift(),
                Event::KeyDown { keycode: Some(Keycode::LCtrl), .. } |
                Event::KeyDown { keycode: Some(Keycode::RCtrl), .. } => self.state.input.keyboard.press_ctrl(),
                Event::KeyUp { keycode: Some(Keycode::LCtrl), .. } |
                Event::KeyUp { keycode: Some(Keycode::RCtrl), .. } => self.state.input.keyboard.release_ctrl(),
                Event::KeyDown { keycode: Some(Keycode::Return), ..} => {
                    match self.state.active_component {
                        Component::CONSOLE => {
                            Self::handle_cmd(&mut self.state);
                            continue;
                        },
                        _ => {},
                    }
                },

                // Keyboard cmds
                Event::KeyDown { keycode: Some(Keycode::Equals), .. }
                if self.state.input.keyboard.ctrl_down() => {
                    self.state.text.enlarge_text()?;
                    self.state.console.enlarge_text()?;
                    let (w_w, w_h) = self.context.canvas.window().size();
                    Self::realign_textboxes(&mut self.state.text, &mut self.state.console, w_w as i32, w_h as i32);
                }
                Event::KeyDown { keycode: Some(Keycode::Minus), .. }
                if self.state.input.keyboard.ctrl_down() => {
                    self.state.text.shrink_text()?;
                    self.state.console.shrink_text()?;
                    let (w_w, w_h) = self.context.canvas.window().size();
                    Self::realign_textboxes(&mut self.state.text, &mut self.state.console, w_w as i32, w_h as i32);
                }

                // Mouse state
                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
                    self.state.input.mouse.press_left();
                    if self.state.text.click_in_window(*x, *y) {
                        self.state.switch_to_text();
                    } else {
                        self.state.switch_to_console();
                    }
                },
                Event::MouseButtonUp { mouse_btn: MouseButton::Left, .. } => self.state.input.mouse.release_left(),
                Event::MouseWheel { .. } => {
                    self.state.text.handle_input(event, &self.state.input)?;
                    continue;
                },
                Event::DropFile { filename, .. } => Self::open_file_from_path(&mut self.state.text, &filename),

                // File io
                Event::KeyDown { keycode: Some(Keycode::O), .. }
                if unsafe { SDL_GetModState() } & SDL_KMOD_CTRL > 0 => {
                    let filters = [
                        DialogFileFilter {
                            name: "All Files (*.*)",
                            pattern: "*",
                        },
                        DialogFileFilter {
                            name: "Text Document (*.txt)",
                            pattern: "txt",
                        },
                    ];
                    let file_path_ref = self.state.open_file_paths.clone();
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
                            name: "All Files (*.*)",
                            pattern: "*",
                        },
                        DialogFileFilter {
                            name: "Text Document (*.txt)",
                            pattern: "txt",
                        },
                    ];
                    let file_path_ref = self.state.save_file_paths.clone();
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

                _ => {},
            }
            match self.state.active_component {
                Component::TEXT => self.state.text.handle_input(event, &self.state.input)?,
                Component::CONSOLE => self.state.console.handle_input(event, &self.state.input)?,
            }
        }

        Ok(())
    }

    pub fn render(&mut self) -> Result<(), Box<dyn Error>> {
        if self.state.text.should_render() | self.state.console.should_render() {
            self.context.canvas.set_draw_color(Color::BLACK);
            self.context.canvas.clear();
            self.state.text.draw(&mut self.context.canvas, &self.context.texture_creator)?;
            self.state.console.draw(&mut self.context.canvas, &self.context.texture_creator)?;
            self.state.text.draw_console(&mut self.context.canvas, &self.context.texture_creator)?;
            if !self.context.canvas.present() {
                return Err(Box::new(get_error()));
            }
        };
        Ok(())
    }

    pub fn update(&mut self) {
        self.state.text.update();
        self.state.console.update();

        self.check_open_files();
        self.check_save_files();
    }

    pub fn close(self) {
        unsafe {SDL_StopTextInput(self.context.canvas.window().raw()); }
    }
}

impl <'a> Editor<'a> {
    pub fn open_file(&mut self, file_path: &str) {
        Self::open_file_from_path(&mut self.state.text, &file_path);
    }

    fn check_open_files(&mut self) {
        let mut open_file_paths = self.state.open_file_paths.lock().unwrap_or_else(|mut err| {
            **err.get_mut() = vec![];
            self.state.open_file_paths.clear_poison();
            err.into_inner()
        });
        while let Some(file_path) = open_file_paths.pop() {
            Self::open_file_from_path(&mut self.state.text, file_path.to_str().unwrap_or(""));
        }
    }

    fn check_save_files(&mut self) {
        let mut save_file_paths = self.state.save_file_paths.lock().unwrap_or_else(|mut err| {
            **err.get_mut() = vec![];
            self.state.open_file_paths.clear_poison();
            err.into_inner()
        });
        while let Some(file_path) = save_file_paths.pop() {
            let data = self.state.text.export();
            let normalized_data = data.replace("\n", "\r\n");
            _ = std::fs::write(file_path, normalized_data);
        }
    }
}

impl <'a> Editor<'a> {
    fn realign_textboxes(text: &mut TextBox, console: &mut TextBox, w_w: i32, w_h: i32) {
        let console_height = console.height_of_one_line() as i32;
        let text_height = w_h  - console_height - 10;
        text.resize(Vector2D::new(0, 0), w_w, text_height as i32);
        console.resize(Vector2D::new(0, text_height as u32 + 10), w_w, console_height as i32);
    }

    fn open_file_from_path(text: &mut TextBox, file_path: &str) {
        let data = std::fs::read_to_string(file_path).unwrap_or_else(|_| String::new());
        let normalized_data = data.replace("\r\n", "\n");
        text.set_text(normalized_data);
    }

    fn handle_cmd(state: &mut State) {
        let cmd_str = state.console.extract_text();
        let cmd = Command::new(cmd_str);
        let clone_cmd = cmd.clone();

        match cmd {
            Command::JUMP(..) => state.text.execute_cmd(cmd),
            Command::QUIT => state.quit = true,
            Command::OPEN(file_path) => {
                let mut open_file_paths = state.open_file_paths.lock().unwrap_or_else(|mut err| {
                    **err.get_mut() = vec![];
                    state.open_file_paths.clear_poison();
                    err.into_inner()
                });
                open_file_paths.push(file_path.clone());
            },
            Command::WRITE(file_path) => {
                let mut open_file_paths = state.save_file_paths.lock().unwrap_or_else(|mut err| {
                    **err.get_mut() = vec![];
                    state.save_file_paths.clear_poison();
                    err.into_inner()
                });
                open_file_paths.push(file_path.clone());
            },
            Command::RUN(program, args) => {
                let _ = std::process::Command::new(program)
                    .args(args).spawn();
            },
            Command::ERROR => {},
            _ => state.command_state.execute_cmd(&mut state.text, cmd),
        }
        state.command_state.set_prev(clone_cmd);
    }
}