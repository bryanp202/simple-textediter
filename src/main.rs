extern crate sdl3;

mod editor;
pub mod vector;
use std::process;

use sdl3::ttf;

use crate::editor::Editor;
use crate::editor::rope::Rope;


pub fn main() {
    let start = std::time::Instant::now();
    let mut rope = Rope::new();
    // rope = rope.insert(0, "Hello!");
    // rope = rope.insert(2, "Coolio");
    // rope = rope.insert(6, "Wowza\n");
    // rope = rope.insert(4, "Kongo\n");
    // rope = rope.insert(10, "TESTING\n");
    rope = rope.insert(0, "0");
    rope = rope.insert(0, "1");
    rope = rope.insert(0, "2");
    rope = rope.insert(0, "3");
    rope = rope.insert(0, "4");
    rope = rope.insert(0, "5");
    rope = rope.insert(4, "6");
    rope = rope.insert(3, "7");
    rope = rope.insert(0, "8");
    rope = rope.insert(0, "9");
    rope = rope.insert(9, "X");
    rope = rope.insert(9, "X");
    rope = rope.insert(9, "8");
    rope = rope.insert(2, "Hello world and welcome to my home!");
    //rope = rope.insert(2, "Hello world and welcome to my home!");
    //rope = rope.insert(2, "Hello world and welcome to my home!");
    //rope = rope.insert(2, "Hello world and welcome to my home!");
    //rope = rope.insert(2, "Hello world and welcome to my home!");
    //rope = rope.insert(2, "Hello world and welcome to my home!");
    //rope = rope.insert(2, "Hello world and welcome to my home!");
    //rope = rope.insert(2, "Hello world and welcome to my home!");
    //rope = rope.insert(2, "Hello world and welcome to my home!");
    //rope = rope.insert(2, "Hello world and welcome to my home!");

    rope = rope.insert(2, "Hello world and welcome to my home!");
    for _ in 0..31_000_000 {
        rope = rope.insert(50, "abc");
    }
    
    let dt = start.elapsed();
    println!("Time to run: {:?}, average: {}", dt, dt.as_nanos()as f64/31_000_000.0);//\nRope:\n{:?}", dt, rope.chars().collect::<String>());
    //println!("{:?}", rope);

    let start = std::time::Instant::now();
    rope = rope.insert(50, "Hello world and welcome to my home!");
    println!("Time to insert single: {:?}", start.elapsed());
    println!("Height: {}, len: {}", rope.height(), rope.len());

    //println!("Rope: {:?}", rope);
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