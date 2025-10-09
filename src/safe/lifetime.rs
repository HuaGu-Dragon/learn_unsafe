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

///```compile_fail
///
/// fn make_static(_s: &'static str) {}
///
/// let mut static_str: &'static str = "static";
/// make_static(static_str); // In order to avoid inheriting 'static lifetime
/// let mut s = String::from("local");
/// let s_ref = &mut static_str;
/// *s_ref = &mut s;
///
/// ```
fn _foo() {}

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
