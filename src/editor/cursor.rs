use sdl3::{pixels::Color, render::{Canvas, FPoint}, video::Window};

use crate::vector::Vector2D;
use std::{error::Error, time::{Duration, Instant}};

const DEFAULT_BLINK_PERIOD: Duration = Duration::from_millis(500);
const DEFAULT_CUSROR_COLOR: Color = Color::WHITE;

pub struct Cursor {
    pos: Vector2D,
    size: f32,
    blink_on: bool,
    blink_period: Duration,
    blink_timer: Instant,
    color: Color,
}

impl Cursor {
    pub fn new(size: f32) -> Self {
        Self{
            size,
            ..Default::default()
        }
    }

    pub fn pos(&self) -> Vector2D {
        self.pos
    }

    pub fn draw_context(&self) -> (f32, Vector2D) {
        (self.size, self.pos)
    }

    pub fn move_x(&mut self, amt: u32) {
        self.move_to(self.pos.x.saturating_add(amt), self.pos.y);
    }

    pub fn move_y(&mut self, amt: u32) {
        self.move_to(self.pos.x, self.pos.y.saturating_add(amt));
    }

    pub fn move_to(&mut self, x: u32, y: u32) {
        self.pos.x = x;
        self.pos.y = y;
        self.blink_timer = Instant::now();
        self.blink_on = true;
    }

    pub fn draw(&mut self, canvas: &mut Canvas<Window>, text_pad: u32, line_pad: u32) -> Result<(), Box<dyn Error>> {
        if self.blink_on {
            canvas.set_draw_color(self.color);
            let text_pad = text_pad as f32;
            let line_pad = line_pad as f32;
            let line_height = self.size*4.0/3.0;

            let x =  self.pos.x as f32 + text_pad;
            let y = self.pos.y as f32 * (line_height + line_pad) + line_pad;

            let start = FPoint::new(x, y);
            let end = FPoint::new(x, y + line_height);
            println!("start {:?}, end {:?}", start, end);
            canvas.draw_line(start, end)?;
        }

        Ok(())
    }

    pub fn update(&mut self) -> bool {
        if self.blink_timer.elapsed() > self.blink_period {
            self.blink_on = !self.blink_on;
            self.blink_timer = Instant::now();
            true
        } else {
            false
        }
    }
}

impl Default for Cursor {
    fn default() -> Self {
        Self {
            size: 12.0,
            pos: Vector2D {
                x: 0,
                y: 0,
            },
            blink_on: false,
            blink_period: DEFAULT_BLINK_PERIOD,
            color: DEFAULT_CUSROR_COLOR,
            blink_timer: Instant::now(),
        }
    }
}