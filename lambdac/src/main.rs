extern crate clap;
extern crate lambdac;

use std::fs;

fn main() {
    let matches = clap::App::new("lambdac")
        .arg(
            clap::Arg::with_name("SOURCE")
                .takes_value(true)
                .required(true),
        )
        .get_matches();
    let fname = matches.value_of("SOURCE").unwrap();
    let src = fs::read_to_string(fname).unwrap();
    let parsed = lambdac::parser::f(&src);
    let alpha = lambdac::alpha::f(&lambdac::alpha::Env::new(), &parsed);
    println!("{:#?}", alpha);
}
