mod joint_id;
mod joint_kind;
mod mouse_joint;
mod mouse_joint_definition;

use std::{ffi::c_void, marker::PhantomData, ops::Deref};

use glam::Vec2;
pub use joint_id::JointId;
pub use joint_kind::JointKind;
pub use mouse_joint::MouseJoint;
pub use mouse_joint_definition::MouseJointDefinition;

use crate::{BodyId, BodyRef, World};

/// A live joint in a [`World`], borrowed from it for reading only.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct JointRef<'a>(pub(crate) sys::b2JointId, PhantomData<&'a World>);

impl<'a> JointRef<'a> {
    /// Creates a new [`JointRef`] out of an existing *valid* [`JointId`].
    pub(crate) fn new(joint_id: JointId) -> Self {
        Self(joint_id.0, PhantomData)
    }

    /// The raw Box2D id backing this handle.
    pub(crate) fn raw(self) -> sys::b2JointId {
        self.0
    }

    /// The [`JointId`] naming this joint.
    pub fn id(&self) -> JointId {
        JointId::from_b2(self.0)
    }

    /// Get the joint kind.
    pub fn kind(&self) -> JointKind {
        let b2joint_type = unsafe { sys::b2Joint_GetType(self.0) };

        match b2joint_type {
            sys::b2JointType_b2_distanceJoint => JointKind::Distance,
            sys::b2JointType_b2_filterJoint => JointKind::Filter,
            sys::b2JointType_b2_motorJoint => JointKind::Motor,
            sys::b2JointType_b2_mouseJoint => JointKind::Mouse,
            sys::b2JointType_b2_prismaticJoint => JointKind::Prismatic,
            sys::b2JointType_b2_revoluteJoint => JointKind::Revolute,
            sys::b2JointType_b2_weldJoint => JointKind::Weld,
            sys::b2JointType_b2_wheelJoint => JointKind::Wheel,
            _ => unreachable!("Box2D returned unknown JointKind"),
        }
    }

    /// The [`BodyId`] of the first body this joint is attached to.
    pub fn body_id_a(&self) -> BodyId {
        BodyId::from_b2(unsafe { sys::b2Joint_GetBodyA(self.0) })
    }

    /// The [`BodyId`] of the second body this joint is attached to.
    pub fn body_id_b(&self) -> BodyId {
        BodyId::from_b2(unsafe { sys::b2Joint_GetBodyB(self.0) })
    }

    /// The first body this joint is attached to, for reading.
    pub fn body_a(&self) -> BodyRef<'a> {
        // safety: a joint's bodies live in the same world as the joint, and destroying either
        // body destroys the joint, which needs the `&mut World` this handle's borrow rules out.
        BodyRef::new(self.body_id_a())
    }

    /// The second body this joint is attached to, for reading.
    pub fn body_b(&self) -> BodyRef<'a> {
        // safety: see `body_a`.
        BodyRef::new(self.body_id_b())
    }

    /// Get the local anchor on body A.
    pub fn local_anchor_a(&self) -> Vec2 {
        unsafe { sys::b2Joint_GetLocalAnchorA(self.0).into() }
    }

    /// Get the local anchor on body B.
    pub fn local_anchor_b(&self) -> Vec2 {
        unsafe { sys::b2Joint_GetLocalAnchorB(self.0).into() }
    }

    /// Whether the two attached bodies may collide with each other.
    pub fn collide_connected(&self) -> bool {
        unsafe { sys::b2Joint_GetCollideConnected(self.0) }
    }

    /// Get the user data stored in a joint, if any. By default, all user data has `0` stored
    /// within it.
    pub fn user_data(&self) -> usize {
        unsafe { sys::b2Joint_GetUserData(self.0) as usize }
    }

    /// Get the current constraint force, usually in newtons.
    pub fn constraint_force(&self) -> Vec2 {
        unsafe { sys::b2Joint_GetConstraintForce(self.0).into() }
    }

    /// Get the current constraint torque, usually in newton-meters.
    pub fn constraint_torque(&self) -> f32 {
        unsafe { sys::b2Joint_GetConstraintTorque(self.0) }
    }
}

impl std::fmt::Debug for JointRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id().fmt(f)
    }
}

/// A live joint in a [`World`], borrowed from it for reading and writing.
///
/// Get one with [`World::joint`]. This also has access to all the functions on [`JointRef`] via
/// deref. Functions specific to one kind of joint live on that kind's handle -- see
/// [`Joint::as_mouse`].
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Joint<'a>(pub(crate) JointRef<'a>);

impl<'a> Joint<'a> {
    /// Creates a new [`Joint`] out of an existing *valid* [`JointId`].
    pub(crate) fn new(joint_id: JointId) -> Self {
        Self(JointRef::new(joint_id))
    }

    /// Sets arbitrary user data on the joint.
    pub fn set_user_data(&self, data: usize) {
        unsafe { sys::b2Joint_SetUserData(self.raw(), data as *mut c_void) }
    }

    /// Set whether the two attached bodies may collide with each other.
    pub fn set_collide_connected(&self, should_collide: bool) {
        unsafe { sys::b2Joint_SetCollideConnected(self.raw(), should_collide) }
    }

    /// Wake the bodies connected to this joint.
    pub fn wake_bodies(&self) {
        unsafe { sys::b2Joint_WakeBodies(self.raw()) }
    }

    /// This joint as a [`MouseJoint`], or `None` if it is some other kind of joint.
    pub fn as_mouse(self) -> Option<MouseJoint<'a>> {
        // Box2D only checks the joint kind in a debug assertion, so the typed handle is what
        // keeps a mouse joint function from being run on some other kind of joint.
        (self.kind() == JointKind::Mouse).then_some(MouseJoint(self))
    }
}

