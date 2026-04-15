use glam::{Affine2, Vec2};

use crate::World;

/// A draw shape command.
pub enum DrawShapeCommand<'a> {
    Circle(CircleDraw),
    Polygon(PolygonDraw<'a>),
}

pub struct CircleDraw {
    pub transform: Affine2,
    pub radius: f32,
    pub color: u32,
}

pub struct PolygonDraw<'a> {
    pub transform: Affine2,
    pub vertices: &'a [Vec2],
    pub radius: f32,
    pub color: u32,
}

impl World {
    /// Takes a callback and draws all the shapes in the world. This function is very stubbed, and
    /// more work is needed to added full support.
    pub fn draw_shapes<DrawShapesFn>(&self, mut draw_shapes: DrawShapesFn)
    where
        DrawShapesFn: FnMut(DrawShapeCommand<'_>),
    {
        unsafe extern "C" fn draw_solid_circle_wrapper<DrawShapesFn>(
            transform: sys::b2Transform,
            radius: f32,
            color: sys::b2HexColor,
            ctx: *mut std::ffi::c_void,
        ) where
            DrawShapesFn: FnMut(DrawShapeCommand<'_>),
        {
            // we own this pointer and know it is valid:
            let f = unsafe { &mut *(ctx as *mut DrawShapesFn) };

            let (s, c) = (transform.q.s, transform.q.c);
            let transform = glam::Affine2 {
                matrix2: glam::Mat2::from_cols(glam::Vec2::new(c, s), glam::Vec2::new(-s, c)),
                translation: Vec2::new(transform.p.x, transform.p.y),
            };

            f(DrawShapeCommand::Circle(CircleDraw {
                transform,
                radius,
                color,
            }));
        }

        unsafe extern "C" fn draw_solid_polygon_wrapper<DrawShapesFn>(
            transform: sys::b2Transform,
            verts: *const sys::b2Vec2,
            count: i32,
            radius: f32,
            color: sys::b2HexColor,
            ctx: *mut std::ffi::c_void,
        ) where
            DrawShapesFn: FnMut(DrawShapeCommand<'_>),
        {
            // we own this pointer and know it is valid:
            let f = unsafe { &mut *(ctx as *mut DrawShapesFn) };

            // safety: glam::Vec2 is the same as b2Vec2, via static tests in lib.rs
            let verts_slice = unsafe { std::slice::from_raw_parts(verts as *const glam::Vec2, count as usize) };
            let (s, c) = (transform.q.s, transform.q.c);
            let transform = glam::Affine2 {
                matrix2: glam::Mat2::from_cols(glam::Vec2::new(c, s), glam::Vec2::new(-s, c)),
                translation: Vec2::new(transform.p.x, transform.p.y),
            };

            let arg = DrawShapeCommand::Polygon(PolygonDraw {
                transform,
                vertices: verts_slice,
                radius,
                color,
            });

            f(arg);
        }

        let mut dd = sys::b2DebugDraw {
            DrawSolidCircleFcn: Some(draw_solid_circle_wrapper::<DrawShapesFn>),
            DrawSolidPolygonFcn: Some(draw_solid_polygon_wrapper::<DrawShapesFn>),

            // not yet supported
            DrawCircleFcn: None,
            DrawPolygonFcn: None,
            DrawSolidCapsuleFcn: None,
            DrawSegmentFcn: None,
            DrawTransformFcn: None,
            DrawPointFcn: None,
            DrawStringFcn: None,

            drawShapes: true,

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

            // we're uh we're drawing everything:
            drawingBounds: sys::b2AABB {
                lowerBound: sys::b2Vec2 {
                    x: -100000.0,
                    y: -100000.0,
                },
                upperBound: sys::b2Vec2 {
                    x: 100000.0,
                    y: 100000.0,
                },
            },

            context: &mut draw_shapes as *mut _ as *mut _,
            useDrawingBounds: false,
        };

        unsafe {
            sys::b2World_Draw(self.id, &mut dd);
        }
    }
}
