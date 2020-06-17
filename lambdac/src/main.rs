#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub grammar);

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn parser() {
        assert_eq!(grammar::TermParser::new().parse("123"), Ok(123));
        assert_eq!(grammar::TermParser::new().parse("(123)"), Ok(123));
    }
}
