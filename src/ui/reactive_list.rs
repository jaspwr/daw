use std::{
    cell::RefCell,
    ops::Index,
    rc::Rc,
    sync::atomic::{AtomicU32, AtomicUsize},
};

use crate::utils::{fetch_ptr, free, leak, malloc};

use super::reactive::{ReactiveSubscription, ReactiveSubscriptionId};

/// Allows adding and removing items to be subscribed to.
pub struct ReactiveList<T> {
    ref_count: *mut usize,
    items: *mut Vec<(ReactiveListKey, T)>,
    push_subscriptions: Rc<RefCell<Vec<ReactiveListSubscription<T>>>>,
    remove_subscriptions: Rc<RefCell<Vec<ReactiveListSubscription<()>>>>,
}

struct ReactiveListSubscription<T> {
    id: ReactiveSubscriptionId,
    callback: ReactiveListSubscriptionCallback<T>,
}

type ReactiveListSubscriptionCallback<T> = Box<dyn Fn(&ReactiveListKey, &T)>;

static ID_COUNTER: AtomicUsize = AtomicUsize::new(0);

pub type ReactiveListKey = usize;

impl<T> Clone for ReactiveList<T> {
    fn clone(&self) -> Self {
        unsafe {
            *self.ref_count += 1;
        }

        Self {
            ref_count: self.ref_count.clone(),
            items: self.items.clone(),
            push_subscriptions: self.push_subscriptions.clone(),
            remove_subscriptions: self.remove_subscriptions.clone(),
        }
    }
}

impl<T> Drop for ReactiveList<T> {
    fn drop(&mut self) {
        unsafe {
            *self.ref_count -= 1;
            if *self.ref_count == 0 {
                free(self.ref_count);
                free(self.items);
            }
        }
    }
}

fn new_key() -> ReactiveListKey {
    ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

impl<T> ReactiveList<T> {
    pub fn new() -> Self {
        let ref_count = malloc(1);
        let items = malloc(vec![]);

        ReactiveList {
            ref_count,
            items,
            push_subscriptions: Rc::new(RefCell::new(vec![])),
            remove_subscriptions: Rc::new(RefCell::new(vec![])),
        }
    }

    unsafe fn items_ptr(&self) -> *mut Vec<(ReactiveListKey, T)> {
        return self.items;
    }

    pub fn push(&mut self, item: T) -> ReactiveListKey {
        let key = new_key();

        for subscription in self.push_subscriptions.borrow().iter() {
            (subscription.callback)(&key, &item);
        }

        unsafe {
            let mut items = fetch_ptr(self.items);
            (*items).push((key, item));
            leak(items);
        }
        key
    }

    pub fn remove(&mut self, key: &ReactiveListKey) {
        for subscription in self.remove_subscriptions.borrow().iter() {
            (subscription.callback)(key, &());
        }

        unsafe {
            let mut items = fetch_ptr(self.items);
            (*items).retain(|(k, _)| k != key);
            leak(items);
        }
    }

    pub fn subscribe_to_push(
        &self,
        callback: ReactiveListSubscriptionCallback<T>,
    ) -> ReactiveSubscriptionId {
        let id = ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.push_subscriptions
            .borrow_mut()
            .push(ReactiveListSubscription { id, callback });
        id
    }

    pub fn unsubscribe_to_push(&self, id: ReactiveSubscriptionId) {
        self.push_subscriptions
            .borrow_mut()
            .retain(|dependant| dependant.id != id);
    }

    pub fn subscribe_to_remove(
        &self,
        callback: ReactiveListSubscriptionCallback<()>,
    ) -> ReactiveSubscriptionId {
        let id = ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.remove_subscriptions
            .borrow_mut()
            .push(ReactiveListSubscription { id, callback });
        id
    }

    pub fn unsubscribe_to_remove(&self, id: ReactiveSubscriptionId) {
        self.remove_subscriptions
            .borrow_mut()
            .retain(|dependant| dependant.id != id);
    }
}

impl<T> ReactiveList<T>
where
    T: Clone,
{
    pub fn get_copy(&self, key: ReactiveListKey) -> Option<T> {
        unsafe {
            let items = fetch_ptr(self.items);
            if let Some((_, item)) = (*items).iter().find(|(k, _)| *k == key) {
                let item = item.clone();
                leak(items);
                return Some(item);
            }
            leak(items);
            None
        }
    }

    pub fn copy_of_whole_list(&self) -> Vec<(ReactiveListKey, T)> {
        unsafe {
            let items = fetch_ptr(self.items);
            let list = (*items).clone();
            leak(items);
            list
        }
    }
}

// impl<T> Index<ReactiveListKey> for ReactiveList<T>
// where T : 'static
// {
//     type Output = T;

//     fn index<'a>(&'a self, index: ReactiveListKey) -> &'a Self::Output {
//         let items = self.items.borrow();
//         let (_, item) = items.iter().find(|(key, _)| *key == index).unwrap();
//         item
//         // self.items.borrow().iter().find(|(key, _)| *key == index).unwrap().1
//     }
// }

// pub struct ReactiveListItemRef<T> {
//     // Because this is a reference counted smart pointer, keeping
//     // a copy of it here keeps it alive while this reference is still alive.
//     list: ReactiveList<T>,
//     key: ReactiveListKey,
// }

// impl<T> ReactiveListItemRef<T> {
//     pub fn borrow<'a>(&'a self) -> &'a T {
//         unsafe {
//             let items = fetch_ptr(self.list.items);
//             let (_, item) = (*items).iter().find(|(key, _)| *key == self.key).unwrap();
//             item: mut* T =
//             leak(items);
//             item
//         }
//     }
// }
