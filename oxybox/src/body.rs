use glam::Vec2;
use std::os::raw::c_void;

use crate::{BodyId, ShapeId, WorldId};

/// A body, with associated shapes, on a given world.
#[derive(Debug, Copy, Clone)]
pub struct Body {
    body_id: BodyId,
    shape_id: ShapeId,
    world_id: WorldId,
}

impl Body {
    /// Creates a new Body.
    pub fn new(body_id: BodyId, shape_id: ShapeId, world_id: WorldId) -> Self {
        Self {
            body_id,
            shape_id,
            world_id,
        }
    }

    /// Get the world position of a body. This is the location of the body origin.
    pub fn position(&self) -> Vec2 {
        unsafe { sys::b2Body_GetPosition(*self.body_id).into() }
    }

    /// Gets the [`BodyId`] of the body.
    pub fn body_id(&self) -> BodyId {
        self.body_id
    }

    pub fn shape_id(&self) -> ShapeId {
        self.shape_id
    }

    /// Gets the [`WorldId`] of the body.
    pub fn world_id(&self) -> WorldId {
        self.world_id
    }

    /// Get the body type: [`BodyKind::Static`], [`BodyKind::Kinematic`], or [`BodyKind::Dynamic`].
    pub fn kind(&self) -> BodyKind {
        let b2body_type = unsafe { sys::b2Body_GetType(*self.body_id) };

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
            sys::b2Body_SetUserData(*self.body_id, data as usize as *mut c_void);
        }
    }

    /// Get the user data stored in a body, if any.
    pub fn user_data(&self) -> Option<u64> {
        unsafe {
            let ptr = sys::b2Body_GetUserData(*self.body_id);
            (!ptr.is_null()).then_some(ptr as u64)
        }
    }

    /// Get the type of a shape
    pub fn body_shape(&self) -> BodyShape {
        let shape = unsafe { sys::b2Shape_GetType(*self.shape_id()) };
        match shape {
            sys::b2ShapeType_b2_circleShape => BodyShape::Circle,
            sys::b2ShapeType_b2_polygonShape => BodyShape::Polygon,
            s => unimplemented!("{s:?}"),
        }
    }

    /// Gets the dimensions of the body. This can be imagined as a box which will
    /// fully enclose the given shape.
    pub fn dimensions(&self) -> Vec2 {
        Vec2::new(self.width(), self.height())
    }

    /// Gets the width of the given shape.
    pub fn width(&self) -> f32 {
        unsafe {
            match self.body_shape() {
                BodyShape::Circle => sys::b2Shape_GetCircle(*self.shape_id).radius * 2.0,
                BodyShape::Polygon => {
                    let polygon = sys::b2Shape_GetPolygon(*self.shape_id);
                    let mut min_x = f32::INFINITY;
                    let mut max_x = f32::NEG_INFINITY;
                    for i in 0..polygon.count as usize {
                        let x = polygon.vertices[i].x;
                        min_x = min_x.min(x);
                        max_x = max_x.max(x);
                    }
                    max_x - min_x
                }
            }
        }
    }

    /// Gets the height of the given shape.
    pub fn height(&self) -> f32 {
        unsafe {
            match self.body_shape() {
                BodyShape::Circle => sys::b2Shape_GetCircle(*self.shape_id).radius * 2.0,
                BodyShape::Polygon => {
                    let polygon = sys::b2Shape_GetPolygon(*self.shape_id);
                    let mut min_y = f32::INFINITY;
                    let mut max_y = f32::NEG_INFINITY;
                    for i in 0..polygon.count as usize {
                        let y = polygon.vertices[i].y;
                        min_y = min_y.min(y);
                        max_y = max_y.max(y);
                    }
                    max_y - min_y
                }
            }
        }
    }

    /// Get the world rotation of a body in radians.
    pub fn rotation(&self) -> f32 {
        let r = unsafe { sys::b2Body_GetRotation(*self.body_id) };
        r.s.atan2(r.c)
    }

    /// Get the linear velocity of a body’s center of mass. Usually in meters per second.
    pub fn linear_velocity(&self) -> Vec2 {
        unsafe { sys::b2Body_GetLinearVelocity(*self.body_id).into() }
    }

    /// Set the linear velocity of a body. Usually in meters per second.
    pub fn set_linear_velocity(&self, linear_velocity: Vec2) {
        unsafe {
            sys::b2Body_SetLinearVelocity(*self.body_id, linear_velocity.into());
        }
    }

    /// Set the world transform of a body. This acts as a teleport and is fairly expensive.
    /// Generally you should create a body with then intended transform.
    ///
    /// `rotation` is in radians.
    pub fn set_tranfsorm(&self, position: Vec2, rotation: f32) {
        unsafe {
            sys::b2Body_SetTransform(
                *self.body_id,
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
        unsafe { sys::b2Body_ApplyLinearImpulseToCenter(*self.body_id, impulse.into(), true) }
    }

    /// Apply an impulse at a point. This immediately modifies the velocity.
    /// It also modifies the angular velocity if the point of application is not at the center of mass.
    ///
    /// `impulse` is the world impulse vector, usually in Ns or kgm/s.
    /// `point` is the world position of the point of application.
    pub fn apply_impulse_at(&self, impulse: Vec2, point: Vec2) {
        unsafe { sys::b2Body_ApplyLinearImpulse(*self.body_id, impulse.into(), point.into(), true) }
    }

    /// Apply an angular impulse. The impulse is ignored if the body is not awake.
    ///
    /// `impulse` is the angular impulse, usually in units of kgmm/s.
    /// `wake` is also wake up the body.
    pub fn apply_angular_impulse(&self, impulse: f32) {
        unsafe { sys::b2Body_ApplyAngularImpulse(*self.body_id, impulse, true) }
    }

    /// Get the mass of the body, usually in kilograms.
    pub fn mass(&self) -> f32 {
        unsafe { sys::b2Body_GetMass(*self.body_id) }
    }

    /// Destroy a rigid body given an id. This destroys all shapes and joints attached to the body.
    ///
    /// Do not keep references to the associated shapes and joints.
    pub fn destroy_body(self) {
        unsafe {
            sys::b2DestroyBody(*self.body_id);
        }
    }

    /// Body identifier validation. Can be used to detect orphaned ids. Provides validation for up to 64K allocations.
    pub fn body_valid(&self) -> bool {
        unsafe { sys::b2Body_IsValid(*self.body_id) }
    }

    /// Shape identifier validation. Provides validation for up to 64K allocations.
    pub fn shape_valid(&self, shape_id: ShapeId) -> bool {
        unsafe { sys::b2Shape_IsValid(*shape_id) }
    }

    /// Create a rigid body given a definition.
    pub fn create(world: WorldId, body_definition: &BodyDefinition) -> Body {
        unsafe {
            let mut body = sys::b2DefaultBodyDef();
            body.type_ = body_definition.kind as u32;

            if let Some(position) = body_definition.position {
                body.position = position.into();
            }
            if let Some(angular_damping) = body_definition.angular_damping {
                body.angularDamping = angular_damping;
            }
            if let Some(linear_damping) = body_definition.linear_damping {
                body.linearDamping = linear_damping;
            }
            if let Some(rotation) = body_definition.rotation {
                body.rotation = sys::b2Rot {
                    c: rotation.cos(),
                    s: rotation.sin(),
                };
            }
            if let Some(linear_velocity) = body_definition.linear_velocity {
                body.linearVelocity = linear_velocity.into();
            }
            if let Some(angular_velocity) = body_definition.angular_velocity {
                body.angularVelocity = angular_velocity;
            }

            body.isBullet = body_definition.is_bullet;

            if let Some(user_data) = body_definition.user_data {
                body.userData = user_data as usize as *mut c_void;
            }

            let body_id = sys::b2CreateBody(*world, &body);

            let mut shape_definition = sys::b2DefaultShapeDef();
            if let Some(friction) = body_definition.friction {
                shape_definition.material.friction = friction;
            }

            if let Some(density) = body_definition.density {
                shape_definition.density = density;
            }
            if let Some(restitution) = body_definition.restitution {
                shape_definition.material.restitution = restitution;
            }
            if let Some(category) = body_definition.category {
                shape_definition.filter.categoryBits = category;
            }
            if let Some(mask) = body_definition.mask {
                shape_definition.filter.maskBits = mask;
            }
            shape_definition.isSensor = body_definition.is_sensor;
            shape_definition.enableContactEvents = body_definition.enable_contact_events;

            let shape_id = match body_definition.build_shape {
                BuildShape::Circle { center, radius } => sys::b2CreateCircleShape(
                    body_id,
                    &shape_definition,
                    &sys::b2Circle {
                        center: center.into(),
                        radius,
                    },
                ),
                BuildShape::Rectangle { dimensions } => sys::b2CreatePolygonShape(
                    body_id,
                    &shape_definition,
                    &sys::b2MakeBox(dimensions.x / 2.0, dimensions.y / 2.0),
                ),
            };

            Body {
                body_id: BodyId::from_b2(body_id),
                shape_id: ShapeId::from_b2(shape_id),
                world_id: world,
            }
        }
    }
}

pub struct BodyDefinition {
    /// The body type: static, kinematic, or dynamic.
    pub kind: BodyKind,

    /// The initial world position of the body. Bodies should be created with the desired position.
    /// Creating bodies at the origin and then moving them nearly doubles the cost of body creation,
    /// especially if the body is moved after shapes have been added.
    pub position: Option<Vec2>,

    /// The initial world rotation of the body.
    pub rotation: Option<f32>,

    /// Treat this body as high speed object that performs continuous collision detection against dynamic
    /// and kinematic bodies, but not other bullet bodies.
    ///
    /// Bullets should be used sparingly. They are not a solution for general dynamic-versus-dynamic continuous collision.
    /// They may interfere with joint constraints.
    pub is_bullet: bool,

    /// The initial linear velocity of the body’s origin. Usually in meters per second.
    pub linear_velocity: Option<Vec2>,

    /// The initial angular velocity of the body. Radians per second.
    pub angular_velocity: Option<f32>,

    /// Linear damping is used to reduce the linear velocity.
    ///
    /// The damping parameter can be larger than 1.0 but the damping effect becomes sensitive to the time step when the damping
    /// parameter is large. Generally linear damping is undesirable because it makes objects move slowly as if they are floating.
    pub linear_damping: Option<f32>,

    /// Angular damping is used to reduce the angular velocity.
    ///
    /// The damping parameter can be larger than 1.0 but the damping effect becomes sensitive to the time step when the damping
    /// parameter is large. Angular damping can be use slow down rotating bodies.
    pub angular_damping: Option<f32>,

    /// Use this to store application specific body data.
    pub user_data: Option<u64>,

    /// The shape to build and attach to the body.
    pub build_shape: BuildShape,

    /// The density, usually in kg/m^2.
    ///
    /// This is not part of the surface material because this is for the interior, which may have other considerations, such as
    /// being hollow. For example a wood barrel may be hollow or full of water.
    pub density: Option<f32>,

    /// The collision category bits. Normally you would just set one bit as a bitflag.
    ///
    /// The category bits should represent your application object types.
    pub category: Option<u64>,

    /// The collision mask bits. This states the categories that this shape would accept for collision.
    ///
    /// For example, you may want your player to only collide with static objects and other players.
    pub mask: Option<u64>,

    /// A sensor shape generates overlap events but never generates a collision response.
    ///
    /// Sensors do not have continuous collision. Instead, use a ray or shape cast for those scenarios.
    /// Sensors still contribute to the body mass if they have non-zero density. Sensor events are disabled by default.
    pub is_sensor: bool,

    /// Enable contact events for this shape.
    ///
    /// Only applies to kinematic and dynamic bodies. Ignored for sensors.
    pub enable_contact_events: bool,

    /// The coefficient of restitution (bounce) usually in the range `0.0..=1.0`.
    ///
    /// See [wikipedia](https://en.wikipedia.org/wiki/Coefficient_of_restitution).
    pub restitution: Option<f32>,

    /// The Coulomb (dry) friction coefficient, usually in the range `0.0..=1.0`.
    pub friction: Option<f32>,
}

impl Default for BodyDefinition {
    fn default() -> Self {
        Self {
            build_shape: BuildShape::Circle {
                center: Vec2::ZERO,
                radius: 0.0,
            },
            kind: BodyKind::default(),
            position: None,
            rotation: None,
            linear_velocity: None,
            angular_velocity: None,
            linear_damping: None,
            angular_damping: None,
            user_data: None,
            is_bullet: false,
            density: None,
            category: None,
            mask: None,
            is_sensor: false,
            enable_contact_events: false,
            restitution: None,
            friction: None,
        }
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

#[derive(Debug, PartialEq)]
pub enum BuildShape {
    Circle {
        center: Vec2,
        radius: f32,
    },

    /// Make a box (rectangle) polygon, bypassing the need for a convex hull.
    /// Dimensions are the dimensions of the rectangle.
    Rectangle {
        dimensions: Vec2,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum BodyShape {
    /// A circle with an offset
    Circle = sys::b2ShapeType_b2_circleShape,

    /// A convex polygon. Often, this is a rectangle.
    Polygon = sys::b2ShapeType_b2_polygonShape,
}
