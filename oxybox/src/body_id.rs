use glam::Vec2;
use std::os::raw::c_void;

use crate::{ShapeDefinition, ShapeId, World};

/// Body id references a body instance. This should be treated as an opaque handle.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct BodyId(sys::b2BodyId);

impl BodyId {
    /// Creates a [`BodyId`] from a [`sys::b2BodyId`].
    pub fn from_b2(input: sys::b2BodyId) -> Self {
        Self(input)
    }

    /// Get the world position of a body. This is the location of the body origin.
    pub fn position(&self) -> Vec2 {
        unsafe { sys::b2Body_GetPosition(self.0).into() }
    }

    /// Get the body type: [`BodyKind::Static`], [`BodyKind::Kinematic`], or [`BodyKind::Dynamic`].
    pub fn kind(&self) -> BodyKind {
        let b2body_type = unsafe { sys::b2Body_GetType(self.0) };

        match b2body_type {
            sys::b2BodyType_b2_dynamicBody => BodyKind::Dynamic,
            sys::b2BodyType_b2_kinematicBody => BodyKind::Kinematic,
            sys::b2BodyType_b2_staticBody => BodyKind::Static,
            _ => panic!("Unexpected b2BodyType"),
        }
    }

    /// Sets arbitrary user data on the body.
    pub fn set_user_data(&self, data: u64) {
        unsafe {
            sys::b2Body_SetUserData(self.0, data as usize as *mut c_void);
        }
    }

    /// Get the user data stored in a body, if any.
    pub fn user_data(&self) -> Option<u64> {
        unsafe {
            let ptr = sys::b2Body_GetUserData(self.0);
            (!ptr.is_null()).then_some(ptr as u64)
        }
    }

    /// Get the world rotation of a body in radians.
    pub fn rotation(&self) -> f32 {
        let r = unsafe { sys::b2Body_GetRotation(self.0) };
        r.s.atan2(r.c)
    }

    /// Get the linear velocity of a body’s center of mass. Usually in meters per second.
    pub fn linear_velocity(&self) -> Vec2 {
        unsafe { sys::b2Body_GetLinearVelocity(self.0).into() }
    }

    /// Set the linear velocity of a body. Usually in meters per second.
    pub fn set_linear_velocity(&self, linear_velocity: Vec2) {
        unsafe {
            sys::b2Body_SetLinearVelocity(self.0, linear_velocity.into());
        }
    }

    /// Set the world transform of a body. This acts as a teleport and is fairly expensive.
    /// Generally you should create a body with then intended transform.
    ///
    /// `rotation` is in radians.
    pub fn set_tranfsorm(&self, position: Vec2, rotation: f32) {
        unsafe {
            sys::b2Body_SetTransform(
                self.0,
                position.into(),
                sys::b2Rot {
                    c: rotation.cos(),
                    s: rotation.sin(),
                },
            );
        }
    }

    /// Apply an impulse to the center of mass. This immediately modifies the velocity.
    /// The impulse is ignored if the body is not awake. This optionally wakes the body.
    ///
    /// `impulse` is the world impulse vector, usually in Ns or kgm/s.
    pub fn apply_impulse(&self, impulse: Vec2) {
        unsafe { sys::b2Body_ApplyLinearImpulseToCenter(self.0, impulse.into(), true) }
    }

    /// Apply an impulse at a point. This immediately modifies the velocity.
    /// It also modifies the angular velocity if the point of application is not at the center of mass.
    ///
    /// `impulse` is the world impulse vector, usually in Ns or kgm/s.
    /// `point` is the world position of the point of application.
    pub fn apply_impulse_at(&self, impulse: Vec2, point: Vec2) {
        unsafe { sys::b2Body_ApplyLinearImpulse(self.0, impulse.into(), point.into(), true) }
    }

    /// Apply an angular impulse. The impulse is ignored if the body is not awake.
    ///
    /// `impulse` is the angular impulse, usually in units of kgmm/s.
    /// `wake` is also wake up the body.
    pub fn apply_angular_impulse(&self, impulse: f32) {
        unsafe { sys::b2Body_ApplyAngularImpulse(self.0, impulse, true) }
    }

    /// Get the mass of the body, usually in kilograms.
    pub fn mass(&self) -> f32 {
        unsafe { sys::b2Body_GetMass(self.0) }
    }

    /// Destroy a rigid body given an id. This destroys all shapes and joints attached to the body.
    ///
    /// Do not keep references to the associated shapes and joints.
    pub fn destroy_body(self) {
        unsafe {
            sys::b2DestroyBody(self.0);
        }
    }

    /// Body identifier validation. Can be used to detect orphaned ids. Provides validation for up to 64K allocations.
    pub fn body_valid(&self) -> bool {
        unsafe { sys::b2Body_IsValid(self.0) }
    }

    /// Create a rigid body given a definition.
    pub fn create(world: &World, body_definition: &BodyDefinition) -> BodyId {
        let body_id = unsafe { sys::b2CreateBody(world.id, &body_definition.0) };

        BodyId::from_b2(body_id)
    }

    /// Attaches a circle to the body.
    ///
    /// The `center` is the local offset from the body, and the `radius` is the radius of the circle.
    pub fn attach_circle(self, center: Vec2, radius: f32, shape_def: &ShapeDefinition) -> ShapeId {
        ShapeId::create_circle(self, center, radius, shape_def)
    }

