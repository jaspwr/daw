use std::{sync::atomic::AtomicUsize, cell::RefCell, rc::Rc};

use crate::global::Globals;

use super::{
    compute_dims,
    element::*,
    gl::{create_quad, render_textured_quad, RENDER_MODE_SOLID, RENDER_MODE_TEXTURE, Quad},
    p, ComputedDimensions, Dimensions, Position, ComputedPosition, BoundingBoxRef, ComputedBoundingBox, reactive::Reactive,
};
use glow::*;

pub struct FrameBuf {
    pub root_node: Option<ElementRef>,
    pub position: Position,
    pub dimensions: Dimensions,
    fbo: NativeFramebuffer,
    // rbo: NativeRenderbuffer,
    texture: Option<NativeTexture>,
    quad: Quad,
    last_size: ComputedDimensions,
    pub bounding_box: BoundingBoxRef,
    pub children_need_rerender: Rc<RefCell<bool>>,
    pub on_cleanup: Vec<Box<dyn Fn()>>,
}

impl FrameBuf {
    pub fn new(
        gl: &Context,
        root_node: Option<ElementRef>,
        pos: Position,
        dims: Dimensions,
        parent_dims: ComputedDimensions,
    ) -> Self {
        let comped_dims = compute_dims(&dims, &parent_dims);

        unsafe {
            let frame_buf = gl.create_framebuffer().unwrap();

            // gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            // let rbo = gl.create_renderbuffer().unwrap();
            // gl.bind_renderbuffer(glow::RENDERBUFFER, Some(rbo));
            // gl.renderbuffer_storage(
            //     glow::RENDERBUFFER,
            //     glow::DEPTH24_STENCIL8,
            //     comped_dims.width as i32,
            //     comped_dims.height as i32,
            // );
            // gl.bind_renderbuffer(glow::RENDERBUFFER, None);

            // gl.framebuffer_renderbuffer(
            //     glow::FRAMEBUFFER,
            //     glow::DEPTH_STENCIL_ATTACHMENT,
            //     glow::RENDERBUFFER,
            //     Some(rbo),
            // );

            if gl.check_framebuffer_status(glow::FRAMEBUFFER) != glow::FRAMEBUFFER_COMPLETE {
                panic!("framebuffer not complete");
            }

            gl.bind_framebuffer(glow::FRAMEBUFFER, None);


            let mut ret = FrameBuf {
                position: pos,
                dimensions: dims,
                root_node,
                fbo: frame_buf,
                // rbo,
                texture: None,
                quad: Quad::new(gl),
                bounding_box: Rc::new(RefCell::new(None)),
                children_need_rerender: Rc::new(RefCell::new(false)),
                last_size: comped_dims,
                on_cleanup: vec![],
            };

            ret.replace_texture(gl, comped_dims);
            return ret;
        }
    }

    pub fn replace_texture(&mut self, gl: &Context, size: ComputedDimensions) {
        unsafe {
            if let Some(texture) = self.texture {
                gl.delete_texture(texture);
            }

            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo));

            let texture = Some(gl.create_texture().unwrap());
            gl.bind_texture(glow::TEXTURE_2D, texture);

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGB as i32,
                size.width as i32,
                size.height as i32,
                0,
                glow::RGB,
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

            gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                texture,
                0,
            );

            gl.bind_texture(glow::TEXTURE_2D, None);

            self.texture = texture;
        }
    }

    pub fn render(
        &mut self,
        gl: &glow::Context,
        origin: ComputedPosition,
        globals: &Globals,
        parent_dims: &ComputedDimensions,
    ) {

        let dims = compute_dims(&self.dimensions, parent_dims);
        let pos = origin + self.position.compute(parent_dims);

        // Update bounding box
        {
            let mut bb = self.bounding_box.borrow_mut();
            *bb = Some(ComputedBoundingBox {
                top_left: pos,
                bottom_right: pos + dims,
            });
        }

        if *self.children_need_rerender.borrow() {
            self.children_need_rerender.replace(false);
            self.render_children(gl, globals, parent_dims);
        }

        unsafe {
            render_textured_quad(gl, globals, &self.quad, &self.texture, pos, &dims);
        }
    }

    fn render_children(
        &mut self,
        gl: &glow::Context,
        globals: &Globals,
        parent_dims: &ComputedDimensions,
    ) {
        println!("rendering children of framebuf");

        let dims = compute_dims(&self.dimensions, parent_dims);

        if dims != self.last_size {
            self.replace_texture(gl, dims);
            self.last_size = dims;
        }

        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, Some(self.fbo));

            gl.viewport(0, 0, dims.width as i32, dims.height as i32);
            gl.uniform_2_f32(
                Some(&globals.element_uniform_locations["window_size"]),
                dims.width as f32,
                dims.height as f32,
            );

            gl.clear(glow::COLOR_BUFFER_BIT);

            let origin = ComputedPosition::origin();

            if let Some(root_node) = &mut self.root_node {
                root_node.render(gl, origin, globals, &dims);
            }

            gl.bind_framebuffer(glow::FRAMEBUFFER, None);

            gl.viewport(
                0,
                0,
                globals.screen_dims.width as i32,
                globals.screen_dims.height as i32,
            );
            gl.uniform_2_f32(
                Some(&globals.element_uniform_locations["window_size"]),
                globals.screen_dims.width as f32,
                globals.screen_dims.height as f32,
            );
        }
    }

    pub fn cleanup(&self, gl: &Context) {
        for f in &self.on_cleanup {
            f();
        }

        unsafe {
            gl.delete_framebuffer(self.fbo);
            self.quad.cleanup(gl);
            if let Some(texture) = self.texture {
                gl.delete_texture(texture);
            }
        }
    }
}
