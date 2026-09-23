mod body_definition;
mod body_id;
mod body_kind;

use std::{ffi::c_void, marker::PhantomData, ops::Deref};

pub use body_definition::{BodyDefinition, BodyName};
pub use body_id::*;
pub use body_kind::BodyKind;
use glam::Vec2;

use crate::{Rotation, Shape, ShapeDefinition, ShapeId, World};

/// A live body in a [`World`], borrowed from it for reading only.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct BodyRef<'a>(pub(crate) sys::b2BodyId, PhantomData<&'a World>);

impl<'a> BodyRef<'a> {
    pub(crate) fn new(body_id: BodyId) -> Self {
        Self(body_id.0, PhantomData)
    }

    /// The raw Box2D id backing this handle.
    pub(crate) fn raw(self) -> sys::b2BodyId {
        self.0
    }

    /// The [`BodyId`] naming this body.
    pub fn id(&self) -> BodyId {
        BodyId::from_b2(self.0)
    }

    /// Get the world position of a body. This is the location of the body origin.
    pub fn position(&self) -> Vec2 {
        unsafe { sys::b2Body_GetPosition(self.0).into() }
    }

    /// Get the body kind.
    pub fn kind(&self) -> BodyKind {
        let b2body_type = unsafe { sys::b2Body_GetType(self.0) };

        match b2body_type {
            sys::b2BodyType_b2_dynamicBody => BodyKind::Dynamic,
            sys::b2BodyType_b2_kinematicBody => BodyKind::Kinematic,
            sys::b2BodyType_b2_staticBody => BodyKind::Static,
            _ => unreachable!("Box2D returned unknown BodyKind"),
        }
    }

    /// Get the user data stored in a body, if any. By default, all user data has `0` stored
    /// within it.
    pub fn user_data(&self) -> usize {
        unsafe { sys::b2Body_GetUserData(self.0) as usize }
    }

    /// Get the world rotation of a body. See [`Rotation`].
    pub fn rotation(&self) -> Rotation {
        unsafe { sys::b2Body_GetRotation(self.0) }.into()
    }

    /// Get the linear velocity of a body’s center of mass. Usually in meters per second.
    pub fn linear_velocity(&self) -> Vec2 {
        unsafe { sys::b2Body_GetLinearVelocity(self.0).into() }
    }

    /// Get the mass of the body, usually in kilograms.
    pub fn mass(&self) -> f32 {
        unsafe { sys::b2Body_GetMass(self.0) }
    }

    /// Returns true if this body is awake.
    pub fn awake(&self) -> bool {
        unsafe { sys::b2Body_IsAwake(self.0) }
    }
}

impl std::fmt::Debug for BodyRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id().fmt(f)
    }
}

/// A live body in a [`World`], borrowed from it for reading and writing.
///
/// Get one with [`World::body`]. This also has access to all the functions
/// on [`BodyRef`] via deref.
#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Body<'a>(pub(crate) BodyRef<'a>);

impl<'a> Body<'a> {
    pub(crate) fn new(body_id: BodyId) -> Self {
        Self(BodyRef::new(body_id))
    }

    /// Sets arbitrary user data on the body.
    pub fn set_user_data(&self, data: usize) {
        unsafe {
            sys::b2Body_SetUserData(self.raw(), data as *mut c_void);
        }
    }

    /// Set the linear velocity of a body. Usually in meters per second.
    pub fn set_linear_velocity(&self, linear_velocity: Vec2) {
        unsafe {
            sys::b2Body_SetLinearVelocity(self.raw(), linear_velocity.into());
        }
    }

    /// Set the world transform of a body. This acts as a teleport and is fairly expensive.
    /// Generally you should create a body with the intended transform.
    pub fn set_transform(&self, position: Vec2, rotation: Rotation) {
        unsafe {
            sys::b2Body_SetTransform(self.raw(), position.into(), rotation.into());
        }
    }

    /// Apply an impulse to the center of mass. This immediately modifies the velocity.
    /// The impulse is ignored if the body is not awake.
    ///
    /// `impulse` is the world impulse vector, usually in Ns or kgm/s.
    /// `wake` will also wake up the body.
    pub fn apply_impulse(&self, impulse: Vec2, wake: bool) {
        unsafe { sys::b2Body_ApplyLinearImpulseToCenter(self.raw(), impulse.into(), wake) }
    }

