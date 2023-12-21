use std::{cell::RefCell, rc::Rc};

use sdl2::{mouse::MouseButton, sys::KeyCode, event::Event};

use crate::{
    global::Globals,
    ui::{bounding_box_ref_contains, p, p_c, BoundingBoxRef, ComputedDimensions, ComputedPosition},
};

pub struct Subscriptions {
    key: Vec<Subscription<Key>>,
    click_in_area: Vec<(BoundingBoxRef, Subscription<MouseButton>)>,
    text_input: Vec<Subscription<String>>,
    midi_input: Vec<Subscription<MidiInputEvent>>,
    scroll_in_area: Vec<(BoundingBoxRef, Subscription<(f32, f32)>)>,
    id_counter: SubscriptionId,
}

pub type SubscriptionId = usize;

type SubscriptionCallback<T> = Rc<RefCell<dyn Fn(&T, &mut Globals)>>;

struct Subscription<T> {
    id: SubscriptionId,
    callback: SubscriptionCallback<T>,
}

impl Subscriptions {
    pub fn new() -> Self {
        Self {
            key: vec![],
            click_in_area: vec![],
            text_input: vec![],
            midi_input: vec![],
            scroll_in_area: vec![],
            id_counter: 0,
        }
    }

    pub fn id(&mut self) -> SubscriptionId {
        let id = self.id_counter;
        self.id_counter += 1;
        id
    }

    pub fn subscribe_key(&mut self, callback: SubscriptionCallback<Key>) -> SubscriptionId {
        let id = self.id();
        self.key.push(Subscription::<Key> { id, callback });
        id
    }

    pub fn unsubscribe_key(&mut self, id: SubscriptionId) {
        self.key.retain(|s| s.id != id);
    }

    pub fn subscribe_click_in_area(
        &mut self,
        bounding_box: BoundingBoxRef,
        callback: SubscriptionCallback<MouseButton>,
    ) -> SubscriptionId {
        let id = self.id();
        self.click_in_area
            .push((bounding_box, Subscription::<MouseButton> { id, callback }));
        id
    }

    pub fn unsubscribe_click_in_area(&mut self, id: SubscriptionId) {
        self.click_in_area.retain(|(_, s)| s.id != id);
    }

    pub fn subscribe_text_input(&mut self, callback: SubscriptionCallback<String>) -> SubscriptionId {
        let id = self.id();
        self.text_input
            .push(Subscription::<String> { id, callback });
        id
    }

    pub fn unsubscribe_text_input(&mut self, id: SubscriptionId) {
        self.text_input.retain(|s| s.id != id);
    }

    pub fn subscribe_midi_input(
        &mut self,
        callback: SubscriptionCallback<MidiInputEvent>,
    ) -> SubscriptionId {
        let id = self.id();
        self.midi_input
            .push(Subscription::<MidiInputEvent> { id, callback });
        id
    }

    pub fn unsubscribe_midi_input(&mut self, id: SubscriptionId) {
        self.midi_input.retain(|s| s.id != id);
    }

    pub fn subscribe_scroll_in_area(
        &mut self,
        bounding_box: BoundingBoxRef,
        callback: SubscriptionCallback<(f32, f32)>,
    ) -> SubscriptionId {
        let id = self.id();
        self.scroll_in_area
            .push((bounding_box, Subscription::<(f32, f32)> { id, callback }));
        id
    }

    pub fn unsubscribe_scoll_in_area(&mut self, id: SubscriptionId) {
        self.scroll_in_area.retain(|(_, s)| s.id != id);
    }
}

pub fn handle_event_subscriptions(globals: &mut Globals, event: &sdl2::event::Event) {
    handle_mouse_move(globals, event);
    handle_mouse_button_down(globals, event);
    handle_key_down(globals, event);
    handle_text_input(globals, event);
    handle_wheel(globals, event);
}

#[derive(Debug, PartialEq, Eq)]
pub struct Key {
    pub code: KeyCode,
    pub control: bool,
    pub shift: bool,
}

