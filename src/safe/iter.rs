pub struct StrSplit<'haystack, 'delimiter> {
    remainder: Option<&'haystack str>,
    delimiter: &'delimiter str,
}

impl<'haystack> Iterator for StrSplit<'haystack, '_> {
    type Item = &'haystack str;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(remainder) = self.remainder {
            let Some(index) = remainder.find(self.delimiter) else {
                return self.remainder.take();
            };
            self.remainder = Some(&remainder[index + self.delimiter.len()..]);
            Some(&remainder[..index])
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
}