    /// Apply an impulse at a point. This immediately modifies the velocity.
    /// It also modifies the angular velocity if the point of application is not at the center of mass.
    ///
    /// `impulse` is the world impulse vector, usually in Ns or kgm/s.
    /// `point` is the world position of the point of application.
    /// `wake` will also wake up the body.
    pub fn apply_impulse_at(&self, impulse: Vec2, point: Vec2, wake: bool) {
        unsafe { sys::b2Body_ApplyLinearImpulse(self.raw(), impulse.into(), point.into(), wake) }
    }

    /// Apply an angular impulse. The impulse is ignored if the body is not awake.
    ///
    /// `impulse` is the angular impulse, usually in units of kgmm/s.
    /// `wake` will also wake up the body.
    pub fn apply_angular_impulse(&self, impulse: f32, wake: bool) {
        unsafe { sys::b2Body_ApplyAngularImpulse(self.raw(), impulse, wake) }
    }
    /// Wake a body from sleep. This wakes the entire island the body is touching.
    ///
    /// **Warning:** Putting a body to sleep will put the entire island of bodies touching this body to sleep,
    /// which can be expensive and possibly unintuitive.
    pub fn set_awake(&self, awake: bool) {
        unsafe {
            sys::b2Body_SetAwake(self.raw(), awake);
        }
    }

    /// Attaches a circle to the body.
    ///
    /// The `center` is the local offset from the body, and the `radius` is the radius of the circle.
    pub fn attach_circle(&self, center: Vec2, radius: f32, shape_def: ShapeDefinition) -> Shape<'a> {
        let shape_id = unsafe {
            sys::b2CreateCircleShape(
                self.raw(),
                shape_def.as_b2(),
                &sys::b2Circle {
                    center: center.into(),
                    radius,
                },
            )
        };

        Shape::new(ShapeId::from_b2(shape_id))
    }

    /// Attaches a rectangle to the body.
    ///
    /// Make a box (rectangle) polygon, bypassing the need for a convex hull.
    /// `half_dims` are the half dimensions of the rectangle, `offset` is the offset relative to the body,
    /// and `rotation` is how far the rectangle is turned within the body.
    pub fn attach_rectangle(
        &self,
        half_dims: Vec2,
        offset: Vec2,
        rotation: Rotation,
        shape_def: ShapeDefinition,
    ) -> Shape<'a> {
        let shape_id = unsafe {
            sys::b2CreatePolygonShape(
                self.raw(),
                shape_def.as_b2(),
                &sys::b2MakeOffsetBox(half_dims.x, half_dims.y, offset.into(), rotation.into()),
            )
        };

        Shape::new(ShapeId::from_b2(shape_id))
    }

    /// Create a polygon shape and attach it to a body.
    ///
    /// Some failure cases:
    /// - All points very close together
    /// - All points on a line
    /// - Less than 3 points
    /// - More than [`ShapeId::MAX_POLYGON_POINTS`].
    ///
    /// We weld close points and remove collinear points.
    ///
    /// If a hull would be made empty, no polygon is attached.
    #[must_use = "a degenerate hull attaches no polygon, leaving the body without this collider"]
    pub fn attach_polygon(&self, polygon_points: &[Vec2], shape_def: ShapeDefinition) -> Option<Shape<'a>> {
        if polygon_points.len() > ShapeId::MAX_POLYGON_POINTS || polygon_points.len() < 3 {
            return None;
        }

        // safety: glam::Vec2 and b2Vec2 are identical in memory.
        let hull = unsafe {
            sys::b2ComputeHull(
                polygon_points.as_ptr() as *const sys::b2Vec2,
                polygon_points.len() as i32,
            )
        };
        if hull.count == 0 {
            return None;
        }

        let shape_id =
            unsafe { sys::b2CreatePolygonShape(self.raw(), shape_def.as_b2(), &sys::b2MakePolygon(&hull, 0.0)) };

        Some(Shape::new(ShapeId::from_b2(shape_id)))
    }
}

impl<'a> Deref for Body<'a> {
    type Target = BodyRef<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::fmt::Debug for Body<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id().fmt(f)
    }
}
