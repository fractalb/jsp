type PkChars<'a> = std::iter::Peekable<std::str::Chars<'a>>;

#[derive(Debug, PartialEq)]
pub enum Number {
    Int(i64),
    Float(f64),
}

pub fn consume_char(p: &mut PkChars, c: char) -> Option<char> {
    p.next_if_eq(&c)
}

pub fn consume_anychar(p: &mut PkChars, chars: &str) -> Option<char> {
    for c in chars.chars() {
        if consume_char(p, c) != None {
            return Some(c);
        }
    }
    None
}

pub fn consume_prefix<'a>(p: &mut PkChars<'a>, t: &mut PkChars<'a>) -> usize {
    let mut count: usize = 0;
    while let Some(&c) = t.peek() {
        if consume_char(p, c) == None {
            return count;
        }
        count += 1;
        t.next();
    }
    count
}

fn consume_key_pair(p: &mut PkChars) -> bool {
    let p_clone = p.clone();
    jsp_consume_whitespace(p);
    if jsp_consume_string(p) {
        jsp_consume_whitespace(p);
        if consume_char(p, ':') != None {
            jsp_consume_whitespace(p);
            if jsp_consume_value(p) {
                return true;
            }
        }
    }
    p.clone_from(&p_clone);
    false
}

fn consume_char_sequence(p: &mut PkChars, seq: &str) -> bool {
    let p_clone = p.clone();
    for c in seq.chars() {
        if consume_char(p, c) == None {
            p.clone_from(&p_clone);
            return false;
        }
    }
    true
}

pub fn jsp_consume_array(p: &mut PkChars) -> bool {
    let p_clone = p.clone();
    if consume_char(p, '[') == None {
        p.clone_from(&p_clone);
        return false;
    }
    if jsp_consume_value(p) {
        loop {
            if consume_char(p, ',') == None {
                break;
            }
            if !jsp_consume_value(p) {
                p.clone_from(&p_clone);
                return false;
            }
        }
    }
    if consume_char(p, ']') == None {
        p.clone_from(&p_clone);
        return false;
    }
    true
}

pub fn jsp_consume_value(p: &mut PkChars) -> bool {
    let p_clone = p.clone();
    jsp_consume_whitespace(p);
    if let Some(&c) = p.peek() {
        if !match c {
            '"' => jsp_consume_string(p),
            '{' => jsp_consume_object(p),
            '[' => jsp_consume_array(p),
            'n' => consume_char_sequence(p, "null"),
            't' => consume_char_sequence(p, "true"),
            'f' => consume_char_sequence(p, "false"),
            '-' | '0'..='9' => jsp_consume_number(p) != None,
            _ => false,
        } {
            p.clone_from(&p_clone);
            return false;
        }
    } else {
        p.clone_from(&p_clone);
        return false;
    }
    jsp_consume_whitespace(p);
    true
}

pub fn jsp_consume_object(mut p: &mut PkChars) -> bool {
    let p_clone = p.clone();
    if consume_char(p, '{') == None {
        p.clone_from(&p_clone);
        return false;
    }

    if consume_key_pair(p) {
        loop {
            jsp_consume_whitespace(&mut p);
            if consume_char(p, ',') == None {
                break;
            }
            if !consume_key_pair(p) {
                p.clone_from(&p_clone);
                return false;
            }
        }
    }

    jsp_consume_whitespace(&mut p);

    if consume_char(p, '}') != None {
        true
    } else {
        p.clone_from(&p_clone);
        false
    }
}

pub fn jsp_consume_digit(p: &mut PkChars) -> Option<u32> {
    let c = p.peek()?;
    if let Some(c) = c.to_digit(10) {
        p.next();
        return Some(c);
    }
    None
}

pub fn jsp_consume_all_digits(p: &mut PkChars) -> usize {
    let mut count = 0;
    while let Some(&c) = p.peek() {
        if c.is_ascii_digit() {
            p.next();
            count += 1;
        } else {
            break;
        }
    }
    return count;
}

pub fn jsp_consume_hexdigit(p: &mut PkChars) -> Option<u32> {
    let c = p.peek()?;
    if let Some(c) = c.to_digit(16) {
        p.next();
        return Some(c);
    }
    None
}

pub fn jsp_consume_four_hexdigits(p: &mut PkChars) -> Option<u32> {
    let p_clone = p.clone();
    let mut val = 0;
    for _i in 0..4 {
        if let Some(v) = jsp_consume_hexdigit(p) {
            val *= 16;
            val += v;
        } else {
            p.clone_from(&p_clone);
            return None;
        }
    }
    Some(val)
}

