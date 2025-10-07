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
        let mut token = s.as_str();
        let mut ret = str_tok(&mut token, ',').unwrap();
        assert_eq!(ret, "hello");
        assert_eq!(token, "world,test");
        ret = str_tok(&mut token, ',').unwrap();
        assert_eq!(ret, "world");
        assert_eq!(token, "test");
        ret = str_tok(&mut token, ',').unwrap();
        assert_eq!(ret, "test");
        assert_eq!(token, "");
    }
}
