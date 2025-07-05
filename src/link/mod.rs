use core::ptr::NonNull;

pub struct List<T> {
    head: Link<T>,
    tail: Link<T>,
    len: usize,
    _marker: std::marker::PhantomData<T>,
}

type Link<T> = Option<NonNull<Node<T>>>;

struct Node<T> {
    front: Link<T>,
    back: Link<T>,
    elem: T,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: None,
            tail: None,
            len: 0,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn front(&self) -> Option<&T> {
        self.head.map(|node| unsafe { &node.as_ref().elem })
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        self.head.map(|mut node| unsafe { &mut node.as_mut().elem })
    }

    pub fn push_front(&mut self, elem: T) {
        let new_node = unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                front: None,
                back: None,
                elem,
            })))
        };
        if let Some(old_head) = self.head {
            unsafe {
                (*old_head.as_ptr()).front = Some(new_node);
                (*new_node.as_ptr()).back = Some(old_head);
            }
        } else {
            self.tail = Some(new_node);
        }
        self.head = Some(new_node);
        self.len += 1;
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.map(|node| {
            self.len -= 1;

            let node = unsafe { Box::from_raw(node.as_ptr()) };
            let elem = node.elem;

            self.head = node.back;
            if let Some(new_head) = self.head {
                unsafe {
                    (*new_head.as_ptr()).front = None;
                }
            } else {
                self.tail = None;
            }
            elem
        })
    }

    pub fn back(&self) -> Option<&T> {
        self.tail.map(|node| unsafe { &node.as_ref().elem })
    }

    pub fn back_mut(&mut self) -> Option<&mut T> {
        self.tail.map(|mut node| unsafe { &mut node.as_mut().elem })
    }

    pub fn push_back(&mut self, elem: T) {
        let new_node = unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                front: None,
                back: None,
                elem,
            })))
        };

        if let Some(old_tail) = self.tail {
            unsafe {
                (*old_tail.as_ptr()).back = Some(new_node);
                (*new_node.as_ptr()).front = Some(old_tail);
            }
        } else {
            self.head = Some(new_node);
        }
        self.tail = Some(new_node);
        self.len += 1;
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.map(|node| {
            self.len -= 1;

            let node = unsafe { Box::from_raw(node.as_ptr()) };
            let elem = node.elem;

            self.tail = node.front;
            if let Some(new_tail) = self.tail {
                unsafe {
                    (*new_tail.as_ptr()).back = None;
                }
            } else {
                self.head = None;
            }

            elem
        })
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn iter(&self) -> Iter<T> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<T> {
        self.into_iter()
    }

    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter { list: self }
    }

    pub fn clear(&mut self) {
        while let Some(_) = self.pop_front() {
            // Continuously pop elements until the list is empty
        }
    }
}

impl<'a, T> IntoIterator for &'a List<T> {
    type Item = &'a T;

    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            front: self.head,
            back: self.tail,
            len: self.len,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<'a, T> IntoIterator for &'a mut List<T> {
    type Item = &'a mut T;

    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            front: self.head,
            back: self.tail,
            len: self.len,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> IntoIterator for List<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop_front() {
            // Continuously pop elements until the list is empty
        }
    }
}

