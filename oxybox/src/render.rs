use bitflags::bitflags;
use glam::Vec2;
use std::ffi::c_void;

use crate::World;

#[derive(Debug, Clone)]
pub enum Draw {
    /// Draws a circle.
    Circle {
        /// The center position of the circle.
        center: Vec2,

        /// The radius of the circle.
        radius: f32,

        /// These colors are used for debug draw and mostly match the named SVG colors.
        color: u32,
    },

    Polygon {
        /// The verts to be drawn.
        vertices: Vec<Vec2>,

        /// These colors are used for debug draw and mostly match the named SVG colors.
        color: u32,
    },
}

bitflags! {
    pub struct DrawInstructions: u32 {
        /// Option to draw shapes
        const SHAPES             = 0b0000_0000_0000_0001;
        // const JOINTS             = 0b0000_0000_0000_0010;
        // const JOINT_EXTRAS       = 0b0000_0000_0000_0100;
        // const BOUNDS             = 0b0000_0000_0000_1000;
        // const MASS               = 0b0000_0000_0001_0000;
        // const BODY_NAMES         = 0b0000_0000_0010_0000;
        // const CONTACTS           = 0b0000_0000_0100_0000;
        // const GRAPH_COLORS       = 0b0000_0000_1000_0000;
        // const CONTACT_NORMALS    = 0b0000_0001_0000_0000;
        // const CONTACT_IMPULSES   = 0b0000_0010_0000_0000;
        // const CONTACT_FEATURES   = 0b0000_0100_0000_0000;
        // const FRICTION_IMPULSES  = 0b0000_1000_0000_0000;
        // const ISLANDS            = 0b0001_0000_0000_0000;
    }
}

impl World {
    /// Creates a list of draw commands.
    pub fn gather_draws(&self, flags: DrawInstructions) -> Vec<Draw> {
        let mut calls = Vec::new();

        let mut dd = sys::b2DebugDraw {
            DrawSolidCircleFcn: Some(draw_solid_circle_cb),
            DrawSolidPolygonFcn: Some(draw_solid_polygon_cb),

            // not yet supported
            DrawCircleFcn: None,
            DrawPolygonFcn: None,
            DrawSolidCapsuleFcn: None,
            DrawSegmentFcn: None,
            DrawTransformFcn: None,
            DrawPointFcn: None,
            DrawStringFcn: None,

            drawShapes: flags.contains(DrawInstructions::SHAPES),

            // not yet supported
            drawJoints: false,
            drawJointExtras: false,
            drawBounds: false,
            drawMass: false,
            drawBodyNames: false,
            drawContacts: false,
            drawGraphColors: false,
            drawContactNormals: false,
            drawContactImpulses: false,
            drawContactFeatures: false,
            drawFrictionImpulses: false,
            drawIslands: false,
            drawingBounds: sys::b2AABB {
                lowerBound: sys::b2Vec2 {
                    x: -100000.0,
                    y: -100000.0,
                },
                upperBound: sys::b2Vec2 {
                    x: 100000.0,
                    y: 100000.0,
                }, // todo
            },

            context: &mut calls as *mut _ as *mut c_void,
            useDrawingBounds: false,
        };

        unsafe {
            sys::b2World_Draw(self.id, &mut dd);
        }

        calls
    }
}

extern "C" fn draw_solid_circle_cb(transform: sys::b2Transform, radius: f32, color: sys::b2HexColor, ctx: *mut c_void) {
    let calls = unsafe { &mut *(ctx as *mut Vec<Draw>) };
    let center = transform.p;
    calls.push(Draw::Circle {
        center: Vec2::new(center.x, center.y),
        radius,
        color,
    });
}

unsafe extern "C" fn draw_solid_polygon_cb(
    transform: sys::b2Transform,
    verts: *const sys::b2Vec2,
    count: i32,
    _radius: f32,
    color: sys::b2HexColor,
    calls: *mut c_void,
) {
    let calls = unsafe { &mut *(calls as *mut Vec<Draw>) };
    let verts_slice = unsafe { std::slice::from_raw_parts(verts, count as usize) };

    calls.push(Draw::Polygon {
        vertices: verts_slice
            .iter()
            .map(|vertex| {
                let x = (transform.q.c * vertex.x - transform.q.s * vertex.y) + transform.p.x;
                let y = (transform.q.s * vertex.x + transform.q.c * vertex.y) + transform.p.y;

                Vec2::new(x, y)
            })
            .collect(),
        color,
    });
}

// extern "C" fn draw_circle_cb(center: sys::b2Vec2, radius: f32, color: sys::b2HexColor, ctx: *mut c_void) {
//     let calls = unsafe { &mut *(ctx as *mut Vec<Draw>) };
//     calls.push(Draw::Circle {
//         center: Vec2::new(center.x, center.y),
//         radius,
//         color,
//         filled: false,
//     });
// }

// extern "C" fn draw_polygon_cb(verts: *const sys::b2Vec2, count: i32, color: sys::b2HexColor, ctx: *mut c_void) {
//     let calls = unsafe { &mut *(ctx as *mut Vec<Draw>) };
//     let verts_slice = unsafe { std::slice::from_raw_parts(verts, count as usize) };
//     if let Some((center, size, rotation)) = polygon_to_rect(verts_slice, None) {
//         calls.push(Draw::Rect {
//             center,
//             size,
//             rotation,
//             color,
//             filled: false,
//         });
//     }
// }
