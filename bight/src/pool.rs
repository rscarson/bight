pub struct Pool<T, M: Manager<Type = T>> {
    list: Vec<T>,
    manager: M,
}

pub trait Manager {
    type Type;
    fn create(&self) -> Self::Type;
}

impl<T, M: Manager<Type = T>> Pool<T, M> {
    pub fn get(&mut self) -> M::Type {
        self.list.pop().unwrap_or_else(|| self.manager.create())
    }
    pub fn put(&mut self, value: T) {
        self.list.push(value);
    }
    pub const fn new(manager: M) -> Self {
        Self {
            list: Vec::new(),
            manager,
        }
    }
}
