type  PkChars<'a> = std::iter::Peekable<std::str::Chars<'a>>;

pub fn consume(p: &mut PkChars, c: char) -> Option<char> {
    p.next_if_eq(&c)
}

pub fn consume_prefix<'a>(p: &'a mut PkChars<'a>, t: &'a mut PkChars<'a>) -> (usize, &'a mut PkChars<'a>, &'a mut PkChars<'a>) {
    let mut count : usize = 0;
    while let Some(&c) = t.peek() {
        if consume(p, c) == None {
            return (count, p, t);
        }
        count += 1;
        t.next();
    }
    (count, p, t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consume() {
        let name = "Balaji";
        let mut pk = name.chars().peekable();
        for c in name.chars() {
            assert_eq!(consume(&mut pk, c), Some(c));
        }
        let mut pk = name.chars().peekable();
        for c in "test".chars() {
            assert_eq!(consume(&mut pk, c), None);
        }
    }

    #[test]
    fn test1_consume_prefix() {
        let name1 = "Balaji";
        let name2 = "Babai";

        let mut p1 = name1.chars().peekable();
        let mut p2 = name2.chars().peekable();
        let res = consume_prefix(&mut p1, &mut p2);
        assert!(res.0 == 2);
        assert!(res.1.peek() == Some(&'l'));
        assert!(res.2.peek() == Some(&'b'));
    }
    #[test]
    fn test2_consume_prefix() {
        let mut p1 = "Balaji".chars().peekable();
        let mut p2 = "Bal".chars().peekable();
        let res = consume_prefix(&mut p1, &mut p2);
        assert!(res.0 == 3);
        assert!(res.1.peek() == Some(&'a'));
        assert!(res.2.peek() == None);
    }

    #[test]
    fn test3_consume_prefix() {
        let name = "Pragna";
        let mut p1 = name.chars().peekable();
        let mut p2 = name.chars().peekable();
        let res = consume_prefix(&mut p1, &mut p2);
        assert!(res.0 == 6);
        assert!(res.1.peek() == None);
        assert!(res.2.peek() == None);
    }
}
