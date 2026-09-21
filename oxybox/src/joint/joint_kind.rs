/// The kind of a joint. All joints share [`JointId`](crate::JointId), so this is how you tell them
/// apart.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u32)]
pub enum JointKind {
    /// Connects two bodies at a fixed distance, optionally as a spring.
    Distance = sys::b2JointType_b2_distanceJoint,

    /// Disables collision between two specific bodies.
    Filter = sys::b2JointType_b2_filterJoint,

    /// Drives the relative transform between two bodies.
    Motor = sys::b2JointType_b2_motorJoint,

    /// Drags a body towards a world-space target. See [`MouseJoint`](crate::MouseJoint).
    Mouse = sys::b2JointType_b2_mouseJoint,

    /// Allows relative translation along one axis.
    Prismatic = sys::b2JointType_b2_prismaticJoint,

    /// Allows relative rotation about a shared point.
    Revolute = sys::b2JointType_b2_revoluteJoint,

    /// Glues two bodies together.
    Weld = sys::b2JointType_b2_weldJoint,

    /// A suspension spring with a rotating wheel.
    Wheel = sys::b2JointType_b2_wheelJoint,
}
