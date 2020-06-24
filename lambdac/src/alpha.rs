use super::Sexp;
use std::collections::HashMap;
pub type Env = HashMap<String, usize>;
pub type Map = HashMap<String, String>;

pub fn f(sexp: Sexp) -> Sexp {
    let mut env = Env::new();
    let map = Map::new();
    g(&mut env, &map, &sexp)
}

pub fn g(env: &mut Env, map: &Map, sexp: &Sexp) -> Sexp {
    match sexp {
        Sexp::If(cond, expr_then, expr_else) => Sexp::If(
            Box::new(g(env, map, &cond)),
            Box::new(g(env, map, &expr_then)),
            Box::new(g(env, map, &expr_else)),
        ),
        Sexp::Str(s) => Sexp::Str(s.to_owned()),
        Sexp::Int(i) => Sexp::Int(*i),
        Sexp::Bool(b) => Sexp::Bool(*b),
        Sexp::Var(id) => {
            println!("{:?}", map);
            Sexp::Var(map.get(id).unwrap().to_owned())
        }
        Sexp::LetRec(id, def, expr) => {
            let n = env.get(id).unwrap() + 1;
            env.insert(id.to_owned(), n);
            let mut nmap = map.clone();
            nmap.insert(id.to_owned(), format!("{}({})", id, n));
            println!("letrec {:?} -> {:?}", map, nmap);
            Sexp::Let(
                nmap.get(id).unwrap().to_owned(),
                Box::new(g(env, &nmap, def)),
                Box::new(g(env, &nmap, expr)),
            )
        }
        Sexp::Let(id, def, expr) => {
            let n = env.get(id).cloned().unwrap_or(0) + 1;
            env.insert(id.to_owned(), n);
            let mut nmap = map.clone();
            nmap.insert(id.to_owned(), format!("{}({})", id, n));
            println!("let {:?} -> {:?}", map, nmap);
            Sexp::Let(
                nmap.get(id).unwrap().to_owned(),
                Box::new(g(env, map, def)),
                Box::new(g(env, &nmap, expr)),
            )
        }
        Sexp::Lambda(args, expr) => {
            let mut nmap = map.clone();
            let args = args
                .iter()
                .map(|id| {
                    let n = env.get(id).cloned().unwrap_or(0) + 1;
                    env.insert(id.clone(), n);
                    let alphad = format!("{}({})", id, n);
                    nmap.insert(id.to_owned(), alphad.clone());
                    alphad
                })
                .collect::<Vec<_>>();
            Sexp::Lambda(args, Box::new(g(env, &nmap, expr)))
        }
        Sexp::Call(id, args) => Sexp::Call(
            id.to_owned(),
            args.iter().map(|arg| g(env, map, arg)).collect::<Vec<_>>(),
        ),
        _ => unimplemented!(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn alpha() {
        let ast = Sexp::Let(
            String::from("x"),
            Box::new(Sexp::Let(
                String::from("x"),
                Box::new(Sexp::Int(1)),
                Box::new(Sexp::Var(String::from("x"))),
            )),
            Box::new(Sexp::LetRec(
                String::from("x"),
                Box::new(Sexp::Var(String::from("x"))),
                Box::new(Sexp::Var(String::from("x"))),
            )),
        );

        let expect = Sexp::Let(
            String::from("x(1)"),
            Box::new(Sexp::Let(
                String::from("x(2)"),
                Box::new(Sexp::Int(1)),
                Box::new(Sexp::Var(String::from("x(2)"))),
            )),
            Box::new(Sexp::Let(
                String::from("x(3)"),
                Box::new(Sexp::Var(String::from("x(3)"))),
                Box::new(Sexp::Var(String::from("x(3)"))),
            )),
        );
        assert_eq!(f(ast), expect);
    }
}
