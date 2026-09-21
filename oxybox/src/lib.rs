mod body;
mod joint;
mod render;
mod rotation;
mod shape;
mod world;

pub use body::{Body, BodyDefinition, BodyId, BodyKind, BodyName, BodyRef};
pub use joint::{InvalidJointBody, Joint, JointId, JointKind, JointRef, MouseJoint, MouseJointDefinition};
pub use render::{CircleDraw, DrawShapeCommand, PolygonDraw};
pub use rotation::Rotation;
pub use shape::*;
pub use world::{OverlapStats, QueryFilter, TooManyWorlds, World, WorldDefinition};

/// Handles and placeholders which have not been fully implemented. These are available
/// if you need to name the type for some reason but access and definition is unstable.
pub mod opaque {
    pub use super::world::{MixingCallbacks, TaskSystem};

    /// The cookie Box2D stamps into a definition so that it can reject one you never initialized.
    ///
    /// This cannot be constructed manually.
    #[repr(C)]
    #[derive(Debug, Clone, Copy)]
    pub struct Internal {
        internal_value: std::os::raw::c_int,
    }
}

pub use sys;

/// Sets the length units per meter Box2D will expect. While you're free to work with whatever units
/// you please, setting this will help Box2D tweak internal numbers to better work with your
/// expectations. For example, if your player character is 32 pixels high, then pass `32.0`, and you
/// may then confidently use pixels for all the length values you send to Box2D.
///
/// **NOTE: This is a global value -- Box2D does not support different unit lengths per-world.**
///
/// **WARNING: This should be set before any other call into Box2D. That includes
/// [`WorldDefinition::new`] and [`BodyDefinition::new`], whose defaults are scaled by this value,
/// not just [`World::new`].**
pub fn set_length_units_per_meter(length_units: f32) {
    let _guard = world::world_lock();
    unsafe { sys::b2SetLengthUnitsPerMeter(length_units) }
}

/// Get the current length units per meter. Defaults to `1.0`.
pub fn length_units_per_meter() -> f32 {
    unsafe { sys::b2GetLengthUnitsPerMeter() }
}

/// Compile-time checks that a Rust mirror struct matches the `b2*` struct it is handed to Box2D as.
/// Asserts that `$rust_ty` has the same size, alignment, and field offsets as `$c_ty`.
///
/// Normal fields map as `rust_name => c_name`. An opaque placeholder standing in for a run of C
/// fields names its type and the C field that follows the run, as in
/// `mixing_callbacks: MixingCallbacks => frictionCallback .. enableSleep`.
macro_rules! mirrors_layout {
    ($rust_ty:ty => $c_ty:ty { $($fields:tt)* }) => {
        const _: () = {
            assert!(::std::mem::size_of::<$rust_ty>() == ::std::mem::size_of::<$c_ty>());
            assert!(::std::mem::align_of::<$rust_ty>() == ::std::mem::align_of::<$c_ty>());
            $crate::mirrors_layout!(@fields $rust_ty, $c_ty, $($fields)*);
        };
    };

    (@fields $rust_ty:ty, $c_ty:ty,) => {};

    // an opaque placeholder standing in for the C fields in `$c_field .. $c_end`
    (@fields $rust_ty:ty, $c_ty:ty, $field:ident : $blob:ty => $c_field:ident .. $c_end:ident, $($rest:tt)*) => {
        assert!(::std::mem::offset_of!($rust_ty, $field) == ::std::mem::offset_of!($c_ty, $c_field));
        assert!(
            ::std::mem::size_of::<$blob>()
                == ::std::mem::offset_of!($c_ty, $c_end) - ::std::mem::offset_of!($c_ty, $c_field)
        );
        $crate::mirrors_layout!(@fields $rust_ty, $c_ty, $($rest)*);
    };

    (@fields $rust_ty:ty, $c_ty:ty, $field:ident => $c_field:ident, $($rest:tt)*) => {
        assert!(::std::mem::offset_of!($rust_ty, $field) == ::std::mem::offset_of!($c_ty, $c_field));
        $crate::mirrors_layout!(@fields $rust_ty, $c_ty, $($rest)*);
    };
}

pub(crate) use mirrors_layout;
