mod jsp;
use jsp::JsonValue;
use std::collections::HashMap;
use std::str::FromStr;
use std::vec::Vec;
type PkChars<'a> = std::iter::Peekable<std::str::Chars<'a>>;

#[derive(Debug, PartialEq)]
pub enum Number {
    Int(i64),
    Float(f64),
}

pub fn consume_char(p: &mut PkChars, c: char) -> bool {
    if p.next_if_eq(&c).is_some() {
        true
    } else {
        false
    }
}

pub fn consume_anychar(p: &mut PkChars, chars: &str) -> Option<char> {
    for c in chars.chars() {
        if consume_char(p, c) {
            return Some(c);
        }
    }
    None
}

pub fn consume_prefix(p: &mut PkChars, t: &mut PkChars) -> String {
    let mut prefix = String::new();
    while let Some(&c) = t.peek() {
        if !consume_char(p, c) {
            return prefix;
        }
        prefix.push(c);
        t.next();
    }
    prefix
}

fn consume_key_pair(p: &mut PkChars) -> Option<(String, JsonValue)> {
    let p_clone = p.clone();
    jsp_consume_whitespace(p);
    if let Some(key) = jsp_consume_string(p) {
        jsp_consume_whitespace(p);
        if consume_char(p, ':') {
            jsp_consume_whitespace(p);
            if let Some(val) = jsp_consume_value(p) {
                return Some((key, val));
            }
        }
    }
    p.clone_from(&p_clone);
    None
}

fn consume_char_sequence(p: &mut PkChars, seq: &str) -> bool {
    let p_clone = p.clone();
    for c in seq.chars() {
        if !consume_char(p, c) {
            p.clone_from(&p_clone);
            return false;
        }
    }
    true
}

pub fn jsp_consume_array(p: &mut PkChars) -> Option<Vec<JsonValue>> {
    let p_clone = p.clone();
    let mut jsa = Vec::<JsonValue>::new();
    if !consume_char(p, '[') {
        p.clone_from(&p_clone);
        return None;
    }
    if let Some(v) = jsp_consume_value(p) {
        jsa.push(v);
        loop {
            if !consume_char(p, ',') {
                break;
            }
            if let Some(v) = jsp_consume_value(p) {
                jsa.push(v);
            } else {
                p.clone_from(&p_clone);
                return None;
            }
        }
    }
    if !consume_char(p, ']') {
        p.clone_from(&p_clone);
        return None;
    }
    Some(jsa)
}

pub fn jsp_parse_json(s: &str) -> Option<JsonValue> {
    let mut p = s.chars().peekable();
    let json = jsp_consume_value(&mut p);
    if p.peek().is_none() { json } else { None }
}

pub fn jsp_consume_value(p: &mut PkChars) -> Option<JsonValue> {
    let p_clone = p.clone();
    let mut jsonval = None;
    jsp_consume_whitespace(p);
    if let Some(&c) = p.peek() {
        match c {
            '"' => {
                if let Some(x) = jsp_consume_string(p) {
                    jsonval = Some(JsonValue::String(x));
                }
            }
            '{' => {
                if let Some(x) = jsp_consume_object(p) {
                    jsonval = Some(JsonValue::Object(x));
                }
            }
            '[' => {
                if let Some(x) = jsp_consume_array(p) {
                    jsonval = Some(JsonValue::Array(x));
                }
            }
            'n' => {
                if consume_char_sequence(p, "null") {
                    jsonval = Some(JsonValue::Null);
                }
            }
            't' => {
                if consume_char_sequence(p, "true") {
                    jsonval = Some(JsonValue::Bool(true));
                }
            }
            'f' => {
                if consume_char_sequence(p, "false") {
                    jsonval = Some(JsonValue::Bool(false));
                }
            }
            '-' | '0'..='9' => {
                jsonval = match jsp_consume_number(p) {
                    Some(Number::Int(x)) => Some(JsonValue::Int(x)),
                    Some(Number::Float(x)) => Some(JsonValue::Float(x)),
                    None => None,
                }
            }
            _ => (),
        }
    }
    if jsonval.is_none() {
        p.clone_from(&p_clone);
        None
    } else {
        jsp_consume_whitespace(p);
        jsonval
    }
}

