use std::{
    alloc::{Layout, handle_alloc_error},
    fmt::Debug,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

#[allow(dead_code)]
pub struct Box<T: ?Sized> {
    inner: NonNull<T>,
    _marker: std::marker::PhantomData<T>,
}

impl<T: Sized> Box<T> {
    pub fn new(value: T) -> Self {
        // Allocate memory for T on the heap
        // and write the value into that memory.
        let layout = std::alloc::Layout::new::<T>();

        let ptr = if layout.size() == 0 {
            NonNull::dangling()
        } else {
            match NonNull::new(unsafe { std::alloc::alloc(layout) as *mut T }) {
                Some(non_null_ptr) => non_null_ptr,
                None => handle_alloc_error(layout),
            }
        };

        unsafe { ptr.as_ptr().write(value) };

        Self {
            inner: ptr,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn into_inner(self) -> T {
        // This consumes the Box and returns the inner value
        let ptr = self.inner.as_ptr();
        let value = unsafe {
            // Use `std::ptr::read` to read the value without dropping it
            std::ptr::read(ptr)
        };
        let boxed = std::mem::ManuallyDrop::new(self);
        if std::mem::size_of::<T>() != 0 {
            unsafe {
                std::alloc::dealloc(
                    boxed.inner.as_ptr() as *mut u8,
                    std::alloc::Layout::new::<T>(),
                );
            }
        }
        value
    }
}

impl<T: ?Sized> Box<T> {
    /// # Safety
    ///
    /// - `ptr` must come from `Box<T>::into_raw`
    /// - cannot double call this function
    pub unsafe fn from_raw(ptr: *mut T) -> Self {
        let non_null_ptr = NonNull::new(ptr).expect("Non-null pointer expected");
        Self {
            inner: non_null_ptr,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn into_raw(self) -> *mut T {
        let ptr = self.inner.as_ptr();
        std::mem::forget(self); // Prevent the destructor from running
        ptr
    }

    pub fn as_ptr(&self) -> *const T {
        self.inner.as_ptr()
    }

    pub fn as_mut_ptr(&mut self) -> *mut T {
        self.inner.as_ptr()
    }
}

impl<T> From<T> for Box<T> {
    fn from(value: T) -> Self {
        Box::new(value)
    }
}

impl<T> AsRef<T> for Box<T> {
    fn as_ref(&self) -> &T {
        self
    }
}

impl<T> AsMut<T> for Box<T> {
    fn as_mut(&mut self) -> &mut T {
        self
    }
}

impl<T: ?Sized> Deref for Box<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { self.inner.as_ref() }
    }
}

impl<T: ?Sized> DerefMut for Box<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.inner.as_mut() }
    }
}

impl<T: Clone + Sized> Clone for Box<T> {
    fn clone(&self) -> Self {
        Box::new((**self).clone())
    }
}

unsafe impl<T: ?Sized + Send> Send for Box<T> {}
unsafe impl<T: ?Sized + Sync> Sync for Box<T> {}

impl<T: PartialEq + ?Sized> PartialEq for Box<T> {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}

impl<T: Eq + ?Sized> Eq for Box<T> {}

impl<T: Debug + ?Sized> Debug for Box<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized> Drop for Box<T> {
    fn drop(&mut self) {
        // Deallocate the memory for T
        unsafe {
            // Use `std::ptr::drop_in_place` to call the destructor of T
            std::ptr::drop_in_place(self.inner.as_ptr());
            let layout = Layout::for_value(&*self.inner.as_ptr());
            if layout.size() != 0 {
                std::alloc::dealloc(self.inner.as_ptr() as *mut u8, layout);
            }
        }
    }
}

#[allow(dead_code)]
#[cfg(test)]
mod test {
    use std::fmt::Debug;

    use crate::r#box::Box;

    #[test]
    fn test_new() {
        let a = Box::new(String::from("value"));
        println!("{a:#?}");
        let b = String::from("value");
        println!("{b:#?}");

        #[derive(Debug)]
        struct TestStruct {
            a: i32,
            b: String,
        }

        impl Drop for TestStruct {
            fn drop(&mut self) {
                println!("Dropping TestStruct with a: {}, b: {}", self.a, self.b);
            }
        }

        let test_struct = TestStruct {
            a: 42,
            b: String::from("Hello"),
        };

        println!("{test_struct:?}");
        // Box the struct
        // This will allocate the struct on the heap
        let boxed_struct = Box::new(test_struct);
        println!("{boxed_struct:?}");
    }

    #[test]
    fn test_into_inner() {
        let b: Box<String> = Box::new("foo".into());
        let s: String = b.into_inner();
        assert_eq!(s, "foo");
    }

    #[test]
    fn test_as_ref() {
        let b: Box<String> = Box::new("foo".into());
        assert_eq!(b.as_ref(), "foo");
    }

    #[test]
    fn test_as_mut() {
        let mut b: Box<String> = Box::new("foo".into());
        b.as_mut().push_str("bar");
        assert_eq!(b.as_ref(), "foobar");
    }

    #[test]
    fn test_from_raw() {
        let b: Box<String> = Box::new("foo".into());
        let raw_ptr = b.into_raw();
        let b2: Box<String> = unsafe { Box::from_raw(raw_ptr) };
        assert_eq!(b2.as_ref(), "foo");
    }

