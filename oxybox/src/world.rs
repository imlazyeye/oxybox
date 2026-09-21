use std::{
    cell::Cell,
    marker::PhantomData,
    sync::{Mutex, MutexGuard},
};

use glam::Vec2;

mod overlap_stats;
mod query_filter;
mod world_definition;

pub use overlap_stats::OverlapStats;
pub use query_filter::QueryFilter;
pub use world_definition::{MixingCallbacks, TaskSystem, WorldDefinition};

use crate::{
    Body, BodyId, InvalidJointBody, Joint, JointId, MouseJoint, MouseJointDefinition, Shape, ShapeId, ShapeRef,
};

/// Box2D keeps every world in one global array and claims slots without synchronization:
/// `b2CreateWorld` scans for the first entry with `inUse == false` and sets it, and
/// `b2DestroyWorld` clears it. Two threads doing that at once can claim the same slot, and one
/// world is then silently reinitialized underneath the other. Every world creation and
/// destruction holds this lock.
static WORLD_LOCK: Mutex<()> = Mutex::new(());

pub(crate) fn world_lock() -> MutexGuard<'static, ()> {
    // the lock guards no data, so a poisoned lock has nothing broken to report -- and this is
    // taken in `World::drop`, which must not panic.
    WORLD_LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A physics world.
///
/// A world contains bodies, shapes, and constraints. You may create up to 128
/// worlds. Each world is completely independent and may be simulated in parallel.
///
/// # Thread safety
///
/// A `World` is [`Send`] but not [`Sync`]: one may be moved to another thread and simulated
/// there, and two worlds may be stepped in parallel, but a single world must only ever be touched
/// by one thread at a time. Worlds may be created and dropped from any number of threads at once.
///
/// We allocate a global mutex which we use to sync World creation, since Box2D stores Worlds in its own
/// global array. This prevents making two Worlds at once on two different threads, which may both be assigned
/// to the same location in memory, leading to UB later. This mutex is only accessed on World creation and
/// when World drops, so if you only make one World and only drop it at the end of the program's life, then
/// this mutex will not be a serious concern.
#[derive(Debug)]
pub struct World {
    pub(crate) id: sys::b2WorldId,

    /// Box2D does not synchronize access to a world, and every method here takes `&self`, so two
    /// threads sharing a `&World` could race inside Box2D. This makes `World` `!Sync` while
    /// leaving it `Send`, since moving a whole world between threads is fine.
    not_sync: PhantomData<Cell<()>>,
}

impl World {
    /// The default number of sub_steps to do in [`World::step`].
    pub const SUB_STEPS: u32 = 4;

    /// Create a world for rigid body simulation.
    ///
    /// # Panics
    ///
    /// Panics if more than 128 worlds exist at once. You can use [`World::try_new`] to handle
    /// this panic manually.
    pub fn new(world_definition: WorldDefinition) -> Self {
        Self::try_new(world_definition).unwrap()
    }

    /// Create a world for rigid body simulation.
    pub fn try_new(world_definition: WorldDefinition) -> Result<Self, TooManyWorlds> {
        let _guard = world_lock();

        // safety: `WorldDefinition` is laid out exactly like `b2WorldDef`
        let id = unsafe { sys::b2CreateWorld(world_definition.as_b2()) };

        let is_valid = unsafe { sys::b2World_IsValid(id) };
        if is_valid {
            Ok(Self {
                id,
                not_sync: PhantomData,
            })
        } else {
            Err(TooManyWorlds)
        }
    }

    /// The raw id of the world.
    pub fn id(&self) -> sys::b2WorldId {
        self.id
    }

    /// Simulate a world for one time step.
    /// This performs collision detection, integration, and constraint solution.
    ///
    /// `delta_time` is the amount of time to simulate. This should be a fixed number, usually
    /// `1.0 / 60.0` -- a varying time step will make the simulation non-deterministic and can
    /// hurt stability.
    ///
    /// `sub_steps`: Increasing the sub-step count can increase accuracy. Usually [`World::SUB_STEPS`]
    pub fn step(&mut self, delta_time: f32, sub_steps: u32) {
        unsafe {
            sys::b2World_Step(self.id, delta_time, sub_steps as i32);
        }
    }

    /// Set the gravity vector for the entire world. Box2D has no concept of an up direction and
    /// this is left as a decision for the application. Usually in m/s^2.
    pub fn set_gravity(&self, gravity: Vec2) {
        unsafe { sys::b2World_SetGravity(self.id, gravity.into()) }
    }

