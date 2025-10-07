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
        let token = &mut s.as_str();
        assert_eq!(str_tok(&mut , ','), Some("hello"));
        assert_eq!(str_tok(&mut , ','), Some("world"));
        assert_eq!(str_tok(&mut , ','), Some("test"));
        assert_eq!(str_tok(&mut , ','), None);
        assert_eq!(s, "");
    }
}