#[derive(Debug)]
pub struct MidiInputEvent {
    pub channel: u8,
    pub stamp: u32,
    pub message: [u8; 3],
}

fn handle_mouse_button_down(globals: &mut Globals, event: &sdl2::event::Event) {
    let (button, x, y) = match event {
        sdl2::event::Event::MouseButtonDown {
            mouse_btn, x, y, ..
        } => (mouse_btn, *x, *y),
        _ => return,
    };

    let mut callbacks_to_make: Vec<SubscriptionCallback<MouseButton>> = vec![];

    let pos = convert_sld_mouse_position(x, y, globals);
    {
        let subs: &mut Vec<(BoundingBoxRef, Subscription<MouseButton>)> =
            globals.subscriptions.click_in_area.as_mut();

        for (bb, subscription) in subs {
            if bounding_box_ref_contains(bb, pos) {
                callbacks_to_make.push(subscription.callback.clone());
            }
        }
    }

    for callbacks in callbacks_to_make {
        callbacks.borrow()(&button, globals);
    }
}

pub fn convert_sld_mouse_position(x: i32, y: i32, globals: &mut Globals) -> ComputedPosition {
    p_c(x as f32, globals.screen_dims.height - y as f32)
}

fn handle_key_down(globals: &mut Globals, event: &sdl2::event::Event) {
    let (code, control, shift) = match event {
        sdl2::event::Event::KeyDown {
            keycode, keymod, ..
        } => (
            keycode,
            keymod.contains(sdl2::keyboard::Mod::LCTRLMOD),
            keymod.contains(sdl2::keyboard::Mod::LSHIFTMOD),
        ),
        _ => return,
    };

    let mut callbacks_to_make: Vec<SubscriptionCallback<Key>> = vec![];

    {
        let subs: &mut Vec<Subscription<Key>> = globals.subscriptions.key.as_mut();

        for subscription in subs {
            callbacks_to_make.push(subscription.callback.clone());
        }
    }

    if let Some(code) = code {
        let code = code;
        println!("{}", *code as u8);

        for callbacks in callbacks_to_make {
            callbacks.borrow()(
                &Key {
                    code: *code as u8,
                    control,
                    shift,
                },
                globals,
            );
        }
    }
}

fn handle_text_input(globals: &mut Globals, event: &sdl2::event::Event) {
    let text = match event {
        sdl2::event::Event::TextInput { text, .. } => text,
        _ => return,
    };

    let mut callbacks_to_make: Vec<SubscriptionCallback<String>> = vec![];

    {
        let subs: &mut Vec<Subscription<String>> = globals.subscriptions.text_input.as_mut();

        for subscription in subs {
            callbacks_to_make.push(subscription.callback.clone());
        }
    }

    for callbacks in callbacks_to_make {
        callbacks.borrow()(text, globals);
    }
}

fn handle_wheel(globals: &mut Globals, event: &Event) {
    let dir = match event {
        Event::MouseWheel { precise_x, precise_y, .. } => (*precise_x as f32, *precise_y as f32),
        _ => return,
    };

    let mut callbacks_to_make: Vec<SubscriptionCallback<(f32, f32)>> = vec![];

    let pos = globals.mouse_pos;
    {
        let subs: &mut Vec<(BoundingBoxRef, Subscription<(f32, f32)>)> =
            globals.subscriptions.scroll_in_area.as_mut();

        for (bb, subscription) in subs {
            if bounding_box_ref_contains(bb, pos) {
                callbacks_to_make.push(subscription.callback.clone());
            }
        }
    }

    for callbacks in callbacks_to_make {
        callbacks.borrow()(&dir, globals);
    }
}

fn handle_mouse_move(globals: &mut Globals, event: &Event) {
    let (x, y) = match event {
        Event::MouseMotion { x, y, .. } => (*x, *y),
        _ => return,
    };

    globals.mouse_pos = convert_sld_mouse_position(x, y, globals);
}