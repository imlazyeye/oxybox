use crate::mirrors_layout;

/// Limits which shapes collide with this one.
///
/// Two shapes collide when each one's category is in the other's mask. See
/// [`QueryFilter`](crate::QueryFilter) for the equivalent used by world queries.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Filter {
    /// The collision category bits. Normally you would just set one bit as a bitflag.
    ///
    /// The category bits should represent your application object types.
    /// 
    /// The default value is `1`.
    pub category_bits: u64,

    /// The collision mask bits. This states the categories that this shape would accept for
    /// collision.
    ///
    /// For example, you may want your player to only collide with static objects and other players.
    /// 
    /// The default value is `u64::MAX`.
    pub mask_bits: u64,

    /// Collision groups allow a certain group of objects to never collide (negative) or always
    /// collide (positive). A group index of zero has no effect.
    ///
    /// **Non-zero group filtering always wins against the mask bits.** For example, you may want
    /// ragdolls to collide with other ragdolls but not with themselves. In that case you would give
    /// each ragdoll a unique negative group index and apply it to all of that ragdoll's shapes.
    pub group_index: i32,
}

mirrors_layout! {
    Filter => sys::b2Filter {
        category_bits => categoryBits,
        mask_bits => maskBits,
        group_index => groupIndex,
    }
}

impl Filter {
    /// Creates a new Filter filled in with the Box2D defaults
    pub fn new() -> Self {
        // safety: `Filter` and `b2Filter` have the same layout, which the assertions above check
        // field by field at compile time.
        unsafe { std::mem::transmute(sys::b2DefaultFilter()) }
    }
}

impl Default for Filter {
    fn default() -> Self {
        Self::new()
    }
}
