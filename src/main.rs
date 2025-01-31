use semantic::analyzer;

use crate::parser::parser::Parser;
use crate::lex::lexer::Lexer;
use crate::lex::models::token_type::TokenType;

mod parser;
mod lex;
mod semantic;

fn main() {
    let input = std::fs::read_to_string("own_files/all.own").unwrap();
    let mut analyzer = analyzer::SemanticAnalyzer::new(input);
    println!("{:?}",analyzer.analyze());
}
