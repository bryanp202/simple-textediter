extern crate sdl3;

mod editor;
pub mod vector;
use std::process;

use sdl3::ttf;

use crate::editor::Editor;
use crate::editor::rope::Rope;


pub fn main() {
    unsafe { std::env::set_var("RUST_BACKTRACE", "1"); }
    let start = std::time::Instant::now();
    let mut rope = Rope::new();
    rope = rope.insert(0, "HELLO AND WELCOME TO THE SHOW MY FRIEND");

    for i in 0..30_000 {
        let rope_len = rope.len();
        rope = rope.insert(rope_len, format!("What is this: {}\n\r", i).as_str());
    }
    
    let dt = start.elapsed();
    println!("Time to run: {:?}, average: {}", dt, dt.as_nanos()as f64/1_000_000.0);//\nRope:\n{:?}", dt, rope.chars().collect::<String>());
    //println!("{:?}", rope.chars().collect::<String>());
    let skip_start = std::time::Instant::now();
    let skipped = rope.lines().skip(29_999).next();
    let time_to_skip = skip_start.elapsed();
    println!("Skipped: {:?}, Time to skip: {:?}", skipped, time_to_skip);
    println!("Total lines: {}", rope.line_count());

    //run();    
}

pub fn run() {
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
        state.update();
    }

    state.close()
}