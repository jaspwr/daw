use std::cell::RefCell;
use std::rc::Rc;

use glow::*;

use crate::global::Globals;

use super::gl::{create_quad, Quad};
use super::style::*;
use super::text::Text;
use super::*;

pub struct Element {
    pub position: Position,
    pub dimensions: Dimensions,
    pub children: Vec<Rc<RefCell<Element>>>,
    pub text: Option<Text>,
    pub style: Style,
    quad: Quad,
    pub needs_rerender: Rc<RefCell<bool>>,
    pub on_cleanup: Vec<Box<dyn Fn()>>,
    pub on_render: Vec<Box<dyn Fn()>>,
}

pub type ElementRef = Rc<RefCell<Element>>;

impl Element {
    pub fn new(
        gl: &glow::Context,
        position: Position,
        width: Size,
        height: Size,
        style: Option<Style>,
        text: Option<Text>,
        needs_rerender: Rc<RefCell<bool>>,
        children: Vec<Rc<RefCell<Element>>>,
    ) -> ElementRef {
        needs_rerender.replace(true);

        Rc::new(RefCell::new(Self {
            position,
            dimensions: Dimensions { width, height },
            children,
            style: match style {
                Some(style) => style,
                None => Style::default(),
            },
            text,
            needs_rerender,
            quad: unsafe { Quad::new(gl) },
            on_cleanup: vec![],
            on_render: vec![],
        }))
    }

    pub fn render(
        &mut self,
        gl: &glow::Context,
        origin: ComputedPosition,
        globals: &Globals,
        parent_dims: &ComputedDimensions,
    ) {
        for func in self.on_render.iter() {
            func();
        }

        if !self.style.visible {
            return;
        }

        let dims = ComputedDimensions {
            width: self.dimensions.width.to_size(parent_dims.width),
            height: self.dimensions.height.to_size(parent_dims.height),
        };

        let comped_pos = self.position.compute(parent_dims);

        if self.style.render_self {
            unsafe {
                gl.use_program(Some(globals.element_shader));
                gl.uniform_2_f32(
                    Some(&globals.element_uniform_locations["dims"]),
                    dims.width,
                    dims.height,
                );

                gl.bind_vertex_array(Some(self.quad.vao));

                let pos = origin + comped_pos;
                create_quad(gl, &self.quad.vbo, pos, &dims);

                self.style.set(gl, globals);
                gl.draw_arrays(glow::TRIANGLE_FAN, 0, 4);

                gl.disable_vertex_attrib_array(0);
                gl.bind_buffer(glow::ARRAY_BUFFER, None);
            }
        }

        let child_origin = origin + comped_pos;

        if let Some(text) = &mut self.text {
            let text_pos = child_origin
                + p_c(
                    self.style.padding_left + self.style.padding,
                    -self.style.padding_top + self.style.padding,
                );

            text.render(gl, text_pos, globals, &dims);
        }

        for child in self.children.iter_mut() {
            child.borrow_mut().render(gl, child_origin, globals, &dims);
        }
    }

    pub fn mutate(&mut self, func: Box<dyn Fn(&mut Self)>) {
        func(self);
        self.needs_rerender.replace(true);
    }

    pub fn cleanup(&self, gl: &glow::Context) {
        unsafe {
            self.quad.cleanup(gl);
        }

        for child in self.children.iter() {
            child.borrow().cleanup(gl);
        }

        for func in self.on_cleanup.iter() {
            func();
        }
    }
}
