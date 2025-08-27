use std::cmp::Ordering;

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Vector2D {
    pub x: u32,
    pub y: u32,
}

impl Vector2D {
    pub fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

impl Into<(u32, u32)> for Vector2D {
    /// Returns (x, y)
    fn into(self) -> (u32, u32) {
        (self.x, self.y)
    }
}

impl Ord for Vector2D {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.y.cmp(&other.y) {
            Ordering::Equal => self.x.cmp(&other.x),
            not_eq => not_eq,
        }
    }
}

impl Eq for Vector2D {}

impl PartialOrd for Vector2D {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}