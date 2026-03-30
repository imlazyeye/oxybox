use glam::Vec2;
use oxybox_sys::{self as sys};
use std::os::raw::c_void;

use crate::{World, WorldId};

#[derive(Debug, Copy, Clone)]
pub struct Body {
    body_id: BodyId,
    shape_id: ShapeId,
    world_id: WorldId,
}

impl Body {
    pub fn new(body_id: BodyId, shape_id: ShapeId, world_id: WorldId) -> Self {
        Self {
            body_id,
            shape_id,
            world_id,
        }
    }

    pub fn position(&self) -> Vec2 {
        unsafe { sys::b2Body_GetPosition(*self.body_id).into() }
    }

    pub fn body_id(&self) -> BodyId {
        self.body_id
    }

    pub fn shape_id(&self) -> ShapeId {
        self.shape_id
    }

    pub fn world_id(&self) -> WorldId {
        self.world_id
    }

    pub fn kind(&self) -> BodyKind {
        match unsafe { sys::b2Body_GetType(*self.body_id) } {
            #[allow(non_upper_case_globals)]
            sys::b2BodyType_b2_dynamicBody => BodyKind::Dynamic,
            #[allow(non_upper_case_globals)]
            sys::b2BodyType_b2_kinematicBody => BodyKind::Kinematic,
            #[allow(non_upper_case_globals)]
            sys::b2BodyType_b2_staticBody => BodyKind::Static,
            _ => unimplemented!(),
        }
    }

    pub fn set_user_data(&self, data: u64) {
        unsafe {
            sys::b2Body_SetUserData(*self.body_id, data as usize as *mut c_void);
        }
    }

    pub fn user_data(&self) -> Option<u64> {
        unsafe {
            let ptr = sys::b2Body_GetUserData(*self.body_id);
            if ptr.is_null() { None } else { Some(ptr as usize as u64) }
        }
    }

    /// Returns if this body is dynamic.
    ///
    /// Keep in mind this is _not_ the same thing as being static as it could also be kinematic. Use
    /// [Body::is_static] instead.
    pub fn is_dynamic(&self) -> bool {
        self.kind() == BodyKind::Dynamic
    }

    pub fn is_static(&self) -> bool {
        self.kind() == BodyKind::Static
    }

    pub fn is_kinematic(&self) -> bool {
        self.kind() == BodyKind::Kinematic
    }

    pub fn body_shape(&self) -> BodyShape {
        let shape = unsafe { sys::b2Shape_GetType(*self.shape_id()) };
        match shape {
            #[allow(non_upper_case_globals)]
            sys::b2ShapeType_b2_circleShape => BodyShape::Circle,
            #[allow(non_upper_case_globals)]
            sys::b2ShapeType_b2_polygonShape => BodyShape::Rectangle,
            s => unimplemented!("{s:?}"),
        }
    }

    pub fn dimensions(&self) -> Vec2 {
        Vec2::new(self.width(), self.height())
    }

