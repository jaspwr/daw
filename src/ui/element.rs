use std::cell::RefCell;
use std::rc::Rc;

use glow::*;

use crate::global::Globals;

use super::gl::{create_quad, Quad};
use super::reactive::Reactive;
use super::style::*;
use super::text::Text;
use super::*;

pub struct Element {
    pub position: Position,
    pub dimensions: Dimensions,
    pub children: Vec<ElementRef>,
    pub text: Option<Text>,
    pub style: Style,
    quad: Quad,
    pub needs_rerender: Rc<RefCell<bool>>,
    pub on_cleanup: Vec<Box<dyn Fn()>>,
    pub on_render: Vec<Box<dyn Fn()>>,
}

// pub type ElementRef = Rc<RefCell<Element>>;

pub struct ElementRef {
    pub ptr: *mut Element,
    ref_count: *mut usize,
}

impl ElementRef {
    pub fn new(element: Element) -> Self {
        let ptr = Box::into_raw(Box::new(element));
        let ref_count = Box::into_raw(Box::new(1));

        Self { ptr, ref_count }
    }

    pub fn borrow<'a>(&'a self) -> &'a Element {
        unsafe { &*self.ptr }
    }

    // pub fn borrow_mut<'a>(&'a mut self) -> &'a mut Element {
    //     unsafe { &mut *self.ptr }
    // }

    pub fn render(
        &self,
        gl: &glow::Context,
        origin: ComputedPosition,
        globals: &Globals,
        parent_dims: &ComputedDimensions,
    ) {
        unsafe {
            (*self.ptr).render(gl, origin, globals, parent_dims);
        }
    }

    pub fn mutate(&self, func: Box<dyn Fn(&mut Element)>) {
        unsafe {
            (*self.ptr).mutate(func);
        }
    }

    pub fn add_cleanup_callback(&self, func: Box<dyn Fn()>) {
        unsafe {
            (*self.ptr).on_cleanup.push(func);
        }
    }

    pub fn cleanup(&self, gl: &glow::Context) {
        unsafe {
            (*self.ptr).cleanup(gl);
        }
    }
}

impl Clone for ElementRef {
    fn clone(&self) -> Self {
        unsafe {
            *self.ref_count += 1;
        }

        Self {
            ptr: self.ptr,
            ref_count: self.ref_count,
        }
    }
}

impl Drop for ElementRef {
    fn drop(&mut self) {
        unsafe {
            *self.ref_count -= 1;

            if *self.ref_count == 0 {
                drop(Box::from_raw(self.ptr));
                drop(Box::from_raw(self.ref_count));
            }
        }
    }
}

impl Element {
    pub fn new(
        gl: &glow::Context,
        position: Position,
        width: Size,
        height: Size,
        style: Option<Style>,
        text: Option<Text>,
        needs_rerender: Rc<RefCell<bool>>,
        children: Vec<ElementRef>,
    ) -> ElementRef {
        needs_rerender.replace(true);

        ElementRef::new(Self {
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
        })
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
            child.render(gl, child_origin, globals, &dims);
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

    pub fn subscribe_mutation_to_reactive<T>(
        element: &ElementRef,
        reactive: &Reactive<T>,
        callback: Box<dyn Fn(&mut Element, &T)>,
    ) where
        T: Clone + 'static,
    {
        let mut element = element.clone();

        let id = {
            let element = element.clone();
            let callback = Rc::new(callback);
            reactive.subscribe(Box::new(move |new_value| {
                let new_value = new_value.clone();
                let callback = callback.clone();
                element.mutate(Box::new(move |element: &mut Element| {
                    callback(element, &new_value);
                }));
            }))
        };

        let reactive = reactive.clone();
        element.add_cleanup_callback(Box::new(move || {
            reactive.unsubscribe(id);
        }));
    }

    // pub fn subscribe_recreate_to_reactive<T>(
    //     element: &ElementRef,
    //     reactive: &Reactive<T>,
    //     create: Box<dyn Fn(&T) -> ElementRef>,
    // ) where
    //     T: Clone + 'static,
    // {
    //     let element = element.clone();

    //     let id = {
    //         let element = element.clone();
    //         let callback = Rc::new(create);
    //         reactive.subscribe(Box::new(move |new_value| {
    //             let new_value = new_value.clone();
    //             let callback = callback.clone();
    //             let new_element = callback(&new_value);
    //             element.replace(new_element.borrow().clone());
    //         }))
    //     };

    //     let reactive = reactive.clone();
    //     element.borrow_mut().on_cleanup.push(Box::new(move || {
    //         reactive.unsubscribe(id);
    //     }));
    // }
}
