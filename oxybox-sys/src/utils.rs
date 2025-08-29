use glam::Vec2;

use crate::b2Vec2;

impl From<Vec2> for b2Vec2 {
    fn from(value: Vec2) -> Self {
        b2Vec2 { x: value.x, y: value.y }
    }
}

impl From<b2Vec2> for Vec2 {
    fn from(value: b2Vec2) -> Self {
        Vec2::new(value.x, value.y)
    }
}
