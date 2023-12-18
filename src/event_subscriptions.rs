use std::{cell::RefCell, rc::Rc};

use sdl2::{mouse::MouseButton, sys::KeyCode};

use crate::{
    global::Globals,
    ui::{ComputedDimensions, ComputedPosition},
};

pub struct Subscriptions {
    key: Vec<Subscription<Key>>,
    click_in_area: Vec<(
        ComputedPosition,
        ComputedDimensions,
        Subscription<MouseButton>,
    )>,
    id_counter: SubscriptionId,
}

type SubscriptionId = usize;

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
            id_counter: 0,
        }
    }

    pub fn subscribe_key(&mut self, callback: SubscriptionCallback<Key>) -> SubscriptionId {
        let id = self.id_counter;
        self.id_counter += 1;
        self.key.push(Subscription::<Key> { id, callback });
        id
    }

    pub fn unsubscribe_key(&mut self, id: SubscriptionId) {
        self.key.retain(|s| s.id != id);
    }

    pub fn subscribe_click_in_area(
        &mut self,
        position: ComputedPosition,
        dimensions: ComputedDimensions,
        callback: SubscriptionCallback<MouseButton>,
    ) -> SubscriptionId {
        let id = self.id_counter;
        self.id_counter += 1;
        self.click_in_area.push((
            position,
            dimensions,
            Subscription::<MouseButton> { id, callback },
        ));
        id
    }

    pub fn unsubscribe_click_in_area(&mut self, id: SubscriptionId) {
        self.click_in_area.retain(|s| s.2.id != id);
    }
}

struct Key {
    code: KeyCode,
    control: bool,
    shift: bool,
}

pub fn handle_mouse_button_down(globals: &mut Globals, event: &sdl2::event::Event) {
    let (button, x, y) = match event {
        sdl2::event::Event::MouseButtonDown {
            mouse_btn, x, y, ..
        } => (mouse_btn, *x as f32, *y as f32),
        _ => return,
    };

    let mut callbacks_to_make: Vec<SubscriptionCallback<MouseButton>> = vec![];

    {
        let subs: &mut Vec<(
            ComputedPosition,
            ComputedDimensions,
            Subscription<MouseButton>,
        )> = globals.subscriptions.click_in_area.as_mut();

        for (pos, dims, subscription) in subs {
            let x_max = pos.x + dims.width;
            let y_max = pos.y + dims.height;
            if x >= pos.x && x <= x_max && y >= pos.y && y <= y_max {
                callbacks_to_make.push(subscription.callback.clone());
            }
        }
    }

    for callbacks in callbacks_to_make {
        callbacks.borrow()(&button, globals);
    }
}
