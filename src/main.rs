#![windows_subsystem = "windows"]
extern crate sdl3;

mod editor;
pub mod vector;
use std::process;
use std::time::{Duration, Instant};

use sdl3::sys::video::SDL_SetWindowMinimumSize;
use sdl3::ttf;
use crate::editor::Editor;

pub fn main() {
    //unsafe { std::env::set_var("RUST_BACKTRACE", "1"); }
    let mut args = std::env::args();
    let starting_file = args.nth(1);
    run(starting_file);
}

pub fn run(starting_file: Option<String>) {
    const FRAME_RATE: u64 = 120;
    const FRAME_DELTA: Duration = Duration::from_nanos(1_000_000_000 / FRAME_RATE);
    const INIT_WINDOW_WIDTH: u32 = 800;
    const INIT_WINDOW_HEIGHT: u32 = 600;
    const MIN_WINDOW_WIDTH: i32 = 400;
    const MIN_WINDOW_HEIGHT: i32 = 300;
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
    unsafe { SDL_SetWindowMinimumSize(window.raw(), MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT); }

    let events = sdl_context.event_pump().unwrap_or_else(|err| {
        eprintln!("Failed to create event pump: {}", err.to_string());
        process::exit(1);
    });

    let mut state = Editor::build(sdl_context, video_subsytem, ttf_context, events, window).unwrap_or_else(|err| {
        eprintln!("Failed to create editor state: {}", err.to_string());
        process::exit(1);
    });

    if let Some(starting_file) = starting_file {
        state.open_file(starting_file);
    }

    while !state.should_quit() {
        let frame_start = Instant::now();
        state.handle_input().unwrap_or_else(|err| {
            eprintln!("Failed to handle event: {}", err.to_string());
            process::exit(1);
        });
        state.render().unwrap_or_else(|err| {
            eprintln!("Failed to render: {}", err.to_string());
            process::exit(1);
        });
        state.update();
        std::thread::sleep(FRAME_DELTA.saturating_sub(frame_start.elapsed()));
    }

    state.close()
}