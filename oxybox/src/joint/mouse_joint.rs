use std::ops::Deref;

use glam::Vec2;

use crate::Joint;

/// A live mouse joint in a [`World`](crate::World), borrowed from it for reading and writing.
///
/// A mouse joint drags a point on a body towards a world-space target, using a soft constraint
/// with a maximum force. This allows the constraint to stretch without applying huge forces.
///
/// Get one from [`World::create_mouse_joint`](crate::World::create_mouse_joint), or from
/// [`Joint::as_mouse`]. This also has access to all the functions on [`Joint`] via deref.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct MouseJoint<'a>(pub(crate) Joint<'a>);

impl<'a> MouseJoint<'a> {
    /// Get the target point in world space.
    pub fn target(&self) -> Vec2 {
        unsafe { sys::b2MouseJoint_GetTarget(self.raw()).into() }
    }

    /// Set the target point in world space.
    pub fn set_target(&self, target: Vec2) {
        unsafe { sys::b2MouseJoint_SetTarget(self.raw(), target.into()) }
    }

    /// Get the spring stiffness in hertz.
    pub fn spring_hertz(&self) -> f32 {
        unsafe { sys::b2MouseJoint_GetSpringHertz(self.raw()) }
    }

    /// Set the spring stiffness in hertz.
    pub fn set_spring_hertz(&self, hertz: f32) {
        unsafe { sys::b2MouseJoint_SetSpringHertz(self.raw(), hertz) }
    }

    /// Get the spring damping ratio, non-dimensional.
    pub fn spring_damping_ratio(&self) -> f32 {
        unsafe { sys::b2MouseJoint_GetSpringDampingRatio(self.raw()) }
    }

    /// Set the spring damping ratio, non-dimensional.
    pub fn set_spring_damping_ratio(&self, damping_ratio: f32) {
        unsafe { sys::b2MouseJoint_SetSpringDampingRatio(self.raw(), damping_ratio) }
    }

    /// Get the maximum force, typically in newtons.
    pub fn max_force(&self) -> f32 {
        unsafe { sys::b2MouseJoint_GetMaxForce(self.raw()) }
    }

    /// Set the maximum force, typically in newtons.
    pub fn set_max_force(&self, max_force: f32) {
        unsafe { sys::b2MouseJoint_SetMaxForce(self.raw(), max_force) }
    }
}

impl<'a> Deref for MouseJoint<'a> {
    type Target = Joint<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Debug for MouseJoint<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id().fmt(f)
    }
}
