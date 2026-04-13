use glam::Vec2;

use crate::{BodyId, ShapeId};

/// A physics world.
///
/// A world contains bodies, shapes, and constraints. You make create up to 128 worlds.
/// Each world is completely independent and may be simulated in parallel.
#[derive(Debug, Clone, Copy)]
pub struct World {
    pub id: sys::b2WorldId,
    pub dt: f32,
}

impl World {
    const SUBSTEPS: i32 = 4;

    /// Create a world for rigid body simulation.
    pub fn new(delta_time: f32) -> Self {
        let id = unsafe { sys::b2CreateWorld(&sys::b2DefaultWorldDef()) };
        Self { id, dt: delta_time }
    }

    /// The amount of time to simulate when we call [`World::step`].
    ///
    /// This should be a fixed number. Usually `1.0 / 60.0`.
    pub fn dt(&self) -> f32 {
        self.dt
    }

    /// Simulate a world for one time step.
    /// This performs collision detection, integration, and constraint solution.
    pub fn step(&self) {
        unsafe {
            sys::b2World_Step(self.id, self.dt, Self::SUBSTEPS);
        }
    }

    /// Destroy a world.
    pub fn destroy(self) {
        unsafe { sys::b2DestroyWorld(self.id) }
    }

    /// Set the gravity vector for the entire world. Box2D has no concept of an up direction and
    /// this is left as a decision for the application. Usually in m/s^2.
    pub fn set_gravity(&self, gravity: Vec2) {
        unsafe { sys::b2World_SetGravity(self.id, gravity.into()) }
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
    pub fn create_body(&self, body_definition: &crate::BodyDefinition) -> BodyId {
        BodyId::create(self, body_definition)
    }

    /// Overlap test for circles.
    /// 
    /// The callback will be called for each shape which overlaps with the provided circle. If the callback
    /// returns `false`, then we will stop iterating and return early.
    pub fn overlap_circle<OverlapFn>(&self, circle_position: Vec2, radius: f32, mut overlap: OverlapFn) -> OverlapStats
    where
        OverlapFn: FnMut(ShapeId) -> bool,
    {
        // safety: we are copying all data and we know that glam::Vec2 is the exact same as b2Vec2 so we
        // can make a pointer to it. Additionally, it survives this function entirely.
        let hit_circle = unsafe { sys::b2MakeProxy(&circle_position as *const Vec2 as *const sys::b2Vec2, 1, radius) };

        extern "C" fn overlap_trampoline<OverlapFn>(shape: sys::b2ShapeId, cback: *mut std::ffi::c_void) -> bool
        where
            OverlapFn: FnMut(ShapeId) -> bool,
        {
            // safety: Rust's type system promises that this is the same type of closure
            // which we are passing. We *are* passing this closure as an `&mut OverlapFn`
            // when we call `sys::b2World_OverlapShape`
            let cback: &mut OverlapFn = unsafe { &mut *(cback as *mut OverlapFn) };

            // call the guy!
            cback(ShapeId::from_b2(shape))
        }

        // safety: the callback is owned by us, and we can make a pointer to it, which we can cast to
        // `std::ffi::c_void`, which will get the closure back eventually.
        let performance_stats = unsafe {
            sys::b2World_OverlapShape(
                self.id,
                &hit_circle,
                sys::b2DefaultQueryFilter(),
                Some(overlap_trampoline::<OverlapFn>),
                &mut overlap as *mut _ as *mut _,
            )
        };

        // safety: we know that `OverlapStats` and `b2TreeStats` are the exact
        // same bit-representation as per our static_assertions
        unsafe { std::mem::transmute::<sys::b2TreeStats, OverlapStats>(performance_stats) }
    }

    /// Get contact events for this current time step.
    pub fn contact_events(&self) -> impl Iterator<Item = (BodyId, BodyId)> {
        unsafe {
            let contact_events = sys::b2World_GetContactEvents(self.id);
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

/// These are performance results returned by dynamic tree queries."]
#[derive(Debug, Copy, Clone)]
#[repr(C)]
pub struct OverlapStats {
    /// Number of internal nodes visited during the query"]
    pub node_visits: i32,

    /// Number of leaf nodes visited during the query"]
    pub leaf_visits: i32,
}

// glam and b2Vec2 are the same thing (two f32s):
static_assertions::assert_eq_size!(sys::b2TreeStats, OverlapStats);
static_assertions::assert_eq_align!(sys::b2TreeStats, OverlapStats);
static_assertions::const_assert_eq!(
    std::mem::offset_of!(sys::b2TreeStats, nodeVisits),
    std::mem::offset_of!(OverlapStats, node_visits)
);
static_assertions::const_assert_eq!(
    std::mem::offset_of!(sys::b2TreeStats, leafVisits),
    std::mem::offset_of!(OverlapStats, leaf_visits)
);