pub fn jsp_consume_object(mut p: &mut PkChars) -> Option<HashMap<String, JsonValue>> {
    let p_clone = p.clone();
    let mut m = HashMap::<String, JsonValue>::new();
    if !consume_char(p, '{') {
        p.clone_from(&p_clone);
        return None;
    }

    if let Some((key, val)) = consume_key_pair(p) {
        m.insert(key, val);
        loop {
            jsp_consume_whitespace(&mut p);
            if !consume_char(p, ',') {
                break;
            }
            if let Some((key, val)) = consume_key_pair(p) {
                m.insert(key, val);
            } else {
                p.clone_from(&p_clone);
                return None;
            }
        }
    }

    jsp_consume_whitespace(&mut p);

    if consume_char(p, '}') {
        Some(m)
    } else {
        p.clone_from(&p_clone);
        None
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

pub fn jsp_consume_all_digits(p: &mut PkChars, n: &mut String) -> usize {
    let mut count = 0;
    while let Some(&c) = p.peek() {
        if c.is_ascii_digit() {
            p.next();
            n.push(c);
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
    let mut nstr = String::new();
    let mut frac = false;
    let mut exp = false;

    if consume_char(p, '-') {
        // Consume a minus, if exists.
        nstr.push('-');
    }

    if consume_char(p, '0') {
        nstr.push('0');
        let c = p.peek();
        if c.is_none() {
            return Some(Number::Int(0));
        }
        // Leading zero is not accepted.
        if c.unwrap().is_ascii_digit() {
            p.clone_from(&p_clone);
            return None;
        }
    } else {
        let n: usize = jsp_consume_all_digits(p, &mut nstr);
        if n == 0 {
            p.clone_from(&p_clone);
            return None;
        }
    }
    if consume_char(p, '.') {
        nstr.push('.');
        let n: usize = jsp_consume_all_digits(p, &mut nstr);
        if n == 0 {
            p.clone_from(&p_clone);
            return None;
        }
        frac = true;
    }
    if let Some(e) = consume_anychar(p, "eE") {
        nstr.push(e);
        if let Some(sign) = consume_anychar(p, "-+") {
            nstr.push(sign);
        }
        let n: usize = jsp_consume_all_digits(p, &mut nstr);
        if n == 0 {
            p.clone_from(&p_clone);
            return None;
        }
        exp = true;
    }

    if exp || frac {
        if let Ok(f) = f64::from_str(&nstr) {
            return Some(Number::Float(f));
        }
    } else {
        if let Ok(i) = i64::from_str(&nstr) {
            return Some(Number::Int(i));
        }
    }
    None
}

fn jsp_consume_low_surrogate(p: &mut PkChars) -> Option<u32> {
    let p_clone = p.clone();
    if consume_char_sequence(p, "\\u") {
        if let Some(val) = jsp_consume_four_hexdigits(p) {
            if val >= 0xDC00 && val <= 0xDFFF {
                return Some(val);
            }
        }
    }
    p.clone_from(&p_clone);
    None
}

/** Consume a JSON string */
pub fn jsp_consume_string(p: &mut PkChars) -> Option<String> {
    let p_clone = p.clone();
    let mut s = String::new();
    if !consume_char(p, '"') {
        p.clone_from(&p_clone);
        return None;
    }
    let mut escaped = false;
    loop {
        if let Some(&c) = p.peek() {
            if c.is_ascii_control() && c != '\x7F' {
                p.clone_from(&p_clone);
                return None;
            }
            if escaped {
                match c {
                    '"' | '\\' | '/' => {
                        s.push(c);
                        p.next();
                    }
                    'b' => {
                        s.push('\x08');
                        p.next();
                    }
                    'f' => {
                        s.push('\x0C');
                        p.next();
                    }
                    'n' => {
                        s.push('\n');
                        p.next();
                    }
                    'r' => {
                        s.push('\r');
                        p.next();
                    }
                    't' => {
                        s.push('\t');
                        p.next();
                    }
                    'u' => {
                        p.next();
                        if let Some(val) = jsp_consume_four_hexdigits(p) {
                            if val >= 0xDC00 && val <= 0xDFFF {
                                // Error: Low surrogate without a high surrogate
                                p.clone_from(&p_clone);
                                return None;
                            }
                            if val >= 0xD800 && val <= 0xDBFF {
                                // val is high surrogate.
                                if let Some(lval) = jsp_consume_low_surrogate(p) {
                                    let v: [u16; 2] =
                                        [val.try_into().unwrap(), lval.try_into().unwrap()];
                                    for x in char::decode_utf16(v) {
                                        match x {
                                            Ok(c) => {
                                                s.push(c);
                                            }
                                            _ => {
                                                p.clone_from(&p_clone);
                                                return None;
                                            }
                                        }
                                    }
                                } else {
                                    // Error: Unpaired surrogate
                                    p.clone_from(&p_clone);
                                    return None;
                                }
                            } else {
                                s.push(char::from_u32(val).unwrap());
                            }
                        } else {
                            p.clone_from(&p_clone);
                            return None;
                        }
                    }
                    /* Invalid escape sequence */
                    _ => {
                        p.clone_from(&p_clone);
                        return None;
                    }
                }
                escaped = false;
                continue;
            }
            match c {
                '"' => {
                    p.next();
                    return Some(s);
                }
                '\\' => {
                    escaped = true;
                }
                _ => {
                    s.push(c);
                }
            }
            p.next();
        } else {
            p.clone_from(&p_clone);
            return None;
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
            assert!(consume_char(&mut pk, c));
        }
        let mut pk = name.chars().peekable();
        for c in "test".chars() {
            assert!(!consume_char(&mut pk, c));
        }
    }

    #[test]
    fn test1_consume_prefix() {
        let name1 = "Balaji";
        let name2 = "Lalaji";

        let mut p1 = name1.chars().peekable();
        let mut p2 = name2.chars().peekable();
        let res = consume_prefix(&mut p1, &mut p2);
        assert!(res == "");
        let res = consume_prefix(&mut p1, &mut p2);
        assert!(res == "");
        assert!(p1.peek() == Some(&'B'));
        assert!(p2.peek() == Some(&'L'));
    }
    #[test]
    fn test2_consume_prefix() {
        let mut p1 = "Balaji".chars().peekable();
        let mut p2 = "Bal".chars().peekable();
        let res = consume_prefix(&mut p1, &mut p2);
        assert!(res == "Bal");
        assert!(p1.peek() == Some(&'a'));
        assert!(p2.peek().is_none());
    }

    #[test]
    fn test3_consume_prefix() {
        let name = "Pragna";
        let mut p1 = name.chars().peekable();
        let mut p2 = name.chars().peekable();
        let res = consume_prefix(&mut p1, &mut p2);
        assert!(res == "Pragna");
        assert!(p1.peek().is_none());
        assert!(p2.peek().is_none());
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
            assert!(jsp_consume_string(&mut s.chars().peekable()).is_some());
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
            assert!(z.is_some());
        }
        for t in y {
            println!("test: {t}");
            let z = jsp_consume_number(&mut t.chars().peekable());
            assert!(z.is_none());
        }
        let x = "+123";
        assert!(jsp_consume_number(&mut x.chars().peekable()).is_none());
        let x = "0.1e-2";
        assert_eq!(
            jsp_consume_number(&mut x.chars().peekable()).unwrap(),
            Number::Float(f64::from_str(x).unwrap())
        );
    }

    #[test]
    fn test_jsp_consume_value() {
        let x = "null";
        assert_eq!(jsp_parse_json(x).unwrap().to_string(), "null");
        let x = "true";
        assert_eq!(jsp_parse_json(x).unwrap().to_string(), "true");
        let x = "false";
        assert_eq!(jsp_parse_json(x).unwrap().to_string(), "false");
        let x = "22";
        assert_eq!(jsp_parse_json(x).unwrap().to_string(), "22");
        let x = "0.2E+03";
        assert_eq!(
            jsp_parse_json(x).unwrap().to_string(),
            f64::from_str(x).unwrap().to_string()
        );
        let x = r#"["Balaji",[],{}]"#;
        let y = vec![
            JsonValue::String("Balaji".to_string()),
            JsonValue::Array(vec![]),
            JsonValue::Object(HashMap::new()),
        ];
        let x = jsp_parse_json(x).unwrap();
        println!("Array = {}", x);
        assert_eq!(x, JsonValue::Array(y));
        let x = r#"{"Name" : "Balaji", "Age" : 40}"#;
        let y = HashMap::from([
            ("Name".to_string(), JsonValue::String("Balaji".to_string())),
            ("Age".to_string(), JsonValue::Int(40)),
        ]);
        let x = jsp_parse_json(x).unwrap();
        println!("Object = {}", x);
        assert_eq!(x, JsonValue::Object(y));
    }
}
