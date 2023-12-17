use global::Globals;
use glow::*;
use midi::*;
use ui::{*, style::Style};

mod ui;
mod piano_roll;
mod global;
mod midi;

fn main() {
    unsafe {
        let (gl, window, mut events_loop, _context) = create_sdl2_context();

        let program = create_program(&gl, VERTEX_SHADER_SOURCE, FRAGMENT_SHADER_SOURCE);
        gl.use_program(Some(program));

        let midi = MidiClip { notes: vec![
            Note { note: 60, velocity: 100, start: 0, length: 100 },
            Note { note: 61, velocity: 100, start: 100, length: 100 },
            Note { note: 62, velocity: 100, start: 500, length: 100 },
            Note { note: 63, velocity: 100, start: 0, length: 100 },
        ] };

        let globals = Globals::create(&gl, program);

        let root = piano_roll::e_piano_roll(&gl, &globals, &midi);

        gl.clear_color(0.1, 0.2, 0.3, 1.0);
        let window_size = gl.get_uniform_location(program, "window_size");

        let mut style = Style::default();
        style.background_colour.r = 1.;
        let fuckyou = ui::element::Element::new(
            &gl,
            0.,
            0.,
            Size::Fixed(10.),
            Size::Fixed(10.),
            Some(style),
            vec![],
        );

        'render: loop {
            {
                for event in events_loop.poll_iter() {
                    if let sdl2::event::Event::Quit { .. } = event {
                        break 'render;
                    }
                }
            }

            let (width, height) = window.drawable_size();
            gl.viewport(0, 0, width as i32, height as i32);
            gl.uniform_2_f32(window_size.as_ref(), width as f32, height as f32);

            let screen_dims = ComputedDimensions {
                width: width as f32,
                height: height as f32,
            };

            gl.clear(glow::COLOR_BUFFER_BIT);

            root.render(&gl, p(0., 0.), &globals, &screen_dims);
            window.gl_swap_window();
        }

        root.cleanup(&gl);

        gl.delete_program(program);
    }
}

unsafe fn create_sdl2_context() -> (
    glow::Context,
    sdl2::video::Window,
    sdl2::EventPump,
    sdl2::video::GLContext,
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

    (gl, window, event_loop, gl_context)
}

pub struct Zoom {
    h_zoom: f32,
    v_zoom: f32,
}

unsafe fn create_program(
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

const VERTEX_SHADER_SOURCE: &str = r#"#version 330
  in vec4 in_position;
  out vec2 position;
  out vec2 uv;
  uniform vec2 window_size;
  void main() {
    position = ((in_position.xy) / window_size) * 2.0 - 1.0;
    uv = in_position.zw;

    gl_Position = vec4(position, 1.0, 1.0);
  }"#;
const FRAGMENT_SHADER_SOURCE: &str = r#"#version 330
  precision mediump float;
  in vec2 position;
  in vec2 uv;
  out vec4 color;
  uniform vec2 dims;
  uniform vec4 background_col;
  uniform vec4 border_col;
  uniform float border_width;
  void main() {
    color = background_col;

    vec2 pos = uv * dims;

    if (border_width > 0.0
        && (pos.x < border_width
        || pos.x > dims.x - border_width
        || pos.y < border_width
        || pos.y > dims.y - border_width)) {

      color = border_col;
    }
  }"#;
