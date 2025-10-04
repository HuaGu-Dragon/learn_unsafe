use std::{
    cell::UnsafeCell,
    ops::{Deref, DerefMut},
};

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

#[derive(Clone, Copy)]
enum BorrowState {
    Unshared,
    Shared(usize),
    Exclusive,
}
pub struct RefCell<T> {
    value: UnsafeCell<T>,
    state: Cell<BorrowState>,
}

unsafe impl<T> Send for RefCell<T> where T: Send {}
// unsafe impl<T> !Sync for RefCell<T> {}

pub struct Ref<'refcell, T> {
    cell: &'refcell RefCell<T>,
}

pub struct RefMut<'refcell, T> {
    cell: &'refcell RefCell<T>,
}

impl<T> RefCell<T> {
    pub const fn new(value: T) -> Self {
        Self {
            value: UnsafeCell::new(value),
            state: Cell::new(BorrowState::Unshared),
        }
    }

    pub fn borrow(&self) -> Option<Ref<'_, T>> {
        match self.state.get() {
            BorrowState::Unshared => {
                self.state.set(BorrowState::Shared(1));
                Some(Ref { cell: self })
            }
            BorrowState::Shared(n) => {
                self.state.set(BorrowState::Shared(n + 1));
                Some(Ref { cell: self })
            }
            BorrowState::Exclusive => None,
        }
    }

    pub fn borrow_mut(&self) -> Option<RefMut<'_, T>> {
        match self.state.get() {
            BorrowState::Unshared => {
                self.state.set(BorrowState::Exclusive);
                Some(RefMut { cell: self })
            }
            _ => None,
        }
    }
}

impl<T> Drop for Ref<'_, T> {
    fn drop(&mut self) {
        match self.cell.state.get() {
            BorrowState::Shared(1) => self.cell.state.set(BorrowState::Unshared),
            BorrowState::Shared(n) => self.cell.state.set(BorrowState::Shared(n - 1)),
            BorrowState::Unshared | BorrowState::Exclusive => unreachable!(),
        }
    }
}

impl<T> Drop for RefMut<'_, T> {
    fn drop(&mut self) {
        match self.cell.state.get() {
            BorrowState::Unshared | BorrowState::Shared(_) => unreachable!(),
            BorrowState::Exclusive => self.cell.state.set(BorrowState::Unshared),
        }
    }
}

impl<T> Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.cell.value.get() }
    }
}

impl<T> Deref for RefMut<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.cell.value.get() }
    }
}

impl<T> DerefMut for RefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.cell.value.get() }
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

    #[test]
    fn test_refcell() {
        let refcell = RefCell::new(vec![42]);
        assert_eq!(refcell.borrow().unwrap()[0], 42);
        refcell.borrow_mut().unwrap().push(42);
        assert_eq!(refcell.borrow().unwrap().len(), 2);
    }

    #[test]
    #[should_panic]
    fn refcell_panic() {
        let refcell = RefCell::new(vec![42]);
        for _ in 0..refcell.borrow().unwrap().len() {
            refcell.borrow_mut().unwrap().push(42);
        }
    }
}
