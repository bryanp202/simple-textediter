extern crate sdl3;

mod editor;
use std::process;

use sdl3::ttf;

use crate::editor::Editor;

pub fn main() {
    const INIT_WINDOW_WIDTH: u32 = 800;
    const INIT_WINDOW_HEIGHT: u32 = 600;
    const WINDOW_NAME: &str = "Text Editor";

    let sdl_context = sdl3::init().unwrap_or_else(|err| {
        eprintln!("Failed to initialize SDL3: {err}");
        process::exit(1);
    });
    let video_subsytem = sdl_context.video().unwrap_or_else(|err| {
        eprintln!("Failed to open video subsystem: {err}");
        process::exit(1);
    });
    let ttf_context = ttf::init().unwrap_or_else(|err| {
        eprintln!("Failed to intialize TTF: {err}");
        process::exit(1);
    });

    let window = video_subsytem
        .window(WINDOW_NAME, INIT_WINDOW_WIDTH, INIT_WINDOW_HEIGHT)
        .position_centered()
        .resizable()
        .opengl()
        .build()
        .unwrap_or_else(|err| {
            eprintln!("Failed to create window \"{WINDOW_NAME}\": {}", err.to_string());
            process::exit(1);
        });

    let events = sdl_context.event_pump().unwrap_or_else(|err| {
        eprintln!("Failed to create event pump: {}", err.to_string());
        process::exit(1);
    });

    let mut state = Editor::build(sdl_context, video_subsytem, ttf_context, events, window).unwrap_or_else(|err| {
        eprintln!("Failed to create event pump: {}", err.to_string());
        process::exit(1);
    });

    while !state.should_quit() {
        state.handle_input().unwrap_or_else(|err| {
            eprintln!("Failed to handle event: {}", err.to_string());
            process::exit(1);
        });
        state.render().unwrap_or_else(|err| {
            eprintln!("Failed to render: {}", err.to_string());
            process::exit(1);
        });
    }

    state.close()
}