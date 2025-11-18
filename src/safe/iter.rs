pub struct StrSplit<'haystack, D> {
    remainder: Option<&'haystack str>,
    delimiter: D,
}

trait Delimiter {
    fn find_next(&self, haystack: &str) -> Option<(usize, usize)>;
}

impl Delimiter for &str {
    fn find_next(&self, haystack: &str) -> Option<(usize, usize)> {
        haystack.find(self).map(|index| (index, index + self.len()))
    }
}

impl Delimiter for char {
    fn find_next(&self, haystack: &str) -> Option<(usize, usize)> {
        haystack
            .find(*self)
            .map(|index| (index, index + self.len_utf8()))
    }
}

impl<'haystack, D: Delimiter> Iterator for StrSplit<'haystack, D> {
    type Item = &'haystack str;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(remainder) = self.remainder {
            let Some((start, end)) = self.delimiter.find_next(remainder) else {
                return self.remainder.take();
            };
            self.remainder = Some(&remainder[end..]);
            Some(&remainder[..start])
        } else {
            None
        }
    }
}

pub trait IteratorExt: Iterator {
    fn my_flatten(self) -> Flatten<Self>
    where
        Self: Sized,
        Self::Item: IntoIterator,
    {
        Flatten::new(self)
    }

    fn my_map<F>(self, f: F) -> Map<Self, F>
    where
        Self: Sized,
    {
        Map::new(self, f)
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

#[cfg(test)]
mod tests {
    use crate::safe::iter::IteratorExt;

    #[test]
    pub fn split_test() {
        let s = "hello world,this is rust";
        let mut iter = super::StrSplit {
            remainder: Some(s),
            delimiter: " ",
        };
        assert_eq!(iter.next(), Some("hello"));
        assert_eq!(iter.next(), Some("world,this"));
        assert_eq!(iter.next(), Some("is"));
        assert_eq!(iter.next(), Some("rust"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    pub fn split_no_delimiter() {
        let s = "hello";
        let mut iter = super::StrSplit {
            remainder: Some(s),
            delimiter: " ",
        };
        assert_eq!(iter.next(), Some("hello"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    pub fn it_works() {
        let haystack = "a b c d e";
        let delimiter = " ";
        let mut splitter = super::StrSplit {
            remainder: Some(haystack),
            delimiter,
        };

        assert_eq!(splitter.next(), Some("a"));
        assert_eq!(splitter.next(), Some("b"));
        assert_eq!(splitter.next(), Some("c"));
        assert_eq!(splitter.next(), Some("d"));
        assert_eq!(splitter.next(), Some("e"));
        assert_eq!(splitter.next(), None);
    }

    #[test]
    pub fn split_by_char() {
        let s = "hello world,this is rust";
        let mut iter = super::StrSplit {
            remainder: Some(s),
            delimiter: ' ',
        };
        assert_eq!(iter.next(), Some("hello"));
        assert_eq!(iter.next(), Some("world,this"));
        assert_eq!(iter.next(), Some("is"));
        assert_eq!(iter.next(), Some("rust"));
        assert_eq!(iter.next(), None);
    }

    #[test]
    pub fn test_unicode_delimiter() {
        let s = "I love rust🌍Rust is awesome🌍Let's code!❤";
        let mut iter = super::StrSplit {
            remainder: Some(s),
            delimiter: '🌍',
        };
        assert!('🌍'.len_utf8() == 4);
        assert!('❤'.len_utf8() == 3);
        assert_eq!(iter.next(), Some("I love rust"));
        assert_eq!(iter.next(), Some("Rust is awesome"));
        assert_eq!(iter.next(), Some("Let's code!❤"));
        assert_eq!(iter.next(), None);
    }

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
        let mut iter = v.into_iter().my_flatten();
        assert_eq!(iter.next(), Some(1));
        assert_eq!(iter.next_back(), Some(6));
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next(), Some(3));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), Some(5));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_my_map() {
        let v = vec![1, 2, 3];
        let mut iter = v.into_iter().my_map(|x| x * 2);
        assert_eq!(iter.next(), Some(2));
        assert_eq!(iter.next_back(), Some(6));
        assert_eq!(iter.next(), Some(4));
        assert_eq!(iter.next(), None);
    }
}
