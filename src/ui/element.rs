use glow::*;

use crate::global::Globals;

use super::*;
use super::style::*;

pub struct Element {
    position: Position,
    dimensions: Dimensions,
    children: Vec<Element>,
    style: Style,
    vbo: NativeBuffer,
    vao: NativeVertexArray,
}

impl Element {
    pub fn new(
        gl: &glow::Context,
        x: f32,
        y: f32,
        width: Size,
        height: Size,
        style: Option<Style>,
        children: Vec<Element>,
    ) -> Self {
        let vbo = unsafe { gl.create_buffer().unwrap() };
        let vao = unsafe { gl.create_vertex_array().unwrap() };

        Self {
            position: p(x, y),
            dimensions: d(width, height),
            children,
            style: match style {
                Some(style) => style,
                None => Style::default(),
            },
            vbo,
            vao,
        }
    }

    unsafe fn create_quad(&self, gl: &glow::Context, origin: Position, dims: &ComputedDimensions) {
        let pos = origin + self.position;

        let quad_vertices = [
            pos.x,
            pos.y,
            0.,
            0.,
            pos.x + dims.width,
            pos.y,
            1.,
            0.,
            pos.x + dims.width,
            pos.y + dims.height,
            1.,
            1.,
            pos.x,
            pos.y + dims.height,
            0.,
            1.,
        ];

        let quad_vertices_u8: &[u8] = core::slice::from_raw_parts(
            quad_vertices.as_ptr() as *const u8,
            quad_vertices.len() * core::mem::size_of::<f32>(),
        );

        gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, quad_vertices_u8, glow::STATIC_DRAW);

        gl.enable_vertex_attrib_array(0);
        const DIMENSIONS: i32 = 4;
        gl.vertex_attrib_pointer_f32(0, DIMENSIONS, glow::FLOAT, false, DIMENSIONS * 4, 0);
    }

    pub fn render(
        &self,
        gl: &glow::Context,
        origin: Position,
        globals: &Globals,
        parent_dims: &ComputedDimensions,
    ) {
        if !self.style.visible {
            return;
        }

        let dims = ComputedDimensions {
            width: self.dimensions.width.to_size(parent_dims.width),
            height: self.dimensions.height.to_size(parent_dims.height),
        };

        if self.style.render_self {
            unsafe {
                gl.use_program(Some(globals.shader));
                gl.uniform_2_f32(
                    Some(&(*globals).uniform_locations.dims),
                    dims.width,
                    dims.height,
                );

                gl.bind_vertex_array(Some(self.vao));
                self.create_quad(gl, origin, &dims);

                self.style.set(gl, globals);
                gl.draw_arrays(glow::TRIANGLE_FAN, 0, 4);

                gl.disable_vertex_attrib_array(0);
                gl.bind_buffer(glow::ARRAY_BUFFER, None);
            }
        }

        let child_origin = origin + self.position;

        for child in self.children.iter() {
            child.render(gl, child_origin, globals, &dims);
        }
    }

    pub fn cleanup(&self, gl: &glow::Context) {
        unsafe {
            gl.delete_buffer(self.vbo);
            gl.delete_vertex_array(self.vao);
        }

        for child in self.children.iter() {
            child.cleanup(gl);
        }
    }
}
