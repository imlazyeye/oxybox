use std::collections::HashMap;

use glam::Vec2;
use oxybox_sys::*;

use crate::{Body, BodyId};

pub struct World {
    id: b2WorldId,
    dt: f32,
    substeps: i32,
    pub(crate) bodies: HashMap<BodyId, Body>,
}

impl Default for World {
    fn default() -> Self {
        unsafe {
            let def = b2DefaultWorldDef();
            let id = b2CreateWorld(&def);
            Self {
                id,
                dt: 1.0 / 60.0,
                substeps: 4,
                bodies: HashMap::new(),
            }
        }
    }
}

impl World {
    pub fn with_delta_time(dt: f32) -> Self {
        Self {
            dt,
            bodies: HashMap::default(),
            ..Default::default()
        }
    }

    pub fn set_gravity(&mut self, gravity: Vec2) {
        unsafe { b2World_SetGravity(self.id, gravity.into()) }
    }

    pub fn set_substeps(&mut self, substeps: i32) {
        self.substeps = substeps;
    }

    pub fn set_dt(&mut self, dt: f32) {
        self.dt = dt;
    }

    /// Sets the pixels-per-meter Box2D will expect. While you're free to work with whatever units
    /// you please, setting this will help Box2D tweak internal numbers to better work with your
    /// expectations.
    ///
    /// **NOTE: This is a global value -- Box2D does not support different unit lengths per-world.**
    pub fn set_ppm(&mut self, ppm: f32) {
        unsafe { b2SetLengthUnitsPerMeter(ppm) }
    }

    pub fn ppm(&self) -> f32 {
        unsafe { b2GetLengthUnitsPerMeter() }
    }

    // todo: this will eventually be removed
    #[inline]
    pub fn raw(&self) -> b2WorldId {
        self.id
    }

    pub fn tick(&mut self) {
        unsafe { b2World_Step(self.id, self.dt, self.substeps) }
    }

    pub fn read(&'_ self, body_id: BodyId) -> Option<BodyReader<'_>> {
        self.bodies.get(&body_id).map(BodyReader)
    }

    pub fn write(&'_ mut self, body_id: BodyId) -> Option<BodyWriter<'_>> {
        self.bodies.get_mut(&body_id).map(BodyWriter)
    }
}

impl Drop for World {
    fn drop(&mut self) {
        unsafe { b2DestroyWorld(self.id) }
    }
}

pub struct BodyReader<'w>(&'w Body);
impl<'w> std::ops::Deref for BodyReader<'w> {
    type Target = Body;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

pub struct BodyWriter<'w>(&'w mut Body);
impl<'w> std::ops::Deref for BodyWriter<'w> {
    type Target = Body;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'w> std::ops::DerefMut for BodyWriter<'w> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}