pub struct Iter<'a, T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len != 0 {
            self.front.map(|node| {
                self.len -= 1;
                self.front = unsafe { (*node.as_ptr()).back };
                unsafe { &node.as_ref().elem }
            })
        } else {
            None
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len != 0 {
            self.back.map(|node| {
                self.len -= 1;
                self.back = unsafe { (*node.as_ptr()).front };
                unsafe { &node.as_ref().elem }
            })
        } else {
            None
        }
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

pub struct IterMut<'a, T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    _marker: std::marker::PhantomData<&'a mut T>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len != 0 {
            self.front.map(|mut node| {
                self.len -= 1;
                self.front = unsafe { (*node.as_ptr()).back };
                unsafe { &mut node.as_mut().elem }
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len != 0 {
            self.back.map(|mut node| {
                self.len -= 1;
                self.back = unsafe { (*node.as_ptr()).front };
                unsafe { &mut node.as_mut().elem }
            })
        } else {
            None
        }
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

pub struct IntoIter<T> {
    list: List<T>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.list.len(), Some(self.list.len()))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.list.pop_back()
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.list.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list() {
        let mut list = List::new();
        assert_eq!(list.len(), 0);

        list.push_front(1);
        assert_eq!(list.len(), 1);

        list.push_front(2);
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_push_pop() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_back(), Some(2));
        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_front_back() {
        let mut list = List::new();
        list.push_front(1);
        list.push_back(2);
        assert_eq!(list.front(), Some(&1));
        assert_eq!(list.back(), Some(&2));
        assert_eq!(list.len(), 2);

        if let Some(front) = list.front_mut() {
            *front = 3;
        }
        if let Some(back) = list.back_mut() {
            *back = 4;
        }
        assert_eq!(list.front(), Some(&3));
        assert_eq!(list.back(), Some(&4));
    }

    #[test]
    fn test_empty_list() {
        let mut list: List<i32> = List::new();
        assert!(list.is_empty());
        assert_eq!(list.front(), None);
        assert_eq!(list.back(), None);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.pop_back(), None);
    }

    #[test]
    fn test_iter() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_into_iter() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let mut into_iter = list.into_iter();
        assert_eq!(into_iter.next(), Some(1));
        assert_eq!(into_iter.next(), Some(2));
        assert_eq!(into_iter.next(), Some(3));
        assert_eq!(into_iter.next(), None);
    }

    #[test]
    fn test_clear() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        assert_eq!(list.len(), 3);

        list.clear();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert_eq!(list.front(), None);
        assert_eq!(list.back(), None);
    }

    #[test]
    fn test_size_hint() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let mut iter = list.iter();
        assert_eq!(iter.size_hint(), (3, Some(3)));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.size_hint(), (2, Some(2)));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.size_hint(), (1, Some(1)));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.size_hint(), (0, Some(0)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_double_ended_iter() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let mut iter = list.iter();
        assert_eq!(iter.next_back(), Some(&3));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next_back(), Some(&2));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_exact_size_iter() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let mut iter = list.iter();
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next_back(), Some(&3));
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_into_iter_size_hint() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let mut into_iter = list.into_iter();
        assert_eq!(into_iter.size_hint(), (3, Some(3)));
        assert_eq!(into_iter.next(), Some(1));
        assert_eq!(into_iter.size_hint(), (2, Some(2)));
        assert_eq!(into_iter.next(), Some(2));
        assert_eq!(into_iter.size_hint(), (1, Some(1)));
        assert_eq!(into_iter.next(), Some(3));
        assert_eq!(into_iter.size_hint(), (0, Some(0)));
        assert_eq!(into_iter.next(), None);
    }

    #[test]
    fn test_into_iter_double_ended() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let mut into_iter = list.into_iter();
        assert_eq!(into_iter.next_back(), Some(3));
        assert_eq!(into_iter.next(), Some(1));
        assert_eq!(into_iter.next_back(), Some(2));
        assert_eq!(into_iter.next(), None);
    }

    #[test]
    fn test_into_iter_exact_size() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let mut into_iter = list.into_iter();
        assert_eq!(into_iter.len(), 3);
        assert_eq!(into_iter.next(), Some(1));
        assert_eq!(into_iter.len(), 2);
        assert_eq!(into_iter.next_back(), Some(3));
        assert_eq!(into_iter.len(), 1);
        assert_eq!(into_iter.next(), Some(2));
        assert_eq!(into_iter.len(), 0);
        assert_eq!(into_iter.next(), None);
    }

    #[test]
    fn test_list_with_drop() {
        struct DropItem(i32);

        impl Drop for DropItem {
            fn drop(&mut self) {
                println!("Dropping {}", self.0);
            }
        }

        {
            let mut list = List::new();
            list.push_front(DropItem(1));
            list.push_back(DropItem(2));
            assert_eq!(list.len(), 2);
        } // List goes out of scope and should drop all elements

        let list: List<i32> = List::new();
        assert!(list.is_empty());
    }

    #[test]
    fn test_iter_mut() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        for elem in list.iter_mut() {
            *elem *= 2;
        }

        let mut iter = list.iter();
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), Some(&6));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_iter_mut_size_hint() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let mut iter_mut = list.iter_mut();
        assert_eq!(iter_mut.size_hint(), (3, Some(3)));
        assert_eq!(iter_mut.next(), Some(&mut 1));
        assert_eq!(iter_mut.size_hint(), (2, Some(2)));
        assert_eq!(iter_mut.next(), Some(&mut 2));
        assert_eq!(iter_mut.size_hint(), (1, Some(1)));
        assert_eq!(iter_mut.next(), Some(&mut 3));
        assert_eq!(iter_mut.size_hint(), (0, Some(0)));
        assert_eq!(iter_mut.next(), None);
    }

    #[test]
    fn test_iter_mut_double_ended() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let mut iter_mut = list.iter_mut();
        assert_eq!(iter_mut.next_back(), Some(&mut 3));
        assert_eq!(iter_mut.next(), Some(&mut 1));
        assert_eq!(iter_mut.next_back(), Some(&mut 2));
        assert_eq!(iter_mut.next(), None);
    }

    #[test]
    fn test_iter_mut_exact_size() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let mut iter_mut = list.iter_mut();
        assert_eq!(iter_mut.len(), 3);
        assert_eq!(iter_mut.next(), Some(&mut 1));
        assert_eq!(iter_mut.len(), 2);
        assert_eq!(iter_mut.next_back(), Some(&mut 3));
        assert_eq!(iter_mut.len(), 1);
        assert_eq!(iter_mut.next(), Some(&mut 2));
        assert_eq!(iter_mut.len(), 0);
        assert_eq!(iter_mut.next(), None);
    }
}
