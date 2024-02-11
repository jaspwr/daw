use glow::*;

use crate::global::Globals;

use super::{ComputedDimensions, Position, ComputedPosition};

pub const RENDER_MODE_SOLID: i32 = 0;
pub const RENDER_MODE_TEXTURE: i32 = 1;

pub unsafe fn create_sdl2_context() -> (
    glow::Context,
    sdl2::video::Window,
    sdl2::EventPump,
    sdl2::video::GLContext,
    sdl2::Sdl,
) {
    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let gl_attr = video.gl_attr();
    gl_attr.set_context_profile(sdl2::video::GLProfile::Core);
    gl_attr.set_context_version(3, 3);
    gl_attr.set_context_flags().forward_compatible().set();
    let window = video
        .window("Hello triangle!", 1024, 769)
        .opengl()
        .resizable()
        .build()
        .unwrap();
    let gl_context = window.gl_create_context().unwrap();
    let gl = glow::Context::from_loader_function(|s| video.gl_get_proc_address(s) as *const _);
    let event_loop = sdl.event_pump().unwrap();

    (gl, window, event_loop, gl_context, sdl)
}

pub unsafe fn create_program(
    gl: &glow::Context,
    vertex_shader_source: &str,
    fragment_shader_source: &str,
) -> NativeProgram {
    let program = gl.create_program().expect("Cannot create program");

    let shader_sources = [
        (glow::VERTEX_SHADER, vertex_shader_source),
        (glow::FRAGMENT_SHADER, fragment_shader_source),
    ];

    let mut shaders = Vec::with_capacity(shader_sources.len());

    for (shader_type, shader_source) in shader_sources.iter() {
        let shader = gl
            .create_shader(*shader_type)
            .expect("Cannot create shader");
        gl.shader_source(shader, shader_source);
        gl.compile_shader(shader);
        if !gl.get_shader_compile_status(shader) {
            panic!("{}", gl.get_shader_info_log(shader));
        }
        gl.attach_shader(program, shader);
        shaders.push(shader);
    }

    gl.link_program(program);
    if !gl.get_program_link_status(program) {
        panic!("{}", gl.get_program_info_log(program));
    }

    for shader in shaders {
        gl.detach_shader(program, shader);
        gl.delete_shader(shader);
    }

    program
}

pub unsafe fn create_quad(
    gl: &glow::Context,
    vbo: &NativeBuffer,
    pos: ComputedPosition,
    dims: &ComputedDimensions,
) {
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

    gl.bind_buffer(glow::ARRAY_BUFFER, Some(*vbo));
    gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, quad_vertices_u8, glow::STATIC_DRAW);

    gl.enable_vertex_attrib_array(0);
    const DIMENSIONS: i32 = 4;
    gl.vertex_attrib_pointer_f32(0, DIMENSIONS, glow::FLOAT, false, DIMENSIONS * 4, 0);
}

pub unsafe fn render_textured_quad(
    gl: &Context,
    globals: &Globals,
    quad: &Quad,
    texture: &Option<NativeTexture>,
    pos: ComputedPosition,
    dims: &ComputedDimensions,
) {
    gl.bind_vertex_array(Some((*quad).vao));

    create_quad(gl, &quad.vbo, pos, &dims);

    gl.uniform_1_i32(
        Some(&globals.element_uniform_locations["mode"]),
        RENDER_MODE_TEXTURE,
    );

    gl.bind_texture(glow::TEXTURE_2D, *texture);
    gl.draw_arrays(glow::TRIANGLE_FAN, 0, 4);

    gl.bind_texture(glow::TEXTURE_2D, None);
    gl.disable_vertex_attrib_array(0);
    gl.bind_buffer(glow::ARRAY_BUFFER, None);
    gl.uniform_1_i32(
        Some(&globals.element_uniform_locations["mode"]),
        RENDER_MODE_SOLID,
    );
}

pub struct Quad {
    pub vao: NativeVertexArray,
    pub vbo: NativeBuffer,
}

impl Quad {
    pub unsafe fn new(gl: &Context) -> Self {
        let vao = gl.create_vertex_array().unwrap();
        let vbo = gl.create_buffer().unwrap();

        gl.bind_vertex_array(Some(vao));
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

        gl.enable_vertex_attrib_array(0);
        const DIMENSIONS: i32 = 4;
        gl.vertex_attrib_pointer_f32(0, DIMENSIONS, glow::FLOAT, false, DIMENSIONS * 4, 0);

        gl.bind_vertex_array(None);
        gl.bind_buffer(glow::ARRAY_BUFFER, None);

        Self { vao, vbo }
    }

    pub fn cleanup(&self, gl: &Context) {
        unsafe {
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);
        }
    }
}