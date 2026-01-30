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

#[derive(Debug, PartialEq)]
pub enum JspError {
    Empty,
    Eof,
    HasTail,
    InvalidArray,
    InvalidBool,
    InvalidEscapeSequence,
    InvalidNull,
    InvalidNumber,
    InvalidObject,
    InvalidString,
    InvalidValue,
    MissingArrayEnd,
    MissingArrayStart,
    MissingColon,
    MissingObjectEnd,
    MissingObjectStart,
    NoPair,
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

fn consume_key_pair(p: &mut PkChars) -> Result<(String, JsonValue), JspError> {
    jsp_consume_whitespace(p);
    if p.peek().is_none() {
        return Err(JspError::Eof);
    }
    if Some(&'}') == p.peek() {
        return Err(JspError::NoPair);
    }
    let key = jsp_consume_string(p)?;
    jsp_consume_whitespace(p);
    if consume_char(p, ':') {
        jsp_consume_whitespace(p);
        let val = jsp_consume_value(p)?;
        Ok((key, val))
    } else {
        Err(JspError::MissingColon)
    }
}

fn consume_char_sequence(p: &mut PkChars, seq: &str) -> bool {
    for c in seq.chars() {
        if !consume_char(p, c) {
            return false;
        }
    }
    true
}

pub fn jsp_consume_array(p: &mut PkChars) -> Result<Vec<JsonValue>, JspError> {
    let mut jsa = Vec::<JsonValue>::new();
    if !consume_char(p, '[') {
        return Err(JspError::MissingArrayStart);
    }
    let val = jsp_consume_value(p);
    if val == Err(JspError::Empty) {
        // See if it's an empty array.
        if consume_char(p, ']') {
            return Ok(jsa);
        } else {
            return Err(JspError::MissingArrayEnd);
        }
    }
    let v = val?;
    jsa.push(v);
    loop {
        if !consume_char(p, ',') {
            break;
        }
        let v = jsp_consume_value(p)?;
        jsa.push(v);
    }
    if consume_char(p, ']') {
        Ok(jsa)
    } else {
        Err(JspError::MissingArrayEnd)
    }
}

pub fn jsp_parse_json(s: &str) -> Result<JsonValue, JspError> {
    let mut p = s.chars().peekable();
    let json = jsp_consume_value(&mut p)?;
    if p.peek().is_none() {
        Ok(json)
    } else {
        Err(JspError::HasTail)
    }
}

pub fn jsp_consume_value(p: &mut PkChars) -> Result<JsonValue, JspError> {
    let mut jsonval = Err(JspError::Empty);
    jsp_consume_whitespace(p);
    if let Some(&c) = p.peek() {
        jsonval = match c {
            '"' => {
                let x = jsp_consume_string(p)?;
                Ok(JsonValue::String(x))
            }
            '{' => {
                let x = jsp_consume_object(p)?;
                Ok(JsonValue::Object(x))
            }
            '[' => {
                let x = jsp_consume_array(p)?;
                Ok(JsonValue::Array(x))
            }
            'n' => {
                if consume_char_sequence(p, "null") {
                    Ok(JsonValue::Null)
                } else {
                    Err(JspError::InvalidNull)
                }
            }
            't' => {
                if consume_char_sequence(p, "true") {
                    Ok(JsonValue::Bool(true))
                } else {
                    Err(JspError::InvalidBool)
                }
            }
            'f' => {
                if consume_char_sequence(p, "false") {
                    Ok(JsonValue::Bool(false))
                } else {
                    Err(JspError::InvalidBool)
                }
            }
            '-' | '0'..='9' => match jsp_consume_number(p) {
                Ok(Number::Int(x)) => Ok(JsonValue::Int(x)),
                Ok(Number::Float(x)) => Ok(JsonValue::Float(x)),
                Err(e) => Err(e),
            },
            _ => jsonval,
        }
    }
    if jsonval.is_ok() {
        jsp_consume_whitespace(p);
    }
    jsonval
}

pub fn jsp_consume_object(mut p: &mut PkChars) -> Result<HashMap<String, JsonValue>, JspError> {
    let mut m = HashMap::<String, JsonValue>::new();
    if !consume_char(p, '{') {
        return Err(JspError::MissingObjectStart);
    }

    match consume_key_pair(p) {
        Ok((key, val)) => {
            m.insert(key, val);
            loop {
                jsp_consume_whitespace(&mut p);
                if !consume_char(p, ',') {
                    break;
                }
                match consume_key_pair(p) {
                    Ok((key, val)) => {
                        m.insert(key, val);
                    }
                    Err(x) => return Err(x),
                }
            }
        }
        Err(JspError::NoPair) => {
            if !consume_char(p, '}') {
                panic!("Error: Parsing Bug");
            }
            return Ok(m);
        }
        Err(e) => return Err(e),
    };

    jsp_consume_whitespace(&mut p);

    if !consume_char(p, '}') {
        Err(JspError::MissingObjectEnd)
    } else {
        Ok(m)
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

pub fn jsp_consume_four_hexdigits(p: &mut PkChars) -> Option<u16> {
    let mut val = 0_u16;
    for _i in 0..4 {
        if let Some(v) = jsp_consume_hexdigit(p) {
            val *= 16_u16;
            val += v as u16;
        } else {
            return None;
        }
    }
    Some(val)
}

pub fn jsp_consume_number(p: &mut PkChars) -> Result<Number, JspError> {
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
            return Ok(Number::Int(0));
        }
        // Leading zero is not accepted.
        if c.unwrap().is_ascii_digit() {
            return Err(JspError::InvalidNumber);
        }
    } else {
        let n: usize = jsp_consume_all_digits(p, &mut nstr);
        if n == 0 {
            return Err(JspError::InvalidNumber);
        }
    }
    if consume_char(p, '.') {
        nstr.push('.');
        let n: usize = jsp_consume_all_digits(p, &mut nstr);
        if n == 0 {
            return Err(JspError::InvalidNumber);
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
            return Err(JspError::InvalidNumber);
        }
        exp = true;
    }

    if exp || frac {
        if let Ok(f) = f64::from_str(&nstr) {
            return Ok(Number::Float(f));
        }
    } else {
        if let Ok(i) = i64::from_str(&nstr) {
            return Ok(Number::Int(i));
        }
    }
    Err(JspError::InvalidNumber)
}

fn jsp_consume_low_surrogate(p: &mut PkChars) -> Option<u16> {
    if consume_char_sequence(p, "\\u") {
        let val = jsp_consume_four_hexdigits(p)?;
        if val >= 0xDC00 && val <= 0xDFFF {
            return Some(val);
        }
    }
    None
}

fn read_escaped_char(p: &mut PkChars) -> Result<char, JspError> {
    if !consume_char(p, '\\') {
        return Err(JspError::InvalidEscapeSequence);
    }
    let &c = p.peek().ok_or(JspError::Eof)?;
    match c {
        '"' | '\\' | '/' => {
            p.next();
            Ok(c)
        }
        'b' => {
            p.next();
            Ok('\x08')
        }
        'f' => {
            p.next();
            Ok('\x0C')
        }
        'n' => {
            p.next();
            Ok('\n')
        }
        'r' => {
            p.next();
            Ok('\r')
        }
        't' => {
            p.next();
            Ok('\t')
        }
        'u' => {
            p.next();
            let val = jsp_consume_four_hexdigits(p).ok_or(JspError::InvalidEscapeSequence)?;
            match val {
                0xDC00..=0xDFFF => {
                    // Error: Low surrogate without a high surrogate
                    return Err(JspError::InvalidEscapeSequence);
                }
                0xD800..=0xDBFF => {
                    // val is high surrogate.
                    let lval =
                        jsp_consume_low_surrogate(p).ok_or(JspError::InvalidEscapeSequence)?;
                    match char::decode_utf16([val, lval]).next().unwrap() {
                        Ok(c) => return Ok(c),
                        Err(e) => {
                            println!("Error: {e}");
                            Err(JspError::InvalidString)
                        }
                    }
                }
                _ => Ok(char::from_u32(val as u32).unwrap()),
            }
        }
        /* Invalid escape sequence */
        _ => Err(JspError::InvalidString),
    }
}

/** Consume a JSON string */
pub fn jsp_consume_string(p: &mut PkChars) -> Result<String, JspError> {
    let mut s = String::new();
    if !consume_char(p, '"') {
        return Err(JspError::InvalidString);
    }
    loop {
        if let Some(&c) = p.peek() {
            if c.is_ascii_control() && c != '\x7F' {
                return Err(JspError::InvalidString);
            }
            match c {
                '"' => {
                    p.next();
                    return Ok(s);
                }
                '\\' => {
                    s.push(read_escaped_char(p)?);
                }
                _ => {
                    s.push(c);
                    p.next();
                }
            }
        } else {
            return Err(JspError::InvalidString);
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
            println!("test string: {s}");
            assert!(jsp_consume_string(&mut s.chars().peekable()).is_ok());
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
            assert!(z.is_ok());
        }
        for t in y {
            println!("test: {t}");
            let z = jsp_consume_number(&mut t.chars().peekable());
            assert!(z.is_err());
        }
        let x = "+123";
        assert!(jsp_consume_number(&mut x.chars().peekable()).is_err());
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
        println!("test input: {}", x);
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
        println!("test input: {}", x);
        let x = jsp_parse_json(x).unwrap();
        println!("Array = {}", x);
        assert_eq!(x, JsonValue::Array(y));
        let x = r#"{"Name" : "Balaji", "Age" : 40}"#;
        let y = HashMap::from([
            ("Name".to_string(), JsonValue::String("Balaji".to_string())),
            ("Age".to_string(), JsonValue::Int(40)),
        ]);
        println!("test input: {}", x);
        let x = jsp_parse_json(x).unwrap();
        println!("Object = {}", x);
        assert_eq!(x, JsonValue::Object(y));
    }
}