    /// Attaches a rectangle to the body.
    ///
    /// Make a box (rectangle) polygon, bypassing the need for a convex hull.
    /// `half_dims` are the half dimensions of the rectangle, `offset` is the offset relative to the body,
    /// and `rotation` is the rotation amount in radians.
    pub fn attach_rectangle(
        self,
        half_dims: Vec2,
        offset: Vec2,
        rotation: f32,
        shape_def: &ShapeDefinition,
    ) -> ShapeId {
        ShapeId::create_rectangle(self, half_dims, offset, rotation, shape_def)
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
    #[must_use]
    pub fn attach_polygon(self, polygon_points: &[Vec2], shape_def: &ShapeDefinition) -> Option<ShapeId> {
        ShapeId::create_polygon(self, polygon_points, shape_def)
    }
}

impl From<sys::b2BodyId> for BodyId {
    fn from(value: sys::b2BodyId) -> Self {
        Self::from_b2(value)
    }
}

impl From<BodyId> for sys::b2BodyId {
    fn from(value: BodyId) -> Self {
        value.0
    }
}

impl std::fmt::Debug for BodyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(&format!("{}@{}v{}", self.0.world0, self.0.index1, self.0.generation))
    }
}

/// A body definition holds all the data needed to construct a rigid body.
///
/// You can safely re-use body definitions. Shapes are added to a body after construction.
/// Body definitions are temporary objects used to bundle creation parameters.
#[repr(transparent)]
pub struct BodyDefinition(sys::b2BodyDef);

impl BodyDefinition {
    /// Creates a new BodyDefinition for use in creating a body.
    pub fn new() -> Self {
        Self(unsafe { sys::b2DefaultBodyDef() })
    }

    /// The body type: static, kinematic, or dynamic. The default is [`BodyKind::Static`].
    pub fn kind(mut self, kind: BodyKind) -> Self {
        self.0.type_ = kind as u32;

        self
    }

    /// The initial world position of the body. Bodies should be created with the desired position.
    /// Creating bodies at the origin and then moving them nearly doubles the cost of body creation,
    /// especially if the body is moved after shapes have been added.
    pub fn position(mut self, position: Vec2) -> Self {
        self.0.position = position.into();

        self
    }

    /// The initial world rotation of the body. This number is in radians.
    pub fn rotation(mut self, rotation: f32) -> Self {
        self.0.rotation = sys::b2Rot {
            c: rotation.cos(),
            s: rotation.sin(),
        };

        self
    }

    /// Treat this body as high speed object that performs continuous collision detection against dynamic
    /// and kinematic bodies, but not other bullet bodies.
    ///
    /// Bullets should be used sparingly. They are not a solution for general dynamic-versus-dynamic continuous collision.
    /// They may interfere with joint constraints.
    pub fn is_bullet(mut self, is_bullet: bool) -> Self {
        self.0.isBullet = is_bullet;

        self
    }

    /// The initial linear velocity of the body’s origin. Usually in meters per second.
    pub fn linear_velocity(mut self, linear_velocity: Vec2) -> Self {
        self.0.linearVelocity = linear_velocity.into();
        self
    }

    /// The initial angular velocity of the body. Radians per second.
    pub fn angular_velocity(mut self, angular_velocity: f32) -> Self {
        self.0.angularVelocity = angular_velocity;
        self
    }

    /// Linear damping is used to reduce the linear velocity.
    ///
    /// The damping parameter can be larger than 1.0 but the damping effect becomes sensitive to the time step when the damping
    /// parameter is large. Generally linear damping is undesirable because it makes objects move slowly as if they are floating.
    pub fn linear_damping(mut self, linear_damping: f32) -> Self {
        self.0.linearDamping = linear_damping;
        self
    }

    /// Angular damping is used to reduce the angular velocity.
    ///
    /// The damping parameter can be larger than 1.0 but the damping effect becomes sensitive to the time step when the damping
    /// parameter is large. Angular damping can be use slow down rotating bodies.
    pub fn angular_damping(mut self, angular_damping: f32) -> Self {
        self.0.angularDamping = angular_damping;

        self
    }

    /// Use this to store application specific body data.
    pub fn user_data(mut self, user_data: u64) -> Self {
        self.0.userData = user_data as _;

        self
    }
}

impl Default for BodyDefinition {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Default)]
#[repr(u32)]
pub enum BodyKind {
    /// Positive mass, velocity determined by forces, moved by solver
    Dynamic = sys::b2BodyType_b2_dynamicBody,

    /// Zero mass, velocity set by user, moved by solver
    Kinematic = sys::b2BodyType_b2_kinematicBody,

    /// Zero mass, zero velocity, may be manually moved
    #[default]
    Static = sys::b2BodyType_b2_staticBody,
}

impl BodyKind {
    /// Returns if this body matches [`BodyKind::Dynamic`].
    pub fn is_dynamic(self) -> bool {
        self == BodyKind::Dynamic
    }

    /// Returns if this body matches [`BodyKind::Static`].
    pub fn is_static(self) -> bool {
        self == BodyKind::Static
    }

    /// Returns if this body matches [`BodyKind::Kinematic`].
    pub fn is_kinematic(self) -> bool {
        self == BodyKind::Kinematic
    }
}
