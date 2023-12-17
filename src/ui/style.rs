use glow::*;

use crate::global::Globals;

pub struct Style {
    pub visible: bool,
    pub render_self: bool,
    pub background_colour: Color,
    pub border_colour: Color,
    pub border_width: f32,
}

pub struct ColorPalette {
    pub black: Color,
    pub white: Color,
    pub black_key: Color,
    pub white_key: Color,
    pub black_key_piano_roll_row: Color,
    pub white_key_piano_roll_row: Color,
    pub time_grid: Color,
    pub text: Color,
}

impl Default for ColorPalette {
    #[rustfmt::skip]
    fn default() -> Self {
        ColorPalette {
            black: Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
            white: Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            black_key: Color { r: 0.1, g: 0.1, b: 0.1, a: 1.0 },
            white_key: Color { r: 0.6, g: 0.6, b: 0.6, a: 1.0 },
            black_key_piano_roll_row: Color { r: 0.3, g: 0.3, b: 0.3, a: 1.0 },
            white_key_piano_roll_row: Color { r: 0.45, g: 0.45, b: 0.45, a: 1.0 },
            time_grid: Color { r: 0.2, g: 0.2, b: 0.2, a: 1.0 },
            text: Color { r: 0.8, g: 0.8, b: 0.8, a: 1.0 },
        }
    }
}

impl Default for Style {
    fn default() -> Self {
        Self {
            visible: true,
            render_self: true,
            background_colour: BLACK,
            border_colour: BLACK,
            border_width: 0.,
        }
    }
}

impl Style {
    pub fn set(&self, gl: &glow::Context, globals: &Globals) {
        unsafe {
            self.background_colour
                .set_uniform(gl, &globals.uniform_locations.background_colour);

            self.border_colour
                .set_uniform(gl, &globals.uniform_locations.border_colour);

            gl.uniform_1_f32(
                Some(&globals.uniform_locations.border_width),
                self.border_width,
            );
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

pub static BLACK: Color = Color {
    r: 0.,
    g: 0.,
    b: 0.,
    a: 1.,
};

impl Color {
    unsafe fn set_uniform(&self, gl: &glow::Context, location: &UniformLocation) {
        gl.uniform_4_f32(Some(location), self.r, self.g, self.b, self.a);
    }
}