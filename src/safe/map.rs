pub trait IteratorExt: Iterator {
    fn my_map<F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
    {
        Map::new(self, f)
    }
}

impl<T> IteratorExt for T where T: Iterator {}

pub struct Map<I, F> {
    iter: I,
    f: F,
}

impl<I, F> Map<I, F> {
    pub fn new(iter: I, f: F) -> Self {
        Self { iter, f }
    }

    pub fn into_inner(self) -> I {
        self.iter
    }
}

impl<I, F, R> Iterator for Map<I, F>
where
    I: Iterator,
    F: FnMut(I::Item) -> R,
{
    type Item = R;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(&mut self.f)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<I, F, R> DoubleEndedIterator for Map<I, F>
where
    I: DoubleEndedIterator,
    F: FnMut(I::Item) -> R,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter.next_back().map(&mut self.f)
    }
}

#[test]
fn test_my_map() {
    let v = vec![1, 2, 3];
    let mut iter = v.into_iter().my_map(|x| x * 2);
    assert_eq!(iter.next(), Some(2));
    assert_eq!(iter.next(), Some(4));
    assert_eq!(iter.next(), Some(6));
    assert_eq!(iter.next(), None);
}
