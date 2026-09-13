use std::cell::RefCell;

#[derive(Debug, Default, Clone)]
pub struct Property<T> {
    pub(crate) value: RefCell<Option<T>>,
}

impl<T> Property<T> {
    pub fn set(&self, value: T) {
        *self.value.borrow_mut() = Some(value)
    }

    pub fn clear(&self) {
        *self.value.borrow_mut() = None;
    }

    pub fn take(&self) -> Option<T> {
        self.value.borrow_mut().take()
    }
}

impl<T> Property<T>
where
    T: Clone,
{
    pub fn value(&self) -> Option<T> {
        self.value.borrow().as_ref().cloned()
    }
}
