#[derive(Clone, Copy)]
pub struct Vector2D {
    pub x: u32,
    pub y: u32,
}

impl Vector2D {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}