    pub fn width(&self) -> f32 {
        unsafe {
            match self.body_shape() {
                BodyShape::Circle => sys::b2Shape_GetCircle(*self.shape_id).radius * 2.0,
                BodyShape::Rectangle => {
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

    pub fn height(&self) -> f32 {
        unsafe {
            match self.body_shape() {
                BodyShape::Circle => sys::b2Shape_GetCircle(*self.shape_id).radius * 2.0,
                BodyShape::Rectangle => {
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

    pub fn rotation(&self) -> f32 {
        let r = unsafe { sys::b2Body_GetRotation(*self.body_id) };
        r.s.atan2(r.c)
    }

    pub fn linear_velocity(&self) -> Vec2 {
        unsafe { sys::b2Body_GetLinearVelocity(*self.body_id).into() }
    }

    pub fn set_linear_velocity(&self, linear_velocity: Vec2) {
        unsafe {
            sys::b2Body_SetLinearVelocity(*self.body_id, linear_velocity.into());
        }
    }

    pub fn set_rotation(&self, rotation: f32) {
        unsafe {
            sys::b2Body_SetTransform(
                *self.body_id,
                self.position().into(),
                sys::b2Rot {
                    c: rotation.cos(),
                    s: rotation.sin(),
                },
            );
        }
    }

    pub fn set_position(&self, position: Vec2) {
        unsafe {
            sys::b2Body_SetTransform(*self.body_id, position.into(), sys::b2Body_GetRotation(*self.body_id));
        }
    }

    pub fn apply_impulse(&self, impulse: Vec2) {
        unsafe { sys::b2Body_ApplyLinearImpulseToCenter(*self.body_id, impulse.into(), true) }
    }

    pub fn apply_impulse_at(&self, impulse: Vec2, local_position: Vec2) {
        unsafe { sys::b2Body_ApplyLinearImpulse(*self.body_id, impulse.into(), local_position.into(), true) }
    }

    pub fn apply_angular_impulse(&self, impulse: f32) {
        unsafe { sys::b2Body_ApplyAngularImpulse(*self.body_id, impulse, true) }
    }

    pub fn mass(&self) -> f32 {
        unsafe { sys::b2Body_GetMass(*self.body_id) }
    }

    pub fn contact_begin_bodies<'w>(&self, world: &'w World) -> impl Iterator<Item = Body> + use<'w> {
        assert!(
            unsafe { sys::b2Shape_AreContactEventsEnabled(*self.shape_id) },
            "You must enable contact events to read contact events!"
        );

        let body = self.body_id;

        world.contact_events().filter_map(move |(a, b)| {
            if a == body {
                Some(world.body(b))
            } else if b == body {
                Some(world.body(a))
            } else {
                None
            }
        })
    }

    pub fn contact_begin(&self, other: Body, world: &World) -> bool {
        self.contact_begin_bodies(world).any(|v| v.body_id == other.body_id)
    }
}

pub struct BodyBuilder {
    held_shape: HeldShape,
    kind: Option<BodyKind>,
    position: Option<Vec2>,
    rotation: Option<f32>,
    linear_velocity: Option<Vec2>,
    angular_velocity: Option<f32>,
    linear_damping: Option<f32>,
    angular_damping: Option<f32>,
    user_data: Option<u64>,
    is_bullet: Option<bool>,
    density: Option<f32>,
    category: Option<u64>,
    mask: Option<u64>,
    is_sensor: Option<bool>,
    enable_contact_events: Option<bool>,
    restitution: Option<f32>,
    friction: Option<f32>,
}

impl BodyBuilder {
    pub fn circle(radius: f32) -> Self {
        Self {
            held_shape: HeldShape::Circle(sys::b2Circle {
                center: Vec2::ZERO.into(),
                radius,
            }),
            ..Default::default()
        }
    }

    pub fn rectangle(dimensions: Vec2) -> Self {
        unsafe {
            Self {
                held_shape: HeldShape::Rectangle(sys::b2MakeBox(dimensions.x / 2.0, dimensions.y / 2.0)),
                ..Default::default()
            }
        }
    }

    pub fn kind(mut self, kind: BodyKind) -> Self {
        self.kind = Some(kind);
        self
    }

    pub fn position(mut self, position: Vec2) -> Self {
        self.position = Some(position);
        self
    }

    pub fn rotation(mut self, rotation: f32) -> Self {
        self.rotation = Some(rotation);
        self
    }

    pub fn linear_damping(mut self, damping: f32) -> Self {
        self.linear_damping = Some(damping);
        self
    }

    pub fn friction(mut self, friction: f32) -> Self {
        self.friction = Some(friction);
        self
    }

    pub fn density(mut self, density: f32) -> Self {
        self.density = Some(density);
        self
    }

    pub fn angular_damping(mut self, damping: f32) -> Self {
        self.angular_damping = Some(damping);
        self
    }

    pub fn restitution(mut self, restitution: f32) -> Self {
        self.restitution = Some(restitution);
        self
    }

    pub fn linear_velocity(mut self, velocity: Vec2) -> Self {
        self.linear_velocity = Some(velocity);
        self
    }

    pub fn angular_velocity(mut self, velocity: f32) -> Self {
        self.angular_velocity = Some(velocity);
        self
    }

    pub fn bullet(mut self) -> Self {
        assert_ne!(
            self.is_sensor,
            Some(true),
            "A body cannot be both a bullet and a sensor!"
        );
        self.is_bullet = Some(true);
        self
    }

    pub fn category(mut self, category: u64) -> Self {
        self.category = Some(category);
        self
    }

    pub fn mask(mut self, mask: u64) -> Self {
        self.mask = Some(mask);
        self
    }

    pub fn user_data(mut self, data: u64) -> Self {
        self.user_data = Some(data);
        self
    }

    pub fn sensor(mut self) -> Self {
        assert_ne!(
            self.is_bullet,
            Some(true),
            "A body cannot be both a bullet and a sensor!"
        );
        self.is_sensor = Some(true);
        self
    }

    pub fn enable_contact_events(mut self) -> Self {
        self.enable_contact_events = Some(true);
        self
    }

    pub fn build(self, world: WorldId) -> Body {
        unsafe {
            let mut body = sys::b2DefaultBodyDef();
            let mut shape = sys::b2DefaultShapeDef();

            if let Some(kind) = self.kind {
                body.type_ = match kind {
                    BodyKind::Dynamic => sys::b2BodyType_b2_dynamicBody,
                    BodyKind::Kinematic => sys::b2BodyType_b2_kinematicBody,
                    BodyKind::Static => sys::b2BodyType_b2_staticBody,
                };
            }

            if let Some(position) = self.position {
                body.position = position.into();
            }
            if let Some(angular_damping) = self.angular_damping {
                body.angularDamping = angular_damping;
            }
            if let Some(linear_damping) = self.linear_damping {
                body.linearDamping = linear_damping;
            }
            if let Some(rotation) = self.rotation {
                body.rotation = sys::b2Rot {
                    c: rotation.cos(),
                    s: rotation.sin(),
                };
            }
            if let Some(friction) = self.friction {
                shape.material.friction = friction;
            }
            if let Some(linear_velocity) = self.linear_velocity {
                body.linearVelocity = linear_velocity.into();
            }
            if let Some(angular_velocity) = self.angular_velocity {
                body.angularVelocity = angular_velocity;
            }
            if let Some(is_bullet) = self.is_bullet {
                body.isBullet = is_bullet;
            }
            if let Some(user_data) = self.user_data {
                body.userData = user_data as usize as *mut c_void;
                shape.userData = user_data as usize as *mut c_void;
            }
            if let Some(density) = self.density {
                shape.density = density;
            }
            if let Some(restitution) = self.restitution {
                shape.material.restitution = restitution;
            }
            if let Some(category) = self.category {
                shape.filter.categoryBits = category;
            }
            if let Some(mask) = self.mask {
                shape.filter.maskBits = mask;
            }
            if let Some(is_sensor) = self.is_sensor {
                shape.isSensor = is_sensor;
            }
            if let Some(enable_contact_events) = self.enable_contact_events {
                shape.enableContactEvents = enable_contact_events;
            }

            let body_id = sys::b2CreateBody(*world, &body);
            let shape_id = match self.held_shape {
                HeldShape::Circle(b2_circle) => sys::b2CreateCircleShape(body_id, &shape, &b2_circle),
                HeldShape::Rectangle(b2_polygon) => sys::b2CreatePolygonShape(body_id, &shape, &b2_polygon),
            };

            Body {
                body_id: BodyId(body_id),
                shape_id: ShapeId(shape_id),
                world_id: world,
            }
        }
    }
}

impl Default for BodyBuilder {
    fn default() -> Self {
        Self {
            held_shape: HeldShape::Circle(sys::b2Circle {
                center: Vec2::ZERO.into(),
                radius: 0.0,
            }),
            kind: None,
            position: None,
            rotation: None,
            linear_velocity: None,
            angular_velocity: None,
            linear_damping: None,
            angular_damping: None,
            user_data: None,
            is_bullet: None,
            density: None,
            category: None,
            mask: None,
            is_sensor: None,
            enable_contact_events: None,
            restitution: None,
            friction: None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BodyKind {
    Dynamic,
    Kinematic,
    Static,
}

enum HeldShape {
    Circle(sys::b2Circle),
    Rectangle(sys::b2Polygon),
}

#[derive(Copy, Clone)]
pub struct BodyId(sys::b2BodyId);
impl PartialEq for BodyId {
    fn eq(&self, other: &Self) -> bool {
        let a = self.0;
        let b = other.0;
        a.generation == b.generation && a.index1 == b.index1 && a.world0 == b.world0
    }
}
impl Eq for BodyId {}

impl std::fmt::Debug for BodyId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(&format!("{}@{}v{}", self.0.index1, self.0.world0, self.0.generation))
    }
}

impl std::hash::Hash for BodyId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let a = self.0;
        a.world0.hash(state);
        a.index1.hash(state);
        a.generation.hash(state);
    }
}
impl std::ops::Deref for BodyId {
    type Target = sys::b2BodyId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<sys::b2BodyId> for BodyId {
    fn from(value: sys::b2BodyId) -> Self {
        Self(value)
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ShapeId(sys::b2ShapeId);
impl PartialEq for ShapeId {
    fn eq(&self, other: &Self) -> bool {
        let a = self.0;
        let b = other.0;
        a.generation == b.generation && a.index1 == b.index1 && a.world0 == b.world0
    }
}
impl Eq for ShapeId {}

impl std::hash::Hash for ShapeId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let a = self.0;
        a.world0.hash(state);
        a.index1.hash(state);
        a.generation.hash(state);
    }
}
impl std::ops::Deref for ShapeId {
    type Target = sys::b2ShapeId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<sys::b2ShapeId> for ShapeId {
    fn from(value: sys::b2ShapeId) -> Self {
        Self(value)
    }
}

pub enum BodyShape {
    Circle,
    Rectangle,
}

// No Drop: bodies are cleaned up when the World is destroyed.
// If you want manual removal later, expose a `World::destroy_body(b: Body)` that calls the proper C
// API.
