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

    pub fn push_front(&mut self, elem: T) {
        let new_node = unsafe {
            NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                front: None,
                back: self.head,
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

    pub fn len(&self) -> usize {
        self.len
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
}
