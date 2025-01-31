use crate::ast::parser::Parser;
use crate::lex::lexer::Lexer;
use crate::lex::models::token_type::TokenType;

mod ast;
mod lex;

fn main() {
    let input = std::fs::read_to_string("test.own").unwrap();
    println!("Source:\n{}", input);

    // 1) On génère des tokens avec le lexer
    let mut lexer = Lexer::new(input);
    let mut tokens = Vec::new();

    loop {
        let token = lexer.next_token();
        if token.token_type == TokenType::EOF {
            break;
        }
        tokens.push(token);
    }

    println!("Tokens: {:#?}", tokens);

    // 2) On les envoie au parser
    let mut parser = Parser::new(tokens);
    let ast_nodes = parser.parse();

    // 3) On affiche l’AST résultant
    println!("AST: {:#?}", ast_nodes);
}
