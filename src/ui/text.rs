use std::{rc::Rc, cell::RefCell};

use glow::*;
use rusttype::PositionedGlyph;

use crate::global::Globals;

use super::{
    compute_dims,
    gl::{render_textured_quad, Quad},
    style::Colour,
    ComputedDimensions, Position, ComputedPosition,
};

pub struct Text {
    pub text: String,
    pub size: f32,
    pub font: Rc<Font>,
    pub position: Position,
    pub colour: Colour,
    pub needs_rerender: Rc<RefCell<bool>>,
    pub needs_glyphs_rerender: bool,
    texture: Option<NativeTexture>,
    pub dims: ComputedDimensions,
    quad: Quad,
}

impl Text {
    pub fn new(
        gl: &Context,
        text: String,
        size: f32,
        font: &Rc<Font>,
        colour: Colour,
        position: Position,
        needs_rerender: Rc<RefCell<bool>>,
    ) -> Self {
        let mut ret = Self {
            text,
            size,
            position,
            font: font.clone(),
            colour,
            texture: None,
            needs_rerender,
            needs_glyphs_rerender: true,
            quad: unsafe { Quad::new(gl) },
            dims: ComputedDimensions {
                width: 0.0,
                height: 0.0,
            },
        };

        let dims = ComputedDimensions {
            width: 512. as f32,
            height: 512. as f32,
        };


        ret.dims = dims;
        ret.recreate_texture(gl);

        ret
    }

    pub fn rerender_glyphs(&mut self, gl: &glow::Context) {
        // self.recreate_texture(gl);
        let glyphs = self.font.as_ref().layout(self.text.as_str(), self.size);

        for glyph in glyphs {
            let bb = glyph.pixel_bounding_box().unwrap();
            let mut data = vec![0; bb.width() as usize * bb.height() as usize * 4];
            glyph.draw(|x, y, v| {
                let y = bb.height() as u32 - y as u32 - 1;
                let pix_index = (x + y * bb.width() as u32) as usize * 4;
                let alpha = (v * 255.) as u8;
                data[pix_index] = (self.colour.r * 255.) as u8;
                data[pix_index + 1] = (self.colour.g * 255.) as u8;
                data[pix_index + 2] = (self.colour.b * 255.) as u8;
                data[pix_index + 3] = alpha;
            });

            unsafe {
                gl.bind_texture(glow::TEXTURE_2D, self.texture);
                gl.tex_sub_image_2d(
                    glow::TEXTURE_2D,
                    0,
                    bb.min.x as i32,
                    0 as i32,
                    bb.width() as i32,
                    bb.height() as i32,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    PixelUnpackData::Slice(&data),
                );
            }
        }
    }

    pub fn mutate(&mut self, func: Box<dyn Fn(&mut Text)>) {
        let pre_text = self.text.clone();
        func(self);

        // FIXME: This means that if the colour is changed or
        //        something but not the text it won't rerender.

        if self.text == pre_text {
            return;
        }

        self.needs_glyphs_rerender = true;
        self.needs_rerender.replace(true);
    }

    pub fn render(
        &mut self,
        gl: &glow::Context,
        origin: ComputedPosition,
        globals: &Globals,
        parent_dims: &ComputedDimensions,
    ) {
        if self.needs_glyphs_rerender {
            self.needs_glyphs_rerender = false;
            self.rerender_glyphs(gl);
        }

        let pos = origin + self.position.compute(parent_dims);
        unsafe {
            render_textured_quad(gl, globals, &self.quad, &self.texture, pos, &self.dims);
        }
    }

    pub fn cleanup(&self, gl: &Context) {
        unsafe {
            self.quad.cleanup(gl);
            if let Some(texture) = self.texture {
                gl.delete_texture(texture);
            }
        }
    }

    fn recreate_texture(&mut self, gl: &Context) {
        if let Some(texture) = self.texture {
            unsafe {
                gl.delete_texture(texture);
            }
        }

        unsafe {
            let texture = gl.create_texture().unwrap();
            gl.bind_texture(glow::TEXTURE_2D, Some(texture));

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                self.dims.width as i32,
                self.dims.height as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                None,
            );

            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                glow::NEAREST as i32,
            );

            gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                glow::NEAREST as i32,
            );

            gl.bind_texture(glow::TEXTURE_2D, None);

            self.texture = Some(texture);
        }
    }
}

pub struct Font {
    font: rusttype::Font<'static>,
}

impl Font {
    pub fn new() -> Rc<Self> {
        let font_data = include_bytes!("/usr/share/fonts/otf/DankMono-Bold.otf");
        let font = rusttype::Font::try_from_bytes(font_data as &[u8]).unwrap();

        Rc::new(Self { font })
    }

    pub fn layout(&self, text: &str, size: f32) -> Vec<PositionedGlyph<'_>> {
        let scale = rusttype::Scale::uniform(size);
        let v_metrics = self.font.v_metrics(scale);
        let offset = rusttype::point(0.0, v_metrics.ascent);
        self.font.layout(text, scale, offset).collect()
    }
}
