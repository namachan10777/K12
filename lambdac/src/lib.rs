#[macro_use]
extern crate lalrpop_util;
pub mod alpha;
pub mod flatten;
pub mod parser;

#[derive(Debug, PartialEq, Clone)]
pub enum Sexp {
    List(Vec<Sexp>),
    Int(i32),
    Str(String),
    Bool(bool),
    Var(String),
    If(Box<Sexp>, Box<Sexp>, Box<Sexp>),
    Let(String, Box<Sexp>, Box<Sexp>),
    LetRec(String, Box<Sexp>, Box<Sexp>),
    Lambda(Vec<String>, Box<Sexp>),
    Call(String, Vec<Sexp>),
}
