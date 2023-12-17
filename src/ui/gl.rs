use glow::*;

pub struct UniformLocations {
    pub dims: UniformLocation,
    pub background_colour: UniformLocation,
    pub border_colour: UniformLocation,
    pub border_width: UniformLocation,
}

impl UniformLocations {
    pub fn get(gl: &glow::Context, program: &NativeProgram) -> Self {
        unsafe {
            Self {
                dims: gl.get_uniform_location(*program, "dims").unwrap(),
                background_colour: gl.get_uniform_location(*program, "background_col").unwrap(),
                border_colour: gl.get_uniform_location(*program, "border_col").unwrap(),
                border_width: gl.get_uniform_location(*program, "border_width").unwrap(),
            }
        }
    }
}