    #[test]
    fn test_into_raw() {
        let b: Box<String> = Box::new("foo".into());
        let raw_ptr = b.into_raw();
        assert!(!raw_ptr.is_null());
        // Ensure that the Box is no longer valid after calling into_raw
        // This is a safety check, not a runtime check
        unsafe { Box::from_raw(raw_ptr) }; // Convert back to Box to avoid memory leak
        // This should not panic, as we are converting back to Box<String>
    }

    #[test]
    fn test_as_ptr() {
        let b: Box<String> = Box::new("foo".into());
        let ptr = b.as_ptr();
        assert!(!ptr.is_null());
        unsafe {
            assert_eq!(*ptr, "foo");
        }
    }

    #[test]
    fn test_as_mut_ptr() {
        let mut b: Box<String> = Box::new("foo".into());
        let ptr = b.as_mut_ptr();
        assert!(!ptr.is_null());
        unsafe {
            *ptr = "bar".into();
            assert_eq!(*ptr, "bar");
        }
    }

    #[test]
    fn test_clone() {
        let b1: Box<String> = Box::new("foo".into());
        let b2 = b1.clone();
        assert_eq!(b1.as_ref(), "foo");
        assert_eq!(b2.as_ref(), "foo");
    }

    #[test]
    fn test_deref() {
        let b: Box<String> = Box::new("foo".into());
        assert_eq!(&*b, "foo");
    }

    #[test]
    fn test_deref_mut() {
        let mut b: Box<String> = Box::new("foo".into());
        *b = "bar".into();
        assert_eq!(&*b, "bar");
    }

    #[test]
    fn test_partial_eq() {
        let b1: Box<String> = Box::new("foo".into());
        let b2: Box<String> = Box::new("foo".into());
        assert!(b1 == b2);
    }

    #[test]
    fn test_eq() {
        let b1: Box<String> = Box::new("foo".into());
        let b2: Box<String> = Box::new("foo".into());
        assert!(b1 == b2);
    }

    #[test]
    fn test_debug() {
        let b: Box<String> = Box::new("foo".into());
        let debug_str = format!("{:?}", b);
        assert_eq!(debug_str, "\"foo\"");
    }

    #[test]
    fn test_drop() {
        {
            let _b: Box<String> = Box::new("foo".into());
            // The drop should happen automatically at the end of this scope
        } // Here we can check if the drop message is printed
        // This is a manual check, as Rust does not provide a way to assert drop behavior
    }

    #[test]
    fn test_boxed_struct() {
        #[derive(Debug)]
        struct MyStruct {
            value: i32,
        }

        impl Drop for MyStruct {
            fn drop(&mut self) {
                println!("Dropping MyStruct with value: {}", self.value);
            }
        }

        let boxed_struct = Box::new(MyStruct { value: 42 });
        println!("{:?}", boxed_struct);
        // The drop will happen automatically at the end of this scope
    }

    #[test]
    fn test_boxed_struct_drop() {
        #[derive(Debug)]
        struct MyStruct {
            value: i32,
        }

        impl Drop for MyStruct {
            fn drop(&mut self) {
                println!("Dropping MyStruct with value: {}", self.value);
            }
        }

        {
            let _boxed_struct = Box::new(MyStruct { value: 42 });
            // The drop will happen automatically at the end of this scope
        } // Here we can check if the drop message is printed
    }

    #[test]
    fn test_boxed_struct_clone() {
        #[derive(Debug, Clone)]
        struct MyStruct {
            value: i32,
        }

        let boxed_struct1 = Box::new(MyStruct { value: 42 });
        let boxed_struct2 = boxed_struct1.clone();
        assert_eq!(boxed_struct1.as_ref().value, boxed_struct2.as_ref().value);
    }

    #[test]
    fn test_boxed_struct_partial_eq() {
        #[derive(Debug, PartialEq)]
        struct MyStruct {
            value: i32,
        }

        let boxed_struct1 = Box::new(MyStruct { value: 42 });
        let boxed_struct2 = Box::new(MyStruct { value: 42 });
        assert!(boxed_struct1 == boxed_struct2);
    }

    #[test]
    fn test_zst() {
        #[derive(Debug)]
        struct MyZST;

        let boxed_zst = Box::new(MyZST);
        println!("{:?}", boxed_zst);
        // The drop will happen automatically at the end of this scope
    }

    #[test]
    fn test_zst_clone() {
        #[derive(Debug, Clone, PartialEq)]
        struct MyZST;

        let boxed_zst1 = Box::new(MyZST);
        let boxed_zst2 = boxed_zst1.clone();
        assert_eq!(boxed_zst1.as_ref(), boxed_zst2.as_ref());
    }

    #[test]
    fn test_dyn() {
        trait MyTrait {
            fn do_something(&self);
        }

        struct MyStruct;

        impl MyTrait for MyStruct {
            fn do_something(&self) {
                println!("Doing something in MyStruct");
            }
        }

        impl Drop for MyStruct {
            fn drop(&mut self) {
                println!("Dropping MyStruct");
            }
        }

        let boxed_struct: Box<MyStruct> = Box::new(MyStruct);
        let boxed_dyn: Box<dyn MyTrait> =
            unsafe { Box::from_raw(Box::into_raw(boxed_struct) as *mut dyn MyTrait) };
        boxed_dyn.do_something();
        // The drop will happen automatically at the end of this scope
    }
}
