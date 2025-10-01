pub trait IteratorExt: Iterator {
    fn flatten(self) -> Flatten<Self>
    where
        Self: Sized,
        Self::Item: IntoIterator,
    {
        Flatten::new(self)
    }
}

impl<T> IteratorExt for T where T: Iterator {}

pub struct Flatten<O>
where
    O: Iterator,
    O::Item: IntoIterator,
{
    pub outer: O,
    pub front_iter: Option<<O::Item as IntoIterator>::IntoIter>,
    pub back_iter: Option<<O::Item as IntoIterator>::IntoIter>,
}

impl<O> Flatten<O>
where
    O: Iterator,
    O::Item: IntoIterator,
{
    pub fn new(outer: O) -> Self {
        Self {
            outer,
            front_iter: None,
            back_iter: None,
        }
    }
}

impl<O> Iterator for Flatten<O>
where
    O: Iterator,
    O::Item: IntoIterator,
{
    type Item = <O::Item as IntoIterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut inner) = self.front_iter {
                if let Some(item) = inner.next() {
                    break Some(item);
                }
                self.front_iter = None;
            }
            if let Some(next_outer) = self.outer.next() {
                self.front_iter = Some(next_outer.into_iter());
            } else {
                break self.back_iter.as_mut()?.next();
            }
        }
    }
}

impl<O> DoubleEndedIterator for Flatten<O>
where
    O: DoubleEndedIterator,
    O::Item: IntoIterator,
    <O::Item as IntoIterator>::IntoIter: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut inner) = self.back_iter {
                if let Some(item) = inner.next_back() {
                    break Some(item);
                }
                self.back_iter = None;
            }
            if let Some(next_outer) = self.outer.next_back() {
                self.back_iter = Some(next_outer.into_iter());
            } else {
                break self.front_iter.as_mut()?.next_back();
            }
        }
    }
}

pub fn flatten<O, T>(outer: O) -> Flatten<T>
where
    O: IntoIterator<IntoIter = T>,
    T: Iterator,
    T::Item: IntoIterator,
{
    Flatten::new(outer.into_iter())
}

#[cfg(test)]
mod tests {
    #[test]
    pub fn count() {
        let v = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let iter = super::Flatten::new(v.into_iter());
        assert_eq!(iter.count(), 6);
        let v = vec![vec![1, 2, 3], vec![], vec![4, 5, 6]];
        let iter = super::Flatten::new(v.into_iter());
        assert_eq!(iter.count(), 6);
        let v: Vec<Vec<i32>> = vec![];
        let iter = super::Flatten::new(v.into_iter());
        assert_eq!(iter.count(), 0);
    }

    #[test]
    pub fn flatten_empty_inner() {
        let v = vec![vec![1], vec![], vec![3]];
        let mut iter = super::flatten(v);
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), None);
    }

    #[test]
    pub fn flatten_test() {
        let v = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let mut iter = super::Flatten::new(v.into_iter());
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), Some(5));
        assert_eq!(iter.next(), Some(6));
        assert_eq!(iter.next(), None);
    }

    #[test]
    pub fn flatten_double_ended() {
        let v = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let mut iter = super::Flatten::new(v.into_iter());
        assert_eq!(iter.next_back(), Some(6));
        assert_eq!(iter.next_back(), Some(5));
        assert_eq!(iter.next_back(), Some(4));
        assert_eq!(iter.next_back(), Some(3));
        assert_eq!(iter.next_back(), Some(2));
        assert_eq!(iter.next_back(), Some(1));
        assert_eq!(iter.next_back(), None);
    }

    #[test]
    pub fn flatten_double_ended_mixed() {
        let v = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let mut iter = super::Flatten::new(v.into_iter());
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next_back(), Some(6));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next_back(), Some(5));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next_back(), Some(4));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    pub fn infinite_iter() {
        let v = (1..).map(|i| 0..i);
        let mut iter = super::Flatten::new(v);
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(0));
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(0));
    }

    #[test]
    pub fn iter_flatten() {
        let v = vec![vec![1, 2, 3], vec![4, 5, 6]];
        let mut iter = v.into_iter().flatten();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next_back(), Some(6));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), Some(5));
        assert_eq!(iter.next(), None);
    }
}
