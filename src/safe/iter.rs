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

#[cfg(test)]
mod tests {
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
}
