use crate::lex::models::token::Token;
use crate::lex::models::token_type::TokenType;

use super::models::token_reader::TokenReader;

pub struct Lexer {
    pub input: String,
    pub position: usize,
    pub read_position: usize,
    pub ch: char,
}

impl Lexer {
    pub fn new(input: String) -> Lexer {
        let mut l = Lexer {
            input,
            position: 0,
            read_position: 0,
            ch: '\0',
        };
        l.read_char();
        l
    }

    pub fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = '\0';
        } else {
            self.ch = self.input.as_bytes()[self.read_position] as char;
        }
        self.position = self.read_position;
        self.read_position += 1;
    }

    fn skip_whitespace(&mut self) {
        while self.ch.is_whitespace() {
            self.read_char();
        }
    }

    fn get_token_type(word: &str) -> TokenType {
        match word {
            "let" | "if" | "else" | "return" | "function" => TokenType::Keyword,
            "int" | "float" | "bool" | "string" => TokenType::Type,
            "true" | "false" => TokenType::Bool,
            ";" => TokenType::Semicolon,
            ":" => TokenType::Colon,
            "," => TokenType::Comma,
            "=" => TokenType::Equals,
            "+" | "-" | "*" | "/" | "==" | "<=" | ">=" | ">" | "<" | "%" => TokenType::Operator,
            "(" => TokenType::LeftParen,
            ")" => TokenType::RightParen,
            "{" => TokenType::LeftBracket,
            "}" => TokenType::RightBracket,
            _ if word.parse::<i64>().is_ok() => TokenType::Int,
            _ if word.parse::<f64>().is_ok() => TokenType::Float,
            _ => TokenType::Identifier,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.ch == '\0' {
            return Token {
                token_type: TokenType::EOF,
                value: "".to_string(),
            };
        }

        if self.ch.is_alphabetic() {
            let word = self.read_identifier();
            let token_type = Self::get_token_type(&word);
            return Token { token_type, value: word };
        }

        if self.ch.is_numeric() {
            let number = self.read_number();
            let token_type = if number.contains('.') { TokenType::Float } else { TokenType::Int };
            return Token { token_type, value: number };
        }

        if self.ch == '"' || self.ch == '\'' {
            let string_value = self.read_string();
            return Token { token_type: TokenType::String, value: string_value };
        }

        let single_char = self.ch.to_string();
        let token_type = Self::get_token_type(&single_char);
        self.read_char();

        Token { token_type, value: single_char }
    }
}



impl TokenReader for Lexer {
    fn read_identifier(&mut self) -> String {
        let position = self.position;
        while self.ch.is_alphanumeric() {
            self.read_char();
        }
        self.input[position..self.position].to_string()
    }

    fn read_number(&mut self) -> String {
        let position = self.position;
        let mut has_dot = false;

        while self.ch.is_numeric() || (self.ch == '.' && !has_dot) {
            if self.ch == '.' {
                has_dot = true;
            }
            self.read_char();
        }

        self.input[position..self.position].to_string()
    }

    fn read_string(&mut self) -> String {
        let mut result = String::new();
        let delimiter = self.ch;
        self.read_char();

        while self.ch != delimiter && self.ch != '\0' {
            result.push(self.ch);
            self.read_char();
        }
        self.read_char();
        result
    }
}