impl<'a> Deref for Joint<'a> {
    type Target = JointRef<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Debug for Joint<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id().fmt(f)
    }
}

/// A joint definition named a body that could not be joined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum InvalidJointBody {
    /// Body A has been destroyed, or belongs to a different world.
    #[error("body A is destroyed or belongs to a different world")]
    BodyA,

    /// Body B has been destroyed, or belongs to a different world.
    #[error("body B is destroyed or belongs to a different world")]
    BodyB,
}

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use crate::*;

    fn anchor_and_ball(world: &World) -> (BodyId, BodyId) {
        let anchor = world.create_body(BodyDefinition::new());
        let ball = world.create_body(BodyDefinition {
            kind: BodyKind::Dynamic,
            ..BodyDefinition::new()
        });
        ball.attach_circle(Vec2::ZERO, 1.0, ShapeDefinition::new());

        (anchor.id(), ball.id())
    }

    fn zero_gravity() -> World {
        World::new(WorldDefinition {
            gravity: Vec2::ZERO,
            ..WorldDefinition::new()
        })
    }

    #[test]
    fn a_mouse_joint_drags_its_body() {
        const TARGET: Vec2 = Vec2::new(10.0, 0.0);

        let mut world = zero_gravity();
        let (anchor, ball) = anchor_and_ball(&world);

        let joint = world
            .create_mouse_joint(MouseJointDefinition {
                body_id_a: anchor,
                body_id_b: ball,
                target: Vec2::ZERO,
                max_force: 1000.0,
                ..MouseJointDefinition::new()
            })
            .unwrap();

        // the joint is created at the ball, then the target moves away from it
        joint.set_target(TARGET);

        for _ in 0..120 {
            world.step(1.0 / 60.0, World::SUB_STEPS);
        }

        let position = world.body(ball).unwrap().position();
        assert!(
            position.distance(TARGET) < 1.0,
            "the ball was not dragged to the target: {position}"
        );
    }

    #[test]
    fn mouse_joint_getters_return_what_was_set() {
        let world = zero_gravity();
        let (anchor, ball) = anchor_and_ball(&world);

        let definition = MouseJointDefinition {
            body_id_a: anchor,
            body_id_b: ball,
            target: Vec2::new(1.0, 2.0),
            hertz: 3.0,
            damping_ratio: 0.5,
            max_force: 42.0,
            user_data: 0xBEEF,
            ..MouseJointDefinition::new()
        };

        let joint = world.create_mouse_joint(definition).unwrap();

        assert_eq!(joint.kind(), JointKind::Mouse);
        assert_eq!(joint.body_id_a(), anchor);
        assert_eq!(joint.body_id_b(), ball);
        assert_eq!(joint.target(), definition.target);
        assert_eq!(joint.spring_hertz(), definition.hertz);
        assert_eq!(joint.spring_damping_ratio(), definition.damping_ratio);
        assert_eq!(joint.max_force(), definition.max_force);
        assert_eq!(joint.user_data(), definition.user_data);

        // setting props does stuff:
        joint.set_spring_hertz(5.0);
        joint.set_spring_damping_ratio(0.25);
        joint.set_max_force(7.0);
        joint.set_user_data(0xCAFE);

        assert_eq!(joint.spring_hertz(), 5.0);
        assert_eq!(joint.spring_damping_ratio(), 0.25);
        assert_eq!(joint.max_force(), 7.0);
        assert_eq!(joint.user_data(), 0xCAFE);
    }

    #[test]
    fn default_joints_are_rejected() {
        let world = zero_gravity();
        let err = world.create_mouse_joint(MouseJointDefinition::new()).unwrap_err();
        assert_eq!(err, InvalidJointBody::BodyA);
    }

    #[test]
    fn bad_joint_bodies_are_rejected() {
        let mut world = zero_gravity();
        let other = zero_gravity();
        let (anchor, ball) = anchor_and_ball(&world);
        let (_, foreign_ball) = anchor_and_ball(&other);

        // a perfectly valid body -- just not one in this world
        let err = world
            .create_mouse_joint(MouseJointDefinition {
                body_id_a: anchor,
                body_id_b: foreign_ball,
                ..MouseJointDefinition::new()
            })
            .unwrap_err();
        assert_eq!(err, InvalidJointBody::BodyB);

        // a body that no longer exists
        assert!(world.destroy_body(ball));
        let err = world
            .create_mouse_joint(MouseJointDefinition {
                body_id_a: anchor,
                body_id_b: ball,
                ..MouseJointDefinition::new()
            })
            .unwrap_err();
        assert_eq!(err, InvalidJointBody::BodyB);
    }

    #[test]
    fn joint_handles_follow_the_joint() {
        let mut world = zero_gravity();
        let other = zero_gravity();
        let (anchor, ball) = anchor_and_ball(&world);
        let definition = MouseJointDefinition {
            body_id_a: anchor,
            body_id_b: ball,
            ..MouseJointDefinition::new()
        };

        let joint_id = world.create_mouse_joint(definition).unwrap().id();

        // the id comes back out of the world, and remembers what kind of joint it is
        let joint = world.joint(joint_id).expect("the joint was just created");
        assert_eq!(joint.as_mouse().map(|j| j.id()), Some(joint_id));
        assert!(other.joint(joint_id).is_none(), "a foreign joint id was accepted");
        assert!(!other.owns_joint(joint_id));

        // destroying the joint invalidates it, and a second destroy is a no-op
        assert!(world.destroy_joint(joint_id));
        assert!(world.joint(joint_id).is_none());
        assert!(!world.destroy_joint(joint_id));

        // destroying a body takes its joints with it
        let joint_id = world.create_mouse_joint(definition).unwrap().id();
        assert!(world.destroy_body(ball));
        assert!(world.joint(joint_id).is_none());
    }
}
