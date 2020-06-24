use super::Sexp;
use std::collections::HashMap;
pub type Env = HashMap<String, usize>;

pub fn f(env: &Env, sexp: &Sexp) -> Sexp {
    match sexp {
        Sexp::If(cond, expr_then, expr_else) => Sexp::If(
            Box::new(f(env, &cond)),
            Box::new(f(env, &expr_then)),
            Box::new(f(env, &expr_else)),
        ),
        Sexp::Str(s) => Sexp::Str(s.to_owned()),
        Sexp::Int(i) => Sexp::Int(*i),
        Sexp::Bool(b) => Sexp::Bool(*b),
        Sexp::Var(id) => Sexp::Var(format!("{}({})", id, env.get(id).unwrap())),
        Sexp::Let(id, def, expr) => {
            let mut nenv = env.clone();
            nenv.insert(id.to_owned(), env.get(id).cloned().unwrap_or(0)+1);
            Sexp::Let(
                format!("{}({})", id, nenv.get(id).unwrap()),
                Box::new(f(env, def)),
                Box::new(f(&nenv, expr)),
            )
        }
        Sexp::Lambda(args, expr) => {
            let mut nenv = env.clone();
            let args = args
                .iter()
                .map(|id| {
                    let n = env.get(id).cloned().unwrap_or(0)+1;
                    nenv.insert(id.to_owned(), n);
                    format!("{}({})", id, n)
                })
                .collect::<Vec<_>>();
            Sexp::Lambda(args, Box::new(f(&nenv, expr)))
        }
        Sexp::Call(id, args) => Sexp::Call(
            id.to_owned(),
            args.iter().map(|arg| f(env, arg)).collect::<Vec<_>>(),
        ),
        _ => unimplemented!(),
    }
}
