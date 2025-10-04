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

    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.value.get() }
    }

    pub fn into_inner(self) -> T {
        self.value.into_inner()
    }
}

/// ```compile_fail
/// use learn_unsafe::cell::Cell;
/// let cell = Cell::new(String::from("Hello"));
/// assert_eq!(cell.get(), String::from("Hello"));
/// println!("Cell contains: {}", cell.get());
/// ```
fn _bar() {}

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

    #[test]
    fn test_cell_into_inner() {
        let cell = Cell::new(String::from("Hello"));
        let inner = cell.into_inner();
        assert_eq!(inner, "Hello");
    }
}
