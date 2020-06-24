use super::Sexp;
lalrpop_mod!(pub grammar);

pub fn f(s: &str) -> Sexp {
    grammar::SexpParser::new().parse(s).unwrap()
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn parse() {
        assert_eq!(grammar::SexpParser::new().parse("1"), Ok(Sexp::Int(1)));
        assert_eq!(
            grammar::SexpParser::new().parse("'(1 2 3)"),
            Ok(Sexp::List(vec![Sexp::Int(1), Sexp::Int(2), Sexp::Int(3)]))
        );
        assert_eq!(
            grammar::SexpParser::new().parse("\"hello\""),
            Ok(Sexp::Str("hello".to_owned()))
        );
        assert_eq!(
            grammar::SexpParser::new().parse("false"),
            Ok(Sexp::Bool(false))
        );
        assert_eq!(
            grammar::SexpParser::new().parse("hoge"),
            Ok(Sexp::Var("hoge".to_owned()))
        );
        assert_eq!(
            grammar::SexpParser::new().parse("(if false 1 2)"),
            Ok(Sexp::If(
                Box::new(Sexp::Bool(false)),
                Box::new(Sexp::Int(1)),
                Box::new(Sexp::Int(2))
            ))
        );
        assert_eq!(
            grammar::SexpParser::new().parse("(add 1 2)"),
            Ok(Sexp::Call(
                "add".to_owned(),
                vec![Sexp::Int(1), Sexp::Int(2)]
            ))
        );
        assert_eq!(
            grammar::SexpParser::new().parse("(lambda (x y) (add x y))"),
            Ok(Sexp::Lambda(
                vec!["x".to_owned(), "y".to_owned()],
                Box::new(Sexp::Call(
                    "add".to_owned(),
                    vec![Sexp::Var("x".to_owned()), Sexp::Var("y".to_owned())]
                ))
            ))
        );
        assert_eq!(
            grammar::SexpParser::new().parse("(let x 2 (add x 2))"),
            Ok(Sexp::Let(
                "x".to_owned(),
                Box::new(Sexp::Int(2)),
                Box::new(Sexp::Call(
                    "add".to_owned(),
                    vec![Sexp::Var("x".to_owned()), Sexp::Int(2)]
                ))
            ))
        );
    }
}
