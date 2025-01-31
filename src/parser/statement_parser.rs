use super::expression_parser;
use super::models::statement::VarAffection;
use super::parser::Parser;
use crate::lex::models::token_type::TokenType;
use crate::parser::models::statement::{Statement, VarDeclaration};

/// Parses a single statement (variable declaration, return, etc.).
pub fn parse_statement(parser: &mut Parser) -> Option<Statement> {
    if parser.is_keyword("let") {
        parse_var_decl(parser).map(Statement::VarDeclaration)
    } else if parser.is_keyword("return") {
        parse_return_stmt(parser)
    } else if is_var_affection(parser) {
        parse_var_affection(parser).map(Statement::VarAffection)
    } else {
        eprintln!("Parser warning: unexpected token: {:?}", parser.peek());
        parser.advance();
        None
    }
}

/// Parses a variable declaration of the form `let x: type = expr;`.
fn parse_var_decl(parser: &mut Parser) -> Option<VarDeclaration> {
    parser.consume_keyword("let")?;

    let name_token = parser.consume(TokenType::Identifier, "Expected identifier after 'let'")?;
    let name = name_token.value;

    parser.consume(TokenType::Colon, "Expected ':' after identifier")?;

    let type_token = parser.consume(
        TokenType::Type,
        "Expected a type keyword (e.g. float, string) after ':'",
    )?;
    let type_name = type_token.value;

    let mut init = None;
    if parser.check(TokenType::Equals) {
        parser.advance();
        init = expression_parser::parse_expression(parser);
    }

    parser.consume(
        TokenType::Semicolon,
        "Expected ';' at the end of variable declaration",
    )?;

    Some(VarDeclaration {
        name,
        type_name,
        init,
    })
}

/// Parses a return statement, which can be `return expr;` or `return;`.
fn parse_return_stmt(parser: &mut Parser) -> Option<Statement> {
    parser.consume_keyword("return")?;

    let expr = if parser.check(TokenType::Semicolon) {
        None
    } else {
        Some(expression_parser::parse_expression(parser)?)
    };

    parser.consume(TokenType::Semicolon, "Expected ';' after return")?;

    Some(Statement::Return(expr))
}

fn parse_var_affection(parser: &mut Parser) -> Option<VarAffection> {
    let name_token = parser.consume(
        TokenType::Identifier,
        "Expected identifier for variable affection"
    )?;
    let name = name_token.value;

    parser.consume(TokenType::Equals, "Expected '=' in variable affection")?;

    let value_expr = expression_parser::parse_expression(parser)?;

    parser.consume(
        TokenType::Semicolon,
        "Expected ';' at the end of variable affection"
    )?;

    Some(VarAffection { name, value: value_expr })
}


fn is_var_affection(parser: &Parser) -> bool {
    if parser.is_at_end() {
        return false;
    }

    if !parser.check(TokenType::Identifier) {
        return false;
    }

    let current_position = parser.position;
    let next_position = current_position + 1;
    if next_position >= parser.tokens.len() {
        return false;
    }
    let next_token = &parser.tokens[next_position];
    next_token.token_type == TokenType::Equals
}

