use bitflags::bitflags;
use glam::Vec2;
use std::ffi::c_void;

use crate::World;

#[derive(Debug, Clone)]
pub enum Draw {
    Circle {
        center: Vec2,
        radius: f32,
        /// These colors are used for debug draw and mostly match the named SVG colors.
        color: u32,
        filled: bool,
    },
    Rect {
        center: Vec2,
        size: Vec2,
        rotation: f32,
        /// These colors are used for debug draw and mostly match the named SVG colors.
        color: u32,
        filled: bool,
    },
    Segment {
        p1: Vec2,
        p2: Vec2,
        /// These colors are used for debug draw and mostly match the named SVG colors.
        color: u32,
    },
    Transform {
        position: Vec2,
        rotation: f32,
    },
    Point {
        position: Vec2,
        size: f32,
        /// These colors are used for debug draw and mostly match the named SVG colors.
        color: u32,
    },
    String {
        position: Vec2,
        text: String,
        /// These colors are used for debug draw and mostly match the named SVG colors.
        color: u32,
    },
    Capsule {
        center: Vec2,
        half_height: f32,
        radius: f32,
        rotation: f32,
        /// These colors are used for debug draw and mostly match the named SVG colors.
        color: u32,
        filled: bool,
    },
}

bitflags! {
    pub struct TeselationFlags: u32 {
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
    pub fn gather_draws(&self, flags: TeselationFlags) -> Vec<Draw> {
        let mut calls = Vec::new();

        let mut dd = sys::b2DebugDraw {
            DrawCircleFcn: Some(draw_circle_cb),
            DrawSolidCircleFcn: Some(draw_solid_circle_cb),
            DrawPolygonFcn: Some(draw_polygon_cb),
            DrawSolidPolygonFcn: Some(draw_solid_polygon_cb),

            // not yet supported
            DrawSolidCapsuleFcn: None,
            DrawSegmentFcn: None,
            DrawTransformFcn: None,
            DrawPointFcn: None,
            DrawStringFcn: None,

            drawShapes: flags.contains(TeselationFlags::SHAPES),

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

extern "C" fn draw_circle_cb(center: sys::b2Vec2, radius: f32, color: sys::b2HexColor, ctx: *mut c_void) {
    let calls = unsafe { &mut *(ctx as *mut Vec<Draw>) };
    calls.push(Draw::Circle {
        center: Vec2::new(center.x, center.y),
        radius,
        color,
        filled: false,
    });
}

extern "C" fn draw_solid_circle_cb(transform: sys::b2Transform, radius: f32, color: sys::b2HexColor, ctx: *mut c_void) {
    let calls = unsafe { &mut *(ctx as *mut Vec<Draw>) };
    let center = transform.p;
    calls.push(Draw::Circle {
        center: Vec2::new(center.x, center.y),
        radius,
        color,
        filled: true,
    });
}

// Helper to extract rectangle from polygon (assumes 4 points forming a rectangle)
fn polygon_to_rect(verts: &[sys::b2Vec2], transform: Option<sys::b2Transform>) -> Option<(Vec2, Vec2, f32)> {
    if verts.len() != 4 {
        return None;
    }

    // If we have transform information, use it (this is the correct approach)
    if let Some(transform) = transform {
        let center = Vec2::new(transform.p.x, transform.p.y);
        let rotation = transform.q.s.atan2(transform.q.c);

        // Calculate the rectangle dimensions from the vertices
        // The vertices are in local space, so we need to find the extents
        let mut min_x = f32::INFINITY;
        let mut max_x = f32::NEG_INFINITY;
        let mut min_y = f32::INFINITY;
        let mut max_y = f32::NEG_INFINITY;

        for vert in verts {
            min_x = min_x.min(vert.x);
            max_x = max_x.max(vert.x);
            min_y = min_y.min(vert.y);
            max_y = max_y.max(vert.y);
        }

        let size = Vec2::new(max_x - min_x, max_y - min_y);
        return Some((center, size, rotation));
    }

    // Fallback: calculate from vertices (for outline polygons without transform)
    // For a rectangle, vertices should be in order around the perimeter
    let v0 = Vec2::new(verts[0].x, verts[0].y);
    let v1 = Vec2::new(verts[1].x, verts[1].y);
    let v2 = Vec2::new(verts[2].x, verts[2].y);
    let v3 = Vec2::new(verts[3].x, verts[3].y);

    // Calculate center as average of all vertices
    let center = (v0 + v1 + v2 + v3) * 0.25;

    // Calculate the two edge vectors
    let edge1 = v1 - v0; // First edge
    let edge2 = v3 - v0; // Adjacent edge

    // The rectangle dimensions are the lengths of these edges
    let width = edge1.length();
    let height = edge2.length();

    // Rotation is the angle of the first edge
    let rotation = edge1.y.atan2(edge1.x);

    Some((center, Vec2::new(width, height), rotation))
}

extern "C" fn draw_polygon_cb(verts: *const sys::b2Vec2, count: i32, color: sys::b2HexColor, ctx: *mut c_void) {
    let calls = unsafe { &mut *(ctx as *mut Vec<Draw>) };
    let verts_slice = unsafe { std::slice::from_raw_parts(verts, count as usize) };
    if let Some((center, size, rotation)) = polygon_to_rect(verts_slice, None) {
        calls.push(Draw::Rect {
            center,
            size,
            rotation,
            color,
            filled: false,
        });
    }
}

unsafe extern "C" fn draw_solid_polygon_cb(
    transform: sys::b2Transform,
    verts: *const sys::b2Vec2,
    count: i32,
    _radius: f32,
    color: sys::b2HexColor,
    ctx: *mut c_void,
) {
    let calls = unsafe { &mut *(ctx as *mut Vec<Draw>) };
    let verts_slice = unsafe { std::slice::from_raw_parts(verts, count as usize) };
    if let Some((center, size, rotation)) = polygon_to_rect(verts_slice, Some(transform)) {
        calls.push(Draw::Rect {
            center,
            size,
            rotation,
            color,
            filled: true,
        });
    }
}
