/// Body id references a body instance. This should be treated as an opaque handle.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct BodyId(sys::b2BodyId);

impl BodyId {
    /// Creates a [`BodyId`] from a [`sys::b2BodyId`].
    pub fn from_b2(input: sys::b2BodyId) -> Self {
        Self(input)
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

impl std::ops::Deref for BodyId {
    type Target = sys::b2BodyId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Shape id references a shape instance. This should be treated as an opaque handle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct ShapeId(sys::b2ShapeId);

impl ShapeId {
    /// Creates a [`ShapeId`] from a [`sys::b2ShapeId`].
    pub fn from_b2(input: sys::b2ShapeId) -> Self {
        Self(input)
    }
}

impl From<sys::b2ShapeId> for ShapeId {
    fn from(value: sys::b2ShapeId) -> Self {
        Self::from_b2(value)
    }
}

impl From<ShapeId> for sys::b2ShapeId {
    fn from(value: ShapeId) -> Self {
        value.0
    }
}

impl std::ops::Deref for ShapeId {
    type Target = sys::b2ShapeId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// World id references a world instance. This should be treated as an opaque handle.
#[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
pub struct WorldId(sys::b2WorldId);

impl WorldId {
    /// Creates a [`WorldId`] from a [`sys::b2WorldId`].
    pub fn from_b2(input: sys::b2WorldId) -> Self {
        Self(input)
    }
}

impl From<sys::b2WorldId> for WorldId {
    fn from(value: sys::b2WorldId) -> Self {
        Self::from_b2(value)
    }
}

impl From<WorldId> for sys::b2WorldId {
    fn from(value: WorldId) -> Self {
        value.0
    }
}

impl std::ops::Deref for WorldId {
    type Target = sys::b2WorldId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
