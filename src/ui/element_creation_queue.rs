use std::{sync::Mutex, cell::{Cell, OnceCell}};

use glow::Context;

use crate::{global::Globals, ui::element::ElementRef};

static mut QUEUE: Vec<CreateElementRequest> = Vec::new();

pub fn queue_element(f: CreateElementFn, parent: ElementRef) {
    unsafe {
        QUEUE.push(CreateElementRequest { f, parent });
    }
}

pub type CreateElementFn = Box<dyn FnOnce(&Context, &mut Globals) -> ElementRef>;

struct CreateElementRequest {
    f: CreateElementFn,
    parent: ElementRef,
}

pub fn fulfil_queue(gl: &Context, globals: &mut Globals) {
    unsafe {
        let queue = std::mem::take(&mut QUEUE);
        for request in queue {
            let element = (request.f)(gl, globals);
            let parent = request.parent;
            parent.mutate(Box::new(move |parent| {
                parent.children.push(element.clone());
            }));
        }
    }
}