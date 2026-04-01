use glam::Vec2;

use crate::{Body, BodyId, ShapeId, WorldId};

/// A physics world.
///
/// A world contains bodies, shapes, and constraints. You make create up to 128 worlds.
/// Each world is completely independent and may be simulated in parallel.
#[derive(Debug, Clone, Copy)]
pub struct World {
    id: WorldId,
    dt: f32,
}

impl World {
    const SUBSTEPS: i32 = 4;

    /// Create a world for rigid body simulation.
    pub fn new(delta_time: f32) -> Self {
        let id = unsafe { WorldId::from_b2(sys::b2CreateWorld(&sys::b2DefaultWorldDef())) };
        Self { id, dt: delta_time }
    }

    /// Gets the world's id.
    pub fn id(&self) -> WorldId {
        self.id
    }

    /// The amount of time to simulate when we call [`World::step`].
    ///
    /// This should be a fixed number. Usually `1.0 / 60.0`.
    pub fn dt(&self) -> f32 {
        self.dt
    }

    pub fn body(&self, body_id: BodyId) -> Body {
        let (body_id, shape_id) = unsafe {
            let count = sys::b2Body_GetShapeCount(*body_id);
            assert_eq!(count, 1, "oxybox can only handle 1 shape per body right now");
            let mut vec = Vec::with_capacity(count as usize);
            sys::b2Body_GetShapes(*body_id, vec.as_mut_ptr(), count);
            vec.set_len(count as usize);
            (body_id, ShapeId::from_b2(vec[0]))
        };
        Body::new(body_id, shape_id, self.id)
    }

    /// Simulate a world for one time step.
    /// This performs collision detection, integration, and constraint solution.
    pub fn step(&self) {
        unsafe {
            sys::b2World_Step(*self.id, self.dt, Self::SUBSTEPS);
        }
    }

    /// Destroy a world.
    pub fn destroy(self) {
        unsafe { sys::b2DestroyWorld(*self.id) }
    }

    /// Set the gravity vector for the entire world. Box2D has no concept of an up direction and
    /// this is left as a decision for the application. Usually in m/s^2.
    pub fn set_gravity(&self, gravity: Vec2) {
        unsafe { sys::b2World_SetGravity(*self.id, gravity.into()) }
    }

    /// Sets the pixels-per-meter Box2D will expect. While you're free to work with whatever units
    /// you please, setting this will help Box2D tweak internal numbers to better work with your
    /// expectations.
    ///
    /// **NOTE: This is a global value -- Box2D does not support different unit lengths per-world.**
    pub fn set_length_units_per_meter(&self, ppm: f32) {
        unsafe { sys::b2SetLengthUnitsPerMeter(ppm) }
    }

    /// Get the current length units per meter.
    pub fn length_units_per_meter(&self) -> f32 {
        unsafe { sys::b2GetLengthUnitsPerMeter() }
    }

    /// Create a rigid body given a definition.
    pub fn create_body(&self, body_definition: &crate::BodyDefinition) -> Body {
        Body::create(self.id, body_definition)
    }

    /// Get contact events for this current time step.
    pub fn contact_events(&self) -> impl Iterator<Item = (BodyId, BodyId)> {
        unsafe {
            let contact_events = sys::b2World_GetContactEvents(*self.id);
            let begin_events: &mut [sys::b2ContactBeginTouchEvent] =
                std::slice::from_raw_parts_mut(contact_events.beginEvents, contact_events.beginCount as usize);

            begin_events.iter_mut().filter_map(|e| {
                if !sys::b2Shape_IsValid(e.shapeIdA) || !sys::b2Shape_IsValid(e.shapeIdB) {
                    None
                } else {
                    Some((
                        sys::b2Shape_GetBody(e.shapeIdA).into(),
                        sys::b2Shape_GetBody(e.shapeIdB).into(),
                    ))
                }
            })
        }
    }
}
