use std::{cell::RefCell, rc::Rc};

use glow::Context;
use sdl2::libc::glob;

use crate::{global::Globals, ui::style::Style, bind_reactives};

use super::{
    element::{Element, ElementRef},
    reactive::Reactive,
    BoundingBoxRef, Position, Size, Coordinate,
};

pub fn e_scroll_window(
    gl: &Context,
    globals: &mut Globals,
    needs_rerender: Rc<RefCell<bool>>,
    frame_bounding_box: BoundingBoxRef,
    v_scroll: bool,
    default_v_scroll: f32,
    h_scroll: bool,
    default_h_scroll: f32,
    children: Vec<ElementRef>,
) -> ElementRef {
    let element = Element::new(
        gl,
        Position::origin(),
        Size::FractionOfParent(1.),
        Size::FractionOfParent(1.),
        Some(Style {
            render_self: false,
            ..Style::default()
        }),
        None,
        needs_rerender.clone(),
        frame_bounding_box.clone(),
        children,
    );

    let scroll = Reactive::new(WindowScroll::new(default_h_scroll, default_v_scroll));


    {
        let scroll = scroll.clone();
        let sub = globals.subscriptions.subscribe_scroll_in_area(
            frame_bounding_box.clone(),
            Rc::new(RefCell::new(
                move |wheel: &(f32, f32), globals: &mut Globals| {
                    let scroll = scroll.clone();
                    let wheel = wheel.clone();
                    scroll.mutate(Box::new(move |scroll: &mut WindowScroll| {
                        let (s_x, s_y) = wheel;
                        const scroll_speed: f32 = 10.;

                        if h_scroll {
                            scroll.scroll_x += s_x * scroll_speed;
                        }

                        if v_scroll {
                            scroll.scroll_y += s_y * scroll_speed;
                        }
                    }));
                },
            )),
        );

        // TODO: unsubscribe when element is deleted

        // element.add_cleanup_callback(Box::new(move || {
        //     globals.subscriptions.unsubscribe_scoll_in_area(sub);
        // }));
    }

    bind_reactives! {
        element {
            [scroll] => (|e: &mut Element, s: WindowScroll| {
                e.position = Position {
                    x: Coordinate::Fixed(-s.scroll_x),
                    y: Coordinate::Fixed(-s.scroll_y),
                };
            })
        }
    }

    return element;
}

#[derive(Debug, Clone)]
pub struct WindowScroll {
    pub scroll_x: f32,
    pub scroll_y: f32,
}

impl WindowScroll {
    fn new(scroll_x: f32, scroll_y: f32) -> Self {
        Self {
            scroll_x,
            scroll_y
        }
    }
}

impl Default for WindowScroll {
    fn default() -> Self {
        Self {
            scroll_x: 0.,
            scroll_y: 0.,
        }
    }
}
