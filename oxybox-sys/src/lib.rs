#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
mod bindings;
pub use bindings::*;
mod utils;

// glam and b2Vec2 are the same thing (two f32s):
const _: () = {
    use std::mem;

    assert!(mem::size_of::<glam::Vec2>() == mem::size_of::<b2Vec2>());
    assert!(mem::align_of::<glam::Vec2>() == mem::align_of::<b2Vec2>());
    assert!(mem::offset_of!(glam::Vec2, x) == mem::offset_of!(b2Vec2, x));
    assert!(mem::offset_of!(glam::Vec2, y) == mem::offset_of!(b2Vec2, y));
};
