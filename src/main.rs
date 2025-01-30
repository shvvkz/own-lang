pub mod lex;

use lex::{lexer::Lexer, models::token_type::TokenType};

fn main() {
    //let input = String::from("function calc(x: float, y: char){\nlet x = 5.0;\nlet y = 'z';\nlet result = x + y;\nreturn result;\n}");
    let input = std::fs::read_to_string("test.own").unwrap();
    println!("{}", input);
    let mut lexer = Lexer::new(input);

    loop {
        let token = lexer.next_token();
        if token.token_type == TokenType::EOF {
            break;
        }
        println!("{:?}", token);
    }
    
}