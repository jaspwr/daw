use glow::*;

use crate::ui::gl::*;
use crate::ui::style::*;

pub struct Globals {
    pub uniform_locations: UniformLocations,
    pub colour_palette: ColorPalette,
    pub shader: NativeProgram,
}

impl Globals {
    pub fn create(gl: &Context, program: NativeProgram) -> Self {
        Globals {
            uniform_locations: UniformLocations::get(&gl, &program),
            colour_palette: ColorPalette::default(),
            shader: program,
        }
    }
}