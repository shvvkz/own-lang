use crate::parser::parser::Parser;
use crate::lex::lexer::Lexer;
use crate::lex::models::token_type::TokenType;

mod parser;
mod lex;

fn main() {
    let input = std::fs::read_to_string("own_files/declaration.own").unwrap();
    println!("Source:\n{}", input);

    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();

    loop {
        let token = lexer.next_token();
        if token.token_type == TokenType::EOF {
            break;
        }
        tokens.push(token);
    }

    let mut parser = Parser::new(tokens);
    let ast_nodes = parser.parse_file();

    println!("AST: {:#?}", ast_nodes);
}
