use core::ptr::NonNull;
use std::{fmt::Debug, hash::Hash};

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

    pub fn iter(&self) -> Iter<'_, T> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.into_iter()
    }

    pub fn clear(&mut self) {
        while self.pop_front().is_some() {
            // Continuously pop elements until the list is empty
        }
    }

    pub fn cursor_mut(&mut self) -> CursorMut<'_, T> {
        CursorMut {
            cur: None,
            list: self,
            index: None,
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
        IntoIter { list: self }
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while self.pop_front().is_some() {
            // Continuously pop elements until the list is empty
        }
    }
}

impl<T> Default for List<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone> Clone for List<T> {
    fn clone(&self) -> Self {
        let mut new_list = List::new();
        for elem in self {
            new_list.push_back(elem.clone());
        }
        new_list
    }
}

impl<T> Extend<T> for List<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for elem in iter {
            self.push_back(elem);
        }
    }
}

impl<T> FromIterator<T> for List<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut list = List::new();
        list.extend(iter);
        list
    }
}

impl<T: Debug> Debug for List<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

impl<T: PartialEq> PartialEq for List<T> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().eq(other.iter())
    }
}

impl<T: Eq> Eq for List<T> {}

impl<T: PartialOrd> PartialOrd for List<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.iter().partial_cmp(other.iter())
    }
}

impl<T: Ord> Ord for List<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.iter().cmp(other.iter())
    }
}

impl<T: Hash> Hash for List<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.len.hash(state);
        for elem in self.iter() {
            elem.hash(state);
        }
    }
}

unsafe impl<T: Send> Send for List<T> {}
unsafe impl<T: Sync> Sync for List<T> {}

unsafe impl<'a, T: Send> Send for Iter<'a, T> {}
unsafe impl<'a, T: Sync> Sync for Iter<'a, T> {}

unsafe impl<'a, T: Send> Send for IterMut<'a, T> {}
unsafe impl<'a, T: Sync> Sync for IterMut<'a, T> {}

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

pub struct CursorMut<'a, T> {
    cur: Link<T>,
    list: &'a mut List<T>,
    index: Option<usize>,
}

#[allow(dead_code)]
impl<'a, T> CursorMut<'a, T> {
    pub fn index(&self) -> Option<usize> {
        self.index
    }

    pub fn move_next(&mut self) {
        if let Some(cur) = self.cur {
            self.cur = unsafe { (*cur.as_ptr()).back };
            if self.cur.is_some() {
                *self.index.as_mut().unwrap() += 1;
            } else {
                self.index = None;
            }
        } else if !self.list.is_empty() {
            self.cur = self.list.head;
            self.index = Some(0);
        }
    }

    pub fn move_prev(&mut self) {
        if let Some(cur) = self.cur {
            self.cur = unsafe { (*cur.as_ptr()).front };
            if self.cur.is_some() {
                *self.index.as_mut().unwrap() -= 1;
            } else {
                self.index = None;
            }
        } else if !self.list.is_empty() {
            self.cur = self.list.tail;
            self.index = Some(self.list.len - 1);
        }
    }

    pub fn current(&mut self) -> Option<&mut T> {
        self.cur.map(|mut node| unsafe { &mut node.as_mut().elem })
    }

    pub fn peek_next(&mut self) -> Option<&mut T> {
        let next = if let Some(cur) = self.cur {
            unsafe { (*cur.as_ptr()).back }
        } else {
            self.list.head
        };
        next.map(|mut node| unsafe { &mut node.as_mut().elem })
    }

    pub fn peek_prev(&mut self) -> Option<&mut T> {
        let prev = if let Some(prev) = self.cur {
            unsafe { (*prev.as_ptr()).front }
        } else {
            self.list.tail
        };
        prev.map(|mut node| unsafe { &mut node.as_mut().elem })
    }

    pub fn split_before(&mut self) -> List<T> {
        if let Some(cur) = self.cur {
            let old_len = self.list.len;
            let old_index = self.index.unwrap();
            let prev = unsafe { (*cur.as_ptr()).front };

            let new_len = old_len - old_index;
            let new_front = self.cur;
            let new_back = self.list.tail;
            let new_index = Some(0);

            let output_len = old_len - new_len;
            let mut output_front = self.list.head;
            let output_back = prev;

            unsafe {
                if let Some(prev) = prev {
                    (*cur.as_ptr()).front = None;
                    (*prev.as_ptr()).back = None;
                } else {
                    output_front = None;
                }
            }

            self.list.len = new_len;
            self.list.head = new_front;
            self.list.tail = new_back;
            self.index = new_index;

            List {
                head: output_front,
                tail: output_back,
                len: output_len,
                _marker: std::marker::PhantomData,
            }
        } else {
            std::mem::take(self.list)
        }
    }

