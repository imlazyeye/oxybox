use glam::Vec2;

use crate::BodyId;
use crate::mirrors_layout;
use crate::opaque::Internal;

/// A mouse joint definition holds all the data needed to construct a
/// [`MouseJoint`](crate::MouseJoint).
///
/// ```
/// # use glam::Vec2;
/// # use oxybox::{BodyDefinition, BodyKind, MouseJointDefinition, World};
/// # let world = World::default();
/// let ground = world.create_body(BodyDefinition::new());
/// let ball = world.create_body(BodyDefinition {
///     kind: BodyKind::Dynamic,
///     ..BodyDefinition::new()
/// });
///
/// let joint = world
///     .create_mouse_joint(MouseJointDefinition {
///         body_id_a: ground.id(),
///         body_id_b: ball.id(),
///         target: Vec2::new(5.0, 0.0),
///         ..MouseJointDefinition::new()
///     })
///     .unwrap();
/// ```
///
/// You can safely re-use joint definitions. Joint definitions are temporary objects used to bundle
/// creation parameters.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MouseJointDefinition {
    /// The first attached body. This is assumed to be static.
    pub body_id_a: BodyId,

    /// The second attached body -- the one being dragged.
    pub body_id_b: BodyId,

    /// The initial target point in world space.
    pub target: Vec2,

    /// Stiffness in hertz.
    pub hertz: f32,

    /// Damping ratio, non-dimensional.
    pub damping_ratio: f32,

    /// Maximum force, typically in newtons.
    pub max_force: f32,

    /// Set this flag to true if the attached bodies should collide.
    pub collide_connected: bool,

    /// Use this to store application specific joint data.
    ///
    /// Box2D stores this as a pointer-sized value, so this is a `usize` rather than a `u64`.
    pub user_data: usize,

    /// Box2D's own validity cookie. See [`Internal`].
    pub internal: Internal,
}

mirrors_layout! {
    MouseJointDefinition => sys::b2MouseJointDef {
        body_id_a => bodyIdA,
        body_id_b => bodyIdB,
        target => target,
        hertz => hertz,
        damping_ratio => dampingRatio,
        max_force => maxForce,
        collide_connected => collideConnected,
        user_data => userData,
        internal => internalValue,
    }
}

impl MouseJointDefinition {
    /// Creates a new MouseJointDefinition filled in with the Box2D defaults.
    ///
    /// Both body ids default to null, so they must be filled in before the definition is used.
    pub fn new() -> Self {
        // safety: `MouseJointDefinition` and `b2MouseJointDef` have the same layout, which the
        // assertions above check field by field at compile time.
        unsafe { std::mem::transmute(sys::b2DefaultMouseJointDef()) }
    }

    /// This definition as the Box2D struct it is already laid out as. Borrowing rather than
    /// converting is the reason for the layout assertions above.
    pub(crate) fn as_b2(&self) -> *const sys::b2MouseJointDef {
        (self as *const MouseJointDefinition).cast()
    }
}

impl Default for MouseJointDefinition {
    fn default() -> Self {
        Self::new()
    }
}
