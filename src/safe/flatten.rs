pub struct Flatten<O>
where
    O: Iterator,
    O::Item: IntoIterator,
{
    pub outer: O,
    pub inner: Option<<O::Item as IntoIterator>::IntoIter>,
}

impl<O> Flatten<O>
where
    O: Iterator,
    O::Item: IntoIterator,
{
    pub fn new(outer: O) -> Self {
        Self { outer, inner: None }
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
            if let Some(ref mut inner) = self.inner {
                if let Some(item) = inner.next() {
                    break Some(item);
                } else {
                    self.inner = None;
                }
            }
            let Some(next_outer) = self.outer.next() else {
                break None;
            };
            self.inner = Some(next_outer.into_iter());
        }
    }
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
}