pub fn jsp_consume_number(p: &mut PkChars) -> Option<Number> {
    let p_clone = p.clone();
    let mut frac = false;
    let mut exp = false;

    consume_char(p, '-'); // Consume a minus, if exists.

    if consume_char(p, '0') == None {
        let n: usize = jsp_consume_all_digits(p);
        if n == 0 {
            p.clone_from(&p_clone);
            return None;
        }
    } else {
        let c = p.peek();
        if c == None {
            return Some(Number::Int(0));
        }
        if c.unwrap().is_ascii_digit() {
            p.clone_from(&p_clone);
            return None;
        }
    }
    if consume_char(p, '.') != None {
        let n: usize = jsp_consume_all_digits(p);
        if n == 0 {
            p.clone_from(&p_clone);
            return None;
        }
        frac = true;
    }
    if consume_anychar(p, "eE") != None {
        consume_anychar(p, "-+");
        let n: usize = jsp_consume_all_digits(p);
        if n == 0 {
            p.clone_from(&p_clone);
            return None;
        }
        exp = true;
    }

    if exp || frac {
        Some(Number::Float(0.0))
    } else {
        Some(Number::Int(0))
    }
}

/** Consume a JSON string */
pub fn jsp_consume_string(p: &mut PkChars) -> bool {
    let p_clone = p.clone();
    if consume_char(p, '"') == None {
        p.clone_from(&p_clone);
        return false;
    }
    let mut escaped = false;
    loop {
        if let Some(&c) = p.peek() {
            if c.is_ascii_control() && c != '\x7F' {
                p.clone_from(&p_clone);
                return false;
            }
            if escaped {
                match c {
                    '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' => {
                        p.next();
                    }
                    'u' => {
                        p.next();
                        if jsp_consume_four_hexdigits(p) == None {
                            p.clone_from(&p_clone);
                            return false;
                        }
                    }
                    /* Invalid escape sequence */
                    _ => {
                        p.clone_from(&p_clone);
                        return false;
                    }
                }
                escaped = false;
                continue;
            }
            match c {
                '"' => {
                    p.next();
                    return true;
                }
                '\\' => {
                    escaped = true;
                }
                _ => {}
            }
            p.next();
        } else {
            p.clone_from(&p_clone);
            return false;
        }
    }
}

pub fn jsp_consume_whitespace(p: &mut PkChars) -> usize {
    let mut count: usize = 0;
    loop {
        match p.peek() {
            Some(&' ') => p.next(),
            Some(&'\t') => p.next(),
            Some(&'\n') => p.next(),
            Some(&'\r') => p.next(),
            _ => break,
        };
        count += 1;
    }
    return count;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consume_char() {
        let name = "Balaji";
        let mut pk = name.chars().peekable();
        for c in name.chars() {
            assert_eq!(consume_char(&mut pk, c), Some(c));
        }
        let mut pk = name.chars().peekable();
        for c in "test".chars() {
            assert_eq!(consume_char(&mut pk, c), None);
        }
    }

    #[test]
    fn test1_consume_prefix() {
        let name1 = "Balaji";
        let name2 = "Babai";

        let mut p1 = name1.chars().peekable();
        let mut p2 = name2.chars().peekable();
        for _i in 1..2 {
            let res = consume_prefix(&mut p1, &mut p2);
            assert!(res == 2);
        }
        assert!(p1.peek() == Some(&'l'));
        assert!(p2.peek() == Some(&'b'));
    }
    #[test]
    fn test2_consume_prefix() {
        let mut p1 = "Balaji".chars().peekable();
        let mut p2 = "Bal".chars().peekable();
        let res = consume_prefix(&mut p1, &mut p2);
        assert!(res == 3);
        assert!(p1.peek() == Some(&'a'));
        assert!(p2.peek() == None);
    }

    #[test]
    fn test3_consume_prefix() {
        let name = "Pragna";
        let mut p1 = name.chars().peekable();
        let mut p2 = name.chars().peekable();
        let res = consume_prefix(&mut p1, &mut p2);
        assert!(res == 6);
        assert!(p1.peek() == None);
        assert!(p2.peek() == None);
    }
    #[test]
    fn test_jsp_consume_string() {
        let test_strs = vec![
            r#""""#,
            r#"":"", r#""\"""#,
            r#""\\""#,
            r#""\b\f\n\r\t ""#,
            r#""\uABCD""#,
        ];
        for s in test_strs {
            assert!(jsp_consume_string(&mut s.chars().peekable()));
        }
    }
    #[test]
    fn test_jsp_consume_digit() {
        let n = vec![0, 1, 2, 3];
        let mut p = "0123".chars().peekable();

        for i in n {
            assert!(Some(i) == jsp_consume_digit(&mut p));
        }
    }
    #[test]
    fn test_jsp_consume_number() {
        let x = vec!["0", "-0", "0.0", "-0.1", "1.0", "3.01e+2", "10E-2"];
        let y = vec![
            "00", "01", "--0", ".1", "-1.", "+3.01e+2", "0.0E--2", "0.6E++2", "0.1e+-2",
        ];
        for t in x {
            println!("test: {t}");
            let z = jsp_consume_number(&mut t.chars().peekable());
            assert!(z != None);
        }
        for t in y {
            println!("test: {t}");
            let z = jsp_consume_number(&mut t.chars().peekable());
            assert!(z == None);
        }
    }
}