    /// Create a rigid body given a definition.
    pub fn create_body(&self, body_definition: crate::BodyDefinition) -> Body<'_> {
        // safety: `BodyDefinition` is laid out exactly like `b2BodyDef` (checked at compile time
        // where it is defined), so Box2D can read it in place -- nothing is copied or converted.
        let body_id = unsafe { sys::b2CreateBody(self.id, body_definition.as_b2()) };

        Body::new(BodyId::from_b2(body_id))
    }

    /// Create a mouse joint given a definition.
    pub fn create_mouse_joint(
        &self,
        mouse_joint_definition: MouseJointDefinition,
    ) -> Result<MouseJoint<'_>, InvalidJointBody> {
        // Box2D only checks the bodies in a debug assertion, and indexes its body array with them
        // regardless, so a stale or foreign id would read out of bounds.
        if !self.owns_body(mouse_joint_definition.body_id_a) {
            return Err(InvalidJointBody::BodyA);
        }
        if !self.owns_body(mouse_joint_definition.body_id_b) {
            return Err(InvalidJointBody::BodyB);
        }

        // safety: `MouseJointDefinition` is laid out exactly like `b2MouseJointDef` (checked at
        // compile time where it is defined), and both of its bodies were just checked.
        let joint_id = unsafe { sys::b2CreateMouseJoint(self.id, mouse_joint_definition.as_b2()) };

        Ok(MouseJoint(Joint::new(JointId::from_b2(joint_id))))
    }

    /// Overlap test for circles.
    ///
    /// The callback will be called for each shape which overlaps with the provided circle. If the callback
    /// returns `Some(r)`, then we will stop iterating early and return `r`.
    ///
    /// Only shapes which pass `filter` are considered -- see [`QueryFilter`].
    /// If query stats are desired, call [`World::overlap_circle_with_stats`].
    pub fn overlap_circle<OverlapFn, R>(
        &mut self,
        circle_position: Vec2,
        radius: f32,
        filter: QueryFilter,
        overlap: OverlapFn,
    ) -> Option<R>
    where
        OverlapFn: FnMut(ShapeRef<'_>) -> Option<R>,
    {
        self.overlap_circle_with_stats(circle_position, radius, filter, overlap)
            .1
    }

    /// Overlap test for circles.
    ///
    /// The callback will be called for each shape which overlaps with the provided circle. If the callback
    /// returns `Some(r)`, then we will stop iterating early and return `r` alongside the query stats.
    ///
    /// Only shapes which pass `filter` are considered -- see [`QueryFilter`].
    pub fn overlap_circle_with_stats<OverlapFn, R>(
        &mut self,
        circle_position: Vec2,
        radius: f32,
        filter: QueryFilter,
        overlap: OverlapFn,
    ) -> (OverlapStats, Option<R>)
    where
        OverlapFn: FnMut(ShapeRef<'_>) -> Option<R>,
    {
        // safety: we are copying all data and we know that glam::Vec2 is the exact same as b2Vec2 so we
        // can make a pointer to it. Additionally, it survives this function entirely.
        let hit_circle = unsafe { sys::b2MakeProxy(&circle_position as *const Vec2 as *const sys::b2Vec2, 1, radius) };

        struct OverlapCtx<OverlapFn, R> {
            overlap: OverlapFn,
            result: Option<R>,
        }

        let mut ctx = OverlapCtx { overlap, result: None };

        extern "C" fn overlap_trampoline<OverlapFn, R>(shape: sys::b2ShapeId, cback: *mut std::ffi::c_void) -> bool
        where
            OverlapFn: FnMut(ShapeRef<'_>) -> Option<R>,
        {
            // safety: Rust's type system promises that this is the same type of context
            // which we are passing. We *are* passing this context as an `&mut OverlapCtx<OverlapFn, R>`
            // when we call `sys::b2World_OverlapShape`
            let ctx: &mut OverlapCtx<OverlapFn, R> = unsafe { &mut *(cback as *mut OverlapCtx<OverlapFn, R>) };

            // Box2D only hands the callback shapes it just found in the tree, so this one is live,
            // and the world outlives the query it is running inside of.
            let shape_ref = ShapeRef::new(ShapeId::from_b2(shape));
            match (ctx.overlap)(shape_ref) {
                Some(r) => {
                    ctx.result = Some(r);
                    false
                }
                None => true,
            }
        }

        // safety: the context is owned by us, and we can make a pointer to it, which we can cast to
        // `std::ffi::c_void`, which will get the context back eventually.
        let performance_stats = unsafe {
            sys::b2World_OverlapShape(
                self.id,
                &hit_circle,
                filter.as_b2(),
                Some(overlap_trampoline::<OverlapFn, R>),
                &mut ctx as *mut OverlapCtx<OverlapFn, R> as *mut std::ffi::c_void,
            )
        };

        let stats = OverlapStats {
            node_visits: performance_stats.nodeVisits,
            leaf_visits: performance_stats.leafVisits,
        };

        (stats, ctx.result)
    }

    /// Get contact events for this current time step.
    ///
    /// Note that contact events are opt-in per shape: a shape must be created with
    /// [`ShapeDefinition::enable_contact_events`](crate::ShapeDefinition::enable_contact_events)
    /// set to `true` or it will never appear here. Box2D leaves this off by default.
    pub fn contact_events(&self) -> impl Iterator<Item = (BodyId, BodyId)> + '_ {
        // safety: Box2D hands us its internal event buffer, which lives until the next step. The
        // buffer pointer is null when the world is locked, and a null pointer is not a valid empty
        // slice, so we check for it.
        let begin_events: &[sys::b2ContactBeginTouchEvent] = unsafe {
            let contact_events = sys::b2World_GetContactEvents(self.id);

            if contact_events.beginEvents.is_null() {
                &[]
            } else {
                std::slice::from_raw_parts(contact_events.beginEvents, contact_events.beginCount as usize)
            }
        };

        begin_events.iter().filter_map(|e| unsafe {
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

    /// Gets a given [`Shape`] from an existing [`ShapeId`].
    ///
    /// Returns `None` if the shape has been destroyed, or belongs to a different world.
    pub fn shape(&self, shape_id: ShapeId) -> Option<Shape<'_>> {
        self.owns_shape(shape_id).then(|| Shape::new(shape_id))
    }

    /// Gets a given [`Body`] from an existing [`BodyId`].
    ///
    /// Returns `None` if the body has been destroyed, or belongs to a different world.
    pub fn body(&self, body_id: BodyId) -> Option<Body<'_>> {
        self.owns_body(body_id).then(|| Body::new(body_id))
    }

    /// Gets a given [`Joint`] from an existing [`JointId`].
    ///
    /// Returns `None` if the joint has been destroyed, or belongs to a different world.
    pub fn joint(&self, joint_id: JointId) -> Option<Joint<'_>> {
        self.owns_joint(joint_id).then(|| Joint::new(joint_id))
    }

    /// Destroy a rigid body. This destroys all shapes and joints attached to the body.
    ///
    /// Returns `false` if the body was already destroyed, or belongs to a different world.
    pub fn destroy_body(&mut self, body_id: BodyId) -> bool {
        if !self.owns_body(body_id) {
            return false;
        }

        unsafe { sys::b2DestroyBody(body_id.0) };
        true
    }

    /// Destroy a joint.
    ///
    /// Returns `false` if the joint was already destroyed, or belongs to a different world.
    pub fn destroy_joint(&mut self, joint_id: JointId) -> bool {
        if !self.owns_joint(joint_id) {
            return false;
        }

        unsafe { sys::b2DestroyJoint(joint_id.0) };
        true
    }

    /// Whether `body_id` names a live body in *this* world.
    ///
    /// If you make and destroy a world, the old body ids from the past world may overlap
    /// (ie, break the A-B-A problem) from bodies in the new world -- Box2d only exposes world
    /// slot index, but not generation data. If you never or rarely delete worlds, you don't have
    /// to worry about it.
    pub fn owns_body(&self, body_id: BodyId) -> bool {
        self.id.index1.wrapping_sub(1) == body_id.0.world0 && body_id.is_valid()
    }

    /// Whether `shape_id` names a live shape in *this* world.
    ///
    /// If you make and destroy a world, the old shape ids from the past world may overlap
    /// (ie, break the A-B-A problem) from shapes in the new world -- Box2d only exposes world
    /// slot index, but not generation data. If you never or rarely delete worlds, you don't have
    /// to worry about it.
    pub fn owns_shape(&self, shape_id: ShapeId) -> bool {
        self.id.index1.wrapping_sub(1) == shape_id.0.world0 && shape_id.is_valid()
    }

    /// Whether `joint_id` names a live joint in *this* world.
    ///
    /// If you make and destroy a world, the old joint ids from the past world may overlap
    /// (ie, break the A-B-A problem) from joints in the new world -- Box2d only exposes world
    /// slot index, but not generation data. If you never or rarely delete worlds, you don't have
    /// to worry about it.
    pub fn owns_joint(&self, joint_id: JointId) -> bool {
        self.id.index1.wrapping_sub(1) == joint_id.0.world0 && joint_id.is_valid()
    }
}

impl Drop for World {
    fn drop(&mut self) {
        // Box2D frees the world's slot here, which would race another thread claiming one
        let _guard = world_lock();

        unsafe { sys::b2DestroyWorld(self.id) }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new(WorldDefinition::default())
    }
}

#[derive(Debug, thiserror::Error)]
#[error("could not make new world; only 128 may exist at once")]
pub struct TooManyWorlds;

#[cfg(test)]
mod tests {
    use glam::Vec2;

    use crate::*;

    const DT: f32 = 1.0 / 60.0;

    #[test]
    fn overlap_circle_respects_the_query_filter() {
        const CATEGORY: u64 = 0b10;

        let mut world = World::new(WorldDefinition::new());

        let body = world.create_body(BodyDefinition::new());
        let shape = body
            .attach_circle(
                Vec2::ZERO,
                1.0,
                ShapeDefinition {
                    filter: Filter {
                        category_bits: CATEGORY,
                        mask_bits: CATEGORY,
                        ..Filter::new()
                    },
                    ..ShapeDefinition::new()
                },
            )
            .id();
        world.step(DT, World::SUB_STEPS);

        // the default query has a category of 1, which this shape's mask excludes, so it must not
        // be reported even though it plainly overlaps
        let hit = world.overlap_circle(Vec2::ZERO, 1.0, QueryFilter::default(), |_| Some(()));
        assert_eq!(hit, None, "shape was reported despite a non-matching filter");

        // a query that the shape's mask accepts finds it
        let filter = QueryFilter {
            category_bits: CATEGORY,
            mask_bits: CATEGORY,
        };
        let hit = world.overlap_circle(Vec2::ZERO, 1.0, filter, |s| (s.id() == shape).then_some(()));
        assert_eq!(hit, Some(()), "shape was not reported despite a matching filter");
    }

    #[test]
    fn an_overlap_callback_can_read_what_it_is_handed() {
        const USER_DATA: usize = 0xBEEF;
        const BODY_POSITION: Vec2 = Vec2::new(3.0, 4.0);

        let mut world = World::new(WorldDefinition::new());

        let body = world.create_body(BodyDefinition {
            position: BODY_POSITION,
            ..BodyDefinition::new()
        });
        let shape = body.attach_circle(Vec2::ZERO, 2.0, ShapeDefinition::new());
        shape.set_user_data(USER_DATA);

        let body_id = body.id();
        let shape_id = shape.id();
        world.step(DT, World::SUB_STEPS);

        let found = world.overlap_circle(BODY_POSITION, 1.0, QueryFilter::default(), |s| {
            Some((s.id(), s.user_data(), s.dimensions(), s.body_id(), s.body().position()))
        });

        let (id, user_data, dimensions, hit_body, position) = found.expect("the shape overlaps the query circle");
        assert_eq!(id, shape_id);
        assert_eq!(user_data, USER_DATA);
        assert_eq!(dimensions, Vec2::new(4.0, 4.0), "a circle of radius 2 measures 4x4");
        assert_eq!(hit_body, body_id);
        assert_eq!(position, BODY_POSITION);
    }

    #[test]
    fn contact_events_are_empty_before_stepping() {
        let world = World::new(WorldDefinition::new());

        assert_eq!(world.contact_events().count(), 0);
    }

    #[test]
    fn ids_from_another_world_are_rejected() {
        let owner = World::new(WorldDefinition::new());
        let mut other = World::new(WorldDefinition::new());

        let body = owner.create_body(BodyDefinition::new());
        let shape_id = body.attach_circle(Vec2::ZERO, 1.0, ShapeDefinition::new()).id();

        let body_id = body.id();

        // the id is perfectly valid -- it just does not name anything in `other`
        assert!(body_id.is_valid());
        assert!(shape_id.is_valid());

        assert!(other.body(body_id).is_none(), "a foreign body id was accepted");
        assert!(other.shape(shape_id).is_none(), "a foreign shape id was accepted");
        assert!(!other.destroy_body(body_id), "a foreign body was destroyed");

        // and the owning world still hands them out
        assert!(owner.body(body_id).is_some());
        assert!(owner.shape(shape_id).is_some());
    }

    #[test]
    fn destroying_a_body_invalidates_its_handles() {
        let mut world = World::new(WorldDefinition::new());

        let body = world.create_body(BodyDefinition::new());
        let shape_id = body.attach_circle(Vec2::ZERO, 1.0, ShapeDefinition::new()).id();
        let body_id = body.id();

        assert!(world.destroy_body(body_id), "the body should have been destroyed");

        // destroying a body takes its shapes with it, and neither handle is handed out again
        assert!(world.body(body_id).is_none());
        assert!(world.shape(shape_id).is_none());

        // a second destroy is a no-op rather than a double free
        assert!(!world.destroy_body(body_id));
    }
}
