use std::{cell::RefCell, rc::Rc, ops::ShlAssign};

static ID_COUNTER: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

pub struct Reactive<T>
where
    T: Clone,
{
    value: Rc<RefCell<T>>,
    dependants: Rc<RefCell<Vec<ReactiveSubscription<T>>>>, // todo: global callback system
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
        Self {
            value: self.value.clone(),
            dependants: self.dependants.clone(),
        }
    }
}

impl<T> Reactive<T>
where
    T: Clone,
{
    pub fn new(value: T) -> Self {
        Self {
            value: Rc::new(RefCell::new(value)),
            dependants: Rc::new(RefCell::new(vec![])),
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
}

impl<T> ShlAssign<T> for Reactive<T>
where
    T: Clone,
{
    fn shl_assign(&mut self, rhs: T) {
        self.set(rhs);
    }
}