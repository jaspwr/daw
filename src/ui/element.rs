use std::cell::RefCell;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};

use glow::*;

use crate::global::Globals;

use super::gl::{create_quad, Quad};
use super::reactive::Reactive;
use super::style::*;
use super::text::Text;
use super::*;

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub struct Element {
    uid: usize,
    pub position: Position,
    pub dimensions: Dimensions,
    pub bounding_box: BoundingBoxRef,
    pub frame_bounding_box: BoundingBoxRef,
    pub children: Vec<ElementRef>,
    pub text_node: Option<Text>,
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

    pub fn uid(&self) -> usize {
        unsafe { (*self.ptr).uid }
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

    pub fn subscribe_mutation_to_reactive<T>(
        &self,
        reactive: &Reactive<T>,
        callback: Box<dyn Fn(&mut Element, &T)>,
    ) where
        T: Clone + 'static,
    {
        let mut element = self.clone();

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

    pub fn subscribe_mutation_to_reactive_rc<T>(
        &self,
        reactive: &Reactive<T>,
        callback: Rc<dyn Fn(&mut Element, &T)>,
    ) where
        T: Clone + 'static,
    {
        let mut element = self.clone();

        let id = {
            let element = element.clone();
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
        frame_bounding_box: BoundingBoxRef,
        children: Vec<ElementRef>,
    ) -> ElementRef {
        needs_rerender.replace(true);
        let uid = ID_COUNTER.fetch_add(1, Ordering::SeqCst);

        ElementRef::new(Self {
            uid,
            position,
            dimensions: Dimensions { width, height },
            children,
            style: match style {
                Some(style) => style,
                None => Style::default(),
            },
            text_node: text,
            needs_rerender,
            quad: unsafe { Quad::new(gl) },
            on_cleanup: vec![],
            on_render: vec![],
            frame_bounding_box,
            bounding_box: Rc::new(RefCell::new(None)),
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

        let pos = origin + self.position.compute(parent_dims);

        // Update computed bounding box
        {
            let mut bounding_box = self.bounding_box.borrow_mut();
            let frame_bouding_box = self.frame_bounding_box.borrow();
            if let Some(frame_bb) = frame_bouding_box.as_ref() {
                *bounding_box = Some(ComputedBoundingBox {
                    top_left: pos + frame_bb.top_left,
                    bottom_right: pos + dims + frame_bb.top_left,
                });
            }
        }

        if self.style.render_self {
            unsafe {
                gl.use_program(Some(globals.element_shader));
                gl.uniform_2_f32(
                    Some(&globals.element_uniform_locations["dims"]),
                    dims.width,
                    dims.height,
                );

                gl.bind_vertex_array(Some(self.quad.vao));

                create_quad(gl, &self.quad.vbo, pos, &dims);

                self.style.set(gl, globals);
                gl.draw_arrays(glow::TRIANGLE_FAN, 0, 4);

                gl.disable_vertex_attrib_array(0);
                gl.bind_buffer(glow::ARRAY_BUFFER, None);
            }
        }

        if let Some(text) = &mut self.text_node {
            let text_pos = pos
                + p_c(
                    self.style.padding_left + self.style.padding,
                    -self.style.padding_top + self.style.padding,
                );

            text.render(gl, text_pos, globals, &dims);
        }

        for child in self.children.iter_mut() {
            child.render(gl, pos, globals, &dims);
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
