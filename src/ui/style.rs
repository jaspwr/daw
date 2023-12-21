use glow::*;
use serde::de;

use crate::global::Globals;

#[derive(Clone, Debug)]
pub struct Style {
    pub visible: bool,
    pub render_self: bool,
    pub background_colour: Colour,
    pub border_colour: Colour,
    pub border_width: f32,
    pub padding: f32,
    pub padding_top: f32,
    pub padding_left: f32,
}

pub struct ColourPalette {
    pub bg_primary: Colour,
    pub text_primary: Colour,
    pub black: Colour,
    pub white: Colour,
    pub black_key: Colour,
    pub white_key: Colour,
    pub black_key_piano_roll_row: Colour,
    pub white_key_piano_roll_row: Colour,
    pub black_key_piano_roll_row_alt: Colour,
    pub white_key_piano_roll_row_alt: Colour,
    pub time_grid: Colour,
    pub selected: Colour,
    pub player_head: Colour,
}

impl Default for ColourPalette {
    #[rustfmt::skip]
    fn default() -> Self {
        ColourPalette {
            bg_primary: c("1a1a1a"),
            text_primary: c("ffffff"),
            black: c("000000"),
            white: Colour { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            black_key: c("222222"),
            white_key: c("dddddd"),
            black_key_piano_roll_row: Colour { r: 0.3, g: 0.3, b: 0.3, a: 1.0 },
            white_key_piano_roll_row: Colour { r: 0.45, g: 0.45, b: 0.45, a: 1.0 },
            black_key_piano_roll_row_alt: Colour { r: 0.25, g: 0.25, b: 0.25, a: 1.0 },
            white_key_piano_roll_row_alt: Colour { r: 0.4, g: 0.4, b: 0.4, a: 1.0 },
            time_grid: c("444444"),
            selected: c("ff0000"),
            player_head: c("ff4444"),
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
            padding: 0.,
            padding_top: 0.,
            padding_left: 0.,
        }
    }
}

impl Style {
    pub fn set(&self, gl: &glow::Context, globals: &Globals) {
        unsafe {
            self.background_colour
                .set_uniform(gl, &globals.element_uniform_locations["background_col"]);

            self.border_colour
                .set_uniform(gl, &globals.element_uniform_locations["border_col"]);

            gl.uniform_1_f32(
                Some(&globals.element_uniform_locations["border_width"]),
                self.border_width,
            );
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Colour {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

#[inline]
pub fn c(col: &str) -> Colour {
    assert!(col.len() == 6);
    let r = u8::from_str_radix(&col[0..2], 16).unwrap() as f32 / 255.;
    let g = u8::from_str_radix(&col[2..4], 16).unwrap() as f32 / 255.;
    let b = u8::from_str_radix(&col[4..6], 16).unwrap() as f32 / 255.;
    Colour { r, g, b, a: 1. }
}

pub static BLACK: Colour = Colour {
    r: 0.,
    g: 0.,
    b: 0.,
    a: 1.,
};

impl Colour {
    unsafe fn set_uniform(&self, gl: &glow::Context, location: &UniformLocation) {
        gl.uniform_4_f32(Some(location), self.r, self.g, self.b, self.a);
    }
}