/// Joint id references a joint instance. This should be treated as an opaque handle.
///
/// You get a `JointId` from [`JointRef::id`](crate::JointRef::id). To do anything with one, hand it
/// back to the world it came from -- see [`World::joint`](crate::World::joint), which re-checks that
/// the id names a live joint in that world before giving a [`Joint`](crate::Joint) handle.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct JointId(pub(crate) sys::b2JointId);

impl JointId {
    /// Creates a [`JointId`] from a [`sys::b2JointId`].
    pub fn from_b2(input: sys::b2JointId) -> Self {
        Self(input)
    }

    /// Joint identifier validation. Provides validation for up to 64K allocations.
    pub fn is_valid(self) -> bool {
        unsafe { sys::b2Joint_IsValid(self.0) }
    }
}

impl From<sys::b2JointId> for JointId {
    fn from(value: sys::b2JointId) -> Self {
        Self::from_b2(value)
    }
}

impl From<JointId> for sys::b2JointId {
    fn from(value: JointId) -> Self {
        value.0
    }
}

impl std::fmt::Debug for JointId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.pad(&format!("{}@{}v{}", self.0.world0, self.0.index1, self.0.generation))
    }
}
