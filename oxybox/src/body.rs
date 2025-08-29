use std::hash::Hash;

use crate::World;
use glam::Vec2;
use oxybox_sys::*;

pub struct BodyBuilder {
    body: b2BodyDef,
    shape: b2ShapeDef,
    held_shape_data: HeldShapeData,
}

pub enum BodyKind {
    Dynamic,
    Kinematic,
    Static,
}

pub enum Shape {
    Circle(f32),
    Box(Vec2),
}

enum HeldShapeData {
    Circle(b2Circle),
    Rectangle(b2Polygon),
}

// let body_id = b2CreateBody(world.raw(), &body_def);
impl BodyBuilder {
    pub fn circle(radius: f32) -> Self {
        unsafe {
            Self {
                body: b2DefaultBodyDef(),
                shape: b2DefaultShapeDef(),
                held_shape_data: HeldShapeData::Circle(b2Circle {
                    center: Vec2::ZERO.into(),
                    radius,
                }),
            }
        }
    }

    pub fn rectangle(dimensions: Vec2) -> Self {
        unsafe {
            Self {
                body: b2DefaultBodyDef(),
                shape: b2DefaultShapeDef(),
                held_shape_data: HeldShapeData::Rectangle(b2MakeBox(dimensions.x / 2.0, dimensions.y / 2.0)),
            }
        }
    }

    pub fn kind(mut self, kind: BodyKind) -> Self {
        self.body.type_ = match kind {
            BodyKind::Dynamic => b2BodyType_b2_dynamicBody,
            BodyKind::Kinematic => b2BodyType_b2_kinematicBody,
            BodyKind::Static => b2BodyType_b2_staticBody,
        };
        self
    }

    pub fn position(mut self, position: Vec2) -> Self {
        self.body.position = position.into();
        self
    }

    pub fn friction(mut self, friction: f32) -> Self {
        self.shape.material.friction = friction;
        self
    }

    pub fn density(mut self, density: f32) -> Self {
        self.shape.density = density;
        self
    }

    pub fn angular_damping(mut self, damping: f32) -> Self {
        self.body.angularDamping = damping;
        self
    }

    pub fn restitution(mut self, restitution: f32) -> Self {
        self.shape.material.restitution = restitution;
        self
    }

    pub fn linear_velocity(mut self, velocity: Vec2) -> Self {
        self.body.linearVelocity = velocity.into();
        self
    }

    pub fn angular_velocity(mut self, velocity: f32) -> Self {
        self.body.angularVelocity = velocity;
        self
    }

    pub fn bullet(mut self) -> Self {
        assert!(!self.shape.isSensor, "A body cannot be both a bullet and a sensor!");
        self.body.isBullet = true;
        self
    }

    // pub fn collision_groups(mut self, groups: InteractionGroups) -> Self {
    //     self.collider.set_collision_groups(groups);
    //     self
    // }

    pub fn sensor(mut self) -> Self {
        assert!(!self.body.isBullet, "A body cannot be both a bullet and a sensor!");
        self.shape.isSensor = true;
        self
    }

    // todo: this gives the appearance of being optional right now but in reality if you don't use it
    // everything will explode
    // pub fn entity(mut self, entity: Entity) -> Self {
    //     self.collider.user_data = entity.to_bits().get() as u128;
    //     self
    // }

    pub fn build(self, world: &mut World) -> BodyId {
        unsafe {
            let body_id = b2CreateBody(world.raw(), &self.body);
            let shape_id = match self.held_shape_data {
                HeldShapeData::Circle(b2_circle) => b2CreateCircleShape(body_id, &self.shape, &b2_circle),
                HeldShapeData::Rectangle(b2_polygon) => b2CreatePolygonShape(body_id, &self.shape, &b2_polygon),
            };

            let body_id = BodyId(body_id);
            let body = Body { body_id, shape_id };
            world.bodies.insert(body_id, body);

            body_id
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BodyId(b2BodyId);

impl PartialEq for BodyId {
    fn eq(&self, other: &Self) -> bool {
        self.0.generation == other.0.generation && self.0.index1 == other.0.index1 && self.0.world0 == other.0.world0
    }
}

impl Eq for BodyId {}

impl Hash for BodyId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let bits: u64 = unsafe { std::mem::transmute(self.0) };
        bits.hash(state);
    }
}

impl std::ops::Deref for BodyId {
    type Target = b2BodyId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Body {
    body_id: BodyId,
    shape_id: b2ShapeId,
}

impl Body {
    pub fn position(&self) -> Vec2 {
        unsafe { b2Body_GetPosition(*self.body_id).into() }
    }
}

pub enum BodyShape {
    Circle,
    Box,
}

// No Drop: bodies are cleaned up when the World is destroyed.
// If you want manual removal later, expose a `World::destroy_body(b: Body)` that calls the proper C
// API.
