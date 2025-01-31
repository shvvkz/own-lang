use crate::lex::models::{token::Token, token_type::TokenType};
use crate::parser::models::ast::AST;
use super::statement_parser;

/// 🚀 The `Parser` structure holds the token stream and the current reading position.
/// It provides methods to navigate and check the token stream.
pub struct Parser {
    /// All tokens provided by the lexer.
    pub tokens: Vec<Token>,

    /// The current index in the `tokens` vector.
    pub position: usize,
}

impl Parser {
    /// 🔧 Creates a new `Parser` from a given vector of `Token`.
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, position: 0 }
    }

    /// 🏁 Parses an entire file, producing an `AST` composed of multiple `Statement`s.
    pub fn parse_file(&mut self) -> AST {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            match statement_parser::parse_statement(self) {
                Some(stmt) => statements.push(stmt),
                None => {
                    eprintln!("Error: could not parse statement. Stopping.");
                    break;
                }
            }
        }

        AST { statements }
    }

    /// ❓ Checks if we have reached the end of the tokens or encountered `EOF`.
    pub fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
            || self.tokens[self.position].token_type == TokenType::EOF
    }

    /// 👀 Retrieves the current token without consuming it.
    pub fn peek(&self) -> &Token {
        &self.tokens[self.position]
    }

    /// ⏩ Consumes (advances past) the current token and returns it.
    pub fn advance(&mut self) -> Token {
        let token = self.tokens[self.position].clone();
        self.position += 1;
        token
    }

    /// 🧐 Checks if the current token is of a specified `TokenType`.
    pub fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }

    /// ⚙️ Verifies whether the current token is any operator listed in `ops`.
    pub fn check_operator(&self, ops: &[&str]) -> bool {
        if self.is_at_end() {
            return false;
        }
        let token = self.peek();
        if token.token_type != TokenType::Operator {
            return false;
        }
        ops.contains(&token.value.as_str())
    }

    /// ✅ Consumes a token of the expected `TokenType` or prints an error message if mismatched.
    pub fn consume(&mut self, ttype: TokenType, err_msg: &str) -> Option<Token> {
        if self.check(ttype.clone()) {
            Some(self.advance())
        } else {
            eprintln!("Parser error: {}", err_msg);
            None
        }
    }

    /// 🔍 Checks whether the current token is a specific keyword (like "let", "return", etc.).
    pub fn is_keyword(&self, kw: &str) -> bool {
        if self.is_at_end() {
            return false;
        }
        let t = self.peek();
        t.token_type == TokenType::Keyword && t.value == kw
    }

    /// 🗝️ Consumes the given `keyword` if it matches the current token, otherwise logs an error.
    pub fn consume_keyword(&mut self, keyword: &str) -> Option<Token> {
        if self.is_keyword(keyword) {
            Some(self.advance())
        } else {
            eprintln!("Parser error: Expected keyword '{}'", keyword);
            None
        }
    }
}