    pub fn split_after(&mut self) -> List<T> {
        if let Some(cur) = self.cur {
            let old_len = self.list.len;
            let old_index = self.index.unwrap();
            let next = unsafe { (*cur.as_ptr()).back };

            let new_len = old_index + 1;
            let new_back = self.cur;
            let new_front = self.list.head;
            let new_index = Some(old_index);

            let output_len = old_len - new_len;
            let output_front = next;
            let mut output_back = self.list.tail;

            unsafe {
                if let Some(next) = next {
                    (*cur.as_ptr()).back = None;
                    (*next.as_ptr()).front = None;
                } else {
                    output_back = None;
                }
            }

            self.list.len = new_len;
            self.list.head = new_front;
            self.list.tail = new_back;
            self.index = new_index;

            List {
                head: output_front,
                tail: output_back,
                len: output_len,
                _marker: std::marker::PhantomData,
            }
        } else {
            std::mem::take(self.list)
        }
    }

    fn splice_before(&mut self, mut input: List<T>) {
        if input.is_empty() {
            return;
        } else if let Some(cur) = self.cur {
            let input_head = input.head.take().unwrap();
            let input_tail = input.tail.take().unwrap();
            if let Some(prev) = unsafe { (*cur.as_ptr()).front } {
                unsafe {
                    (*prev.as_ptr()).back = Some(input_head);
                    (*input_head.as_ptr()).front = Some(prev);
                    (*cur.as_ptr()).front = Some(input_tail);
                    (*input_tail.as_ptr()).back = Some(cur);
                };
            } else {
                unsafe {
                    (*cur.as_ptr()).front = Some(input_tail);
                    (*input_tail.as_ptr()).back = Some(cur);
                    self.list.head = Some(input_head);
                }
            }
            *self.index.as_mut().unwrap() += input.len;
        } else if let Some(back) = self.list.tail {
            let input_head = input.head.take().unwrap();
            let input_tail = input.tail.take().unwrap();
            unsafe {
                (*back.as_ptr()).back = Some(input_head);
                (*input_head.as_ptr()).front = Some(back);
                self.list.tail = Some(input_tail);
            }
        } else {
            std::mem::swap(self.list, &mut input);
        }

        self.list.len += input.len;
        input.len = 0;
    }

    fn splice_after(&mut self, mut input: List<T>) {
        if input.is_empty() {
            return;
        } else if let Some(cur) = self.cur {
            let input_head = input.head.take().unwrap();
            let input_tail = input.tail.take().unwrap();
            if let Some(next) = unsafe { (*cur.as_ptr()).back } {
                unsafe {
                    (*next.as_ptr()).front = Some(input_tail);
                    (*input_tail.as_ptr()).back = Some(next);
                    (*cur.as_ptr()).back = Some(input_head);
                    (*input_head.as_ptr()).front = Some(cur);
                };
            } else {
                unsafe {
                    (*cur.as_ptr()).back = Some(input_head);
                    (*input_head.as_ptr()).front = Some(cur);
                    self.list.tail = Some(input_tail);
                }
            }
            *self.index.as_mut().unwrap() += input.len;
        } else if let Some(front) = self.list.head {
            let input_head = input.head.take().unwrap();
            let input_tail = input.tail.take().unwrap();
            unsafe {
                (*front.as_ptr()).front = Some(input_tail);
                (*input_tail.as_ptr()).back = Some(front);
                self.list.head = Some(input_head);
            }
        } else {
            std::mem::swap(self.list, &mut input);
        }

        self.list.len += input.len;
        input.len = 0;
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

    #[test]
    fn test_list_clone() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let cloned_list = list.clone();
        assert_eq!(cloned_list.len(), 3);
        assert_eq!(cloned_list.front(), Some(&1));
        assert_eq!(cloned_list.back(), Some(&3));
    }

