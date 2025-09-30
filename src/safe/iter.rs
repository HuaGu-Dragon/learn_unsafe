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
    pub fn spilt_test() {
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
}
