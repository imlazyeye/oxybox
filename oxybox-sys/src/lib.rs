#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod bindings;
pub use bindings::*;
mod utils;

impl PartialEq for b2BodyId {
    fn eq(&self, other: &Self) -> bool {
        self.index1 == other.index1 && self.world0 == other.world0 && self.generation == other.generation
    }
}
impl Eq for b2BodyId {}

impl PartialEq for b2ShapeId {
    fn eq(&self, other: &Self) -> bool {
        self.index1 == other.index1 && self.world0 == other.world0 && self.generation == other.generation
    }
}
impl Eq for b2ShapeId {}

impl PartialEq for b2JointId {
    fn eq(&self, other: &Self) -> bool {
        self.index1 == other.index1 && self.world0 == other.world0 && self.generation == other.generation
    }
}
impl Eq for b2JointId {}

impl PartialEq for b2WorldId {
    fn eq(&self, other: &Self) -> bool {
        self.generation == other.generation && self.index1 == other.index1
    }
}
impl Eq for b2WorldId {}

impl std::hash::Hash for b2WorldId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index1.hash(state);
        self.generation.hash(state);
    }
}

impl std::hash::Hash for b2BodyId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index1.hash(state);
        self.world0.hash(state);
        self.generation.hash(state);
    }
}

impl std::hash::Hash for b2ShapeId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index1.hash(state);
        self.world0.hash(state);
        self.generation.hash(state);
    }
}

impl std::hash::Hash for b2JointId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.index1.hash(state);
        self.world0.hash(state);
        self.generation.hash(state);
    }
}

// glam and b2Vec2 are the same thing (two f32s):
const _: () = {
    use std::mem;

    assert!(mem::size_of::<glam::Vec2>() == mem::size_of::<b2Vec2>());
    assert!(mem::align_of::<glam::Vec2>() == mem::align_of::<b2Vec2>());
    assert!(mem::offset_of!(glam::Vec2, x) == mem::offset_of!(b2Vec2, x));
    assert!(mem::offset_of!(glam::Vec2, y) == mem::offset_of!(b2Vec2, y));
};
