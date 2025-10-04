use std::cell::UnsafeCell;

pub struct Cell<T> {
    value: UnsafeCell<T>,
}

unsafe impl<T> Send for Cell<T> where T: Send {}
// unsafe impl<T> !Sync for Cell<T> {}

impl<T> Cell<T> {
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
        }
    }

    pub fn get(&self) -> T
    where
        T: Copy,
    {
        unsafe { *self.value.get() }
    }

    pub fn set(&self, value: T) {
        unsafe {
            *self.value.get() = value;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell() {
        let cell = Cell::new(5);
        assert_eq!(cell.get(), 5);
        cell.set(10);
        assert_eq!(cell.get(), 10);
    }
}
