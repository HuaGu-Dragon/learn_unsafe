pub fn str_tok<'s>(s: &mut &'s str, delim: char) -> Option<&'s str> {
    if let Some(index) = s.find(delim) {
        let token = &s[..index];
        *s = &s[index + delim.len_utf8()..];
        Some(token)
    } else {
        let token = &s[..];
        *s = "";
        Some(token)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_str_tok() {
        let s = "hello,world,test".to_string();
        assert_eq!(str_tok(&mut &*s, ','), Some("hello"));
        assert_eq!(str_tok(&mut &*s, ','), Some("world"));
        assert_eq!(str_tok(&mut &*s, ','), Some("test"));
        assert_eq!(str_tok(&mut &*s, ','), None);
        assert_eq!(s, "");
    }
}