    #[test]
    fn test_list_default() {
        let list: List<i32> = List::default();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);
        assert_eq!(list.front(), None);
        assert_eq!(list.back(), None);
    }

    #[test]
    fn test_list_extend() {
        let mut list = List::new();
        list.extend(vec![1, 2, 3]);
        assert_eq!(list.len(), 3);
        assert_eq!(list.front(), Some(&1));
        assert_eq!(list.back(), Some(&3));
    }

    #[test]
    fn test_list_from_iter() {
        let list: List<i32> = vec![1, 2, 3].into_iter().collect();
        assert_eq!(list.len(), 3);
        assert_eq!(list.front(), Some(&1));
        assert_eq!(list.back(), Some(&3));
    }

    #[test]
    fn test_list_partial_eq() {
        let mut list1 = List::new();
        list1.push_back(1);
        list1.push_back(2);

        let mut list2 = List::new();
        list2.push_back(1);
        list2.push_back(2);

        assert_eq!(list1, list2);
        assert_ne!(list1, List::new());
    }

    #[test]
    fn test_list_partial_ord() {
        let mut list1 = List::new();
        list1.push_back(1);
        list1.push_back(2);

        let mut list2 = List::new();
        list2.push_back(1);
        list2.push_back(3);

        assert!(list1 < list2);
        assert!(list2 > list1);
        assert!(list1 <= list2);
        assert!(list2 >= list1);

        let mut list3 = List::new();
        list3.push_back(1);
        list3.push_back(2);

        assert_eq!(list1, list3);
    }

    #[test]
    fn test_list_ord() {
        let mut list1 = List::new();
        list1.push_back(1);
        list1.push_back(2);

        let mut list2 = List::new();
        list2.push_back(1);
        list2.push_back(3);

        assert!(list1 < list2);
        assert!(list2 > list1);
        assert!(list1 <= list2);
        assert!(list2 >= list1);

        let mut list3 = List::new();
        list3.push_back(1);
        list3.push_back(2);

        assert_eq!(list1, list3);
    }

    #[test]
    fn test_list_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let mut hasher = DefaultHasher::new();
        list.hash(&mut hasher);
        let hash1 = hasher.finish();

        let mut list2 = List::new();
        list2.push_back(1);
        list2.push_back(2);
        list2.push_back(3);

        let mut hasher2 = DefaultHasher::new();
        list2.hash(&mut hasher2);
        let hash2 = hasher2.finish();

        assert_eq!(hash1, hash2);

        let mut map = std::collections::HashMap::new();
        let list1 = (1..10).collect::<List<i32>>();
        let list2 = (10..20).collect::<List<i32>>();

        assert_eq!(map.insert(list1.clone(), "list1"), None);
        assert_eq!(map.insert(list2.clone(), "list2"), None);

        assert_eq!(map.len(), 2);

        assert_eq!(map.get(&list1), Some(&"list1"));
        assert_eq!(map.get(&list2), Some(&"list2"));

        assert_eq!(map.remove(&list1), Some("list1"));
        assert_eq!(map.remove(&list2), Some("list2"));

        assert!(map.is_empty());
    }

    #[test]
    fn test_debug() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);

        let debug_str = format!("{:?}", list);
        assert_eq!(debug_str, "[1, 2, 3]");
    }

    #[test]
    #[allow(dead_code)]
    fn test_list_send_sync() {
        fn assert_properties() {
            fn is_send<T: Send>() {}
            fn is_sync<T: Sync>() {}

            is_send::<List<i32>>();
            is_sync::<List<i32>>();

            is_send::<IntoIter<i32>>();
            is_sync::<IntoIter<i32>>();

            is_send::<Iter<i32>>();
            is_sync::<Iter<i32>>();

            is_send::<IterMut<i32>>();
            is_sync::<IterMut<i32>>();

            fn list_covariant<'a, T>(x: List<&'static T>) -> List<&'a T> {
                x
            }
            fn iter_covariant<'i, 'a, T>(x: Iter<'i, &'static T>) -> Iter<'i, &'a T> {
                x
            }
            fn into_iter_covariant<'a, T>(x: IntoIter<&'static T>) -> IntoIter<&'a T> {
                x
            }

            /// ```compile_fail,E0308
            /// use linked_list::IterMut;
            ///
            /// fn iter_mut_covariant<'i, 'a, T>(x: IterMut<'i, &'static T>) -> IterMut<'i, &'a T> { x }
            /// ```
            fn iter_mut_invariant() {}
        }
        assert_properties();
    }

    #[test]
    fn test_cursor_mut() {
        let mut m: List<u32> = List::new();
        m.extend([1, 2, 3, 4, 5, 6]);
        let mut cursor = m.cursor_mut();

        cursor.move_next();
        assert_eq!(cursor.current(), Some(&mut 1));
        assert_eq!(cursor.peek_next(), Some(&mut 2));
        assert_eq!(cursor.peek_prev(), None);
        assert_eq!(cursor.index(), Some(0));

        cursor.move_prev();
        assert_eq!(cursor.current(), None);
        assert_eq!(cursor.peek_next(), Some(&mut 1));
    }

    #[test]
    fn test_cursor_move_peek() {
        let mut m: List<u32> = List::new();
        m.extend([1, 2, 3, 4, 5, 6]);
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        assert_eq!(cursor.current(), Some(&mut 1));
        assert_eq!(cursor.peek_next(), Some(&mut 2));
        assert_eq!(cursor.peek_prev(), None);
        assert_eq!(cursor.index(), Some(0));
        cursor.move_prev();
        assert_eq!(cursor.current(), None);
        assert_eq!(cursor.peek_next(), Some(&mut 1));
        assert_eq!(cursor.peek_prev(), Some(&mut 6));
        assert_eq!(cursor.index(), None);
        cursor.move_next();
        cursor.move_next();
        assert_eq!(cursor.current(), Some(&mut 2));
        assert_eq!(cursor.peek_next(), Some(&mut 3));
        assert_eq!(cursor.peek_prev(), Some(&mut 1));
        assert_eq!(cursor.index(), Some(1));

        let mut cursor = m.cursor_mut();
        cursor.move_prev();
        assert_eq!(cursor.current(), Some(&mut 6));
        assert_eq!(cursor.peek_next(), None);
        assert_eq!(cursor.peek_prev(), Some(&mut 5));
        assert_eq!(cursor.index(), Some(5));
        cursor.move_next();
        assert_eq!(cursor.current(), None);
        assert_eq!(cursor.peek_next(), Some(&mut 1));
        assert_eq!(cursor.peek_prev(), Some(&mut 6));
        assert_eq!(cursor.index(), None);
        cursor.move_prev();
        cursor.move_prev();
        assert_eq!(cursor.current(), Some(&mut 5));
        assert_eq!(cursor.peek_next(), Some(&mut 6));
        assert_eq!(cursor.peek_prev(), Some(&mut 4));
        assert_eq!(cursor.index(), Some(4));
    }

    #[test]
    fn test_cursor_mut_insert() {
        let mut m: List<u32> = List::new();
        m.extend([1, 2, 3, 4, 5, 6]);
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.splice_before(Some(7).into_iter().collect());
        cursor.splice_after(Some(8).into_iter().collect());
        // check_links(&m);
        assert_eq!(
            m.iter().cloned().collect::<Vec<_>>(),
            &[7, 1, 8, 2, 3, 4, 5, 6]
        );
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.move_prev();
        cursor.splice_before(Some(9).into_iter().collect());
        cursor.splice_after(Some(10).into_iter().collect());
        check_links(&m);
        assert_eq!(
            m.iter().cloned().collect::<Vec<_>>(),
            &[10, 7, 1, 8, 2, 3, 4, 5, 6, 9]
        );

        /* remove_current not impl'd
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.move_prev();
        assert_eq!(cursor.remove_current(), None);
        cursor.move_next();
        cursor.move_next();
        assert_eq!(cursor.remove_current(), Some(7));
        cursor.move_prev();
        cursor.move_prev();
        cursor.move_prev();
        assert_eq!(cursor.remove_current(), Some(9));
        cursor.move_next();
        assert_eq!(cursor.remove_current(), Some(10));
        check_links(&m);
        assert_eq!(m.iter().cloned().collect::<Vec<_>>(), &[1, 8, 2, 3, 4, 5, 6]);
        */

        let mut m: List<u32> = List::new();
        m.extend([1, 8, 2, 3, 4, 5, 6]);
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        let mut p: List<u32> = List::new();
        p.extend([100, 101, 102, 103]);
        let mut q: List<u32> = List::new();
        q.extend([200, 201, 202, 203]);
        cursor.splice_after(p);
        cursor.splice_before(q);
        check_links(&m);
        assert_eq!(
            m.iter().cloned().collect::<Vec<_>>(),
            &[200, 201, 202, 203, 1, 100, 101, 102, 103, 8, 2, 3, 4, 5, 6]
        );
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.move_prev();
        let tmp = cursor.split_before();
        assert_eq!(m.into_iter().collect::<Vec<_>>(), &[]);
        m = tmp;
        let mut cursor = m.cursor_mut();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        cursor.move_next();
        let tmp = cursor.split_after();
        assert_eq!(
            tmp.into_iter().collect::<Vec<_>>(),
            &[102, 103, 8, 2, 3, 4, 5, 6]
        );
        check_links(&m);
        assert_eq!(
            m.iter().cloned().collect::<Vec<_>>(),
            &[200, 201, 202, 203, 1, 100, 101]
        );
    }

    fn check_links<T: Eq + std::fmt::Debug>(list: &List<T>) {
        let from_front: Vec<_> = list.iter().collect();
        let from_back: Vec<_> = list.iter().rev().collect();
        let re_reved: Vec<_> = from_back.into_iter().rev().collect();

        assert_eq!(from_front, re_reved);
    }
}
