struct Reactive<T> {
    value: T,
    // dependants: Vec<Dependant>,
}

impl<T> Reactive<T> {
    fn new(value: T) -> Self {
        Self {
            value,
            // dependants: Vec::new(),
        }
    }

    fn get(&self) -> &T {
        &self.value
    }

    fn set(&mut self, value: T) {
        self.value = value;
        // for dependant in &self.dependants {
        //     dependant.update();
        // }
    }
}

