mod lex;
mod parse;

use lex::*;
use parse::*;
use std::env;

fn main() {
    let args: String = env::args()
        .into_iter()
        .skip(1)
        .collect::<Vec<String>>()
        .join(" ");

    let lexed = Lexer::new(&args).run();
    let stack = Parse::new(lexed).parse().0;

    let ran = stack.run();

    println!("{:?}", ran);
}
