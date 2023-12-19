use std::{cell::RefCell, rc::Rc, ops::{ShlAssign, AddAssign, SubAssign}};

use sdl2::libc::PACKET_ADD_MEMBERSHIP;

static ID_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub struct Reactive<T>
where
    T: Clone,
{
    ref_count: *mut usize,
    value: Rc<RefCell<T>>,
    dependants: Rc<RefCell<Vec<ReactiveSubscription<T>>>>,
    delete_listeners: Rc<RefCell<Vec<ReactiveSubscription<T>>>>,
}

impl<T> Default for Reactive<T>
where
    T: Default + Clone,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

pub type ReactiveSubscriptionId = usize;

struct ReactiveSubscription<T> {
    id: ReactiveSubscriptionId,
    callback: Box<dyn Fn(&T)>,
}

impl<T> Clone for Reactive<T>
where
    T: Clone,
{
    fn clone(&self) -> Self {
        unsafe {
            *self.ref_count += 1;
        }

        Self {
            ref_count: self.ref_count.clone(),
            value: self.value.clone(),
            dependants: self.dependants.clone(),
            delete_listeners: self.delete_listeners.clone(),
        }
    }
}

impl<T> Reactive<T>
where
    T: Clone,
{
    pub fn new(value: T) -> Self {
        Self {
            ref_count: Box::into_raw( Box::new(1)),
            value: Rc::new(RefCell::new(value)),
            dependants: Rc::new(RefCell::new(vec![])),
            delete_listeners: Rc::new(RefCell::new(vec![])),
        }
    }

    pub fn get(&self) -> Rc<RefCell<T>> {
        self.value.clone()
    }

    pub fn get_copy(&self) -> T {
        self.value.borrow().clone()
    }

    pub fn set(&self, value: T) {
        self.value.replace(value);
        for dependant in self.dependants.borrow().iter() {
            (dependant.callback)(&self.value.borrow());
        }
    }

    pub fn subscribe(&self, callback: Box<dyn Fn(&T)>) -> ReactiveSubscriptionId {
        let id = ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.dependants
            .borrow_mut()
            .push(ReactiveSubscription { id, callback });
        id
    }

    pub fn unsubscribe(&self, id: ReactiveSubscriptionId) {
        self.dependants
            .borrow_mut()
            .retain(|dependant| dependant.id != id);
    }

    pub fn subscribe_delete(&self, callback: Box<dyn Fn(&T)>) -> ReactiveSubscriptionId {
        let id = ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.delete_listeners
            .borrow_mut()
            .push(ReactiveSubscription { id, callback });
        id
    }

    pub fn unsubscribe_delete(&self, id: ReactiveSubscriptionId) {
        self.delete_listeners
            .borrow_mut()
            .retain(|dependant| dependant.id != id);
    }

    pub fn mutate(&mut self, callback: Box<dyn Fn(&mut T)>) {
        callback(&mut self.value.borrow_mut());
        for dependant in self.dependants.borrow().iter() {
            (dependant.callback)(&self.value.borrow());
        }
    }

    fn run_delete_callbacks(&self) {
        for dependant in self.delete_listeners.borrow().iter() {
            (dependant.callback)(&self.value.borrow());
        }
    }
}

impl<T> ShlAssign<T> for Reactive<T>
where
    T: Clone,
{
    fn shl_assign(&mut self, rhs: T) {
        self.set(rhs);
    }
}

impl<T> Drop for Reactive<T>
where
    T: Clone,
{
    fn drop(&mut self) {
        unsafe {
            *self.ref_count -= 1;
            if *self.ref_count == 0 {
                self.run_delete_callbacks();
                drop(Box::from_raw(self.ref_count));
            }
        }
    }
}

impl<T> AddAssign<T> for Reactive<T>
where
    T: Clone + std::ops::AddAssign + 'static,
{
    fn add_assign(&mut self, rhs: T) {
        let rhs = rhs.clone();
        self.mutate(Box::new(move |value: &mut T| {
            *value += rhs.clone();
        }));
    }
}

impl<T> SubAssign<T> for Reactive<T>
where
    T: Clone + std::ops::SubAssign + 'static,
{
    fn sub_assign(&mut self, rhs: T) {
        let rhs = rhs.clone();
        self.mutate(Box::new(move |value: &mut T| {
            *value -= rhs.clone();
        }));
    }
}

