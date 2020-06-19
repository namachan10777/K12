#[macro_use]
extern crate lalrpop_util;
lalrpop_mod!(pub grammar);

#[derive(Debug, PartialEq)]
pub enum Expr {
    Int(i32),
    Bool(bool),
    Str(String),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Mod(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
    Not(Box<Expr>),
    Call(String, Vec<Expr>),
}

#[derive(Debug)]
pub enum Stmt {
    Nop,
    If(Expr, Vec<Stmt>, Vec<Stmt>),
    Loop(Vec<Stmt>),
    Let(String, Box<Expr>),
    Break,
    Continue,
    Return(Vec<Expr>),
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn parser() {
        assert_eq!(grammar::ExprParser::new().parse("123"), Ok(Expr::Int(123)));
        assert_eq!(
            grammar::ExprParser::new().parse("pow(11+15*4, 2)"),
            Ok(Expr::Call(
                "pow".to_owned(),
                vec![
                    Expr::Add(
                        Box::new(Expr::Int(11)),
                        Box::new(Expr::Mul(Box::new(Expr::Int(15)), Box::new(Expr::Int(4))))
                    ),
                    Expr::Int(2)
                ]
            ))
        );
    }
}
