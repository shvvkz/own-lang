use super::expression_parser;
use super::models::statement::{IfStatement, SwitchCase, SwitchStatement, VarAffection};
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
    } else if parser.is_keyword("if") {
        parse_if_stmt(parser).map(Statement::If)
    } else if parser.is_keyword("switch") {
        parse_switch_stmt(parser).map(Statement::Switch)
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
        "Expected identifier for variable affection",
    )?;
    let name = name_token.value;

    parser.consume(TokenType::Equals, "Expected '=' in variable affection")?;

    let value_expr = expression_parser::parse_expression(parser)?;

    parser.consume(
        TokenType::Semicolon,
        "Expected ';' at the end of variable affection",
    )?;

    Some(VarAffection {
        name,
        value: value_expr,
    })
}

pub fn parse_if_stmt(parser: &mut Parser) -> Option<IfStatement> {
    parser.consume_keyword("if")?;
    parser.consume(TokenType::LeftParen, "Expected '(' after 'if'")?;
    let condition = expression_parser::parse_expression(parser)?;

    parser.consume(TokenType::RightParen, "Expected ')' after condition")?;
    parser.consume(TokenType::LeftBracket, "Expected '{' after if condition")?;

    let then_branch = parse_block_like(parser)?;
    parser.consume(TokenType::RightBracket, "Expected '}' after if block")?;
    let else_branch = if parser.is_keyword("else") {
        parser.advance();
        parser.consume(TokenType::LeftBracket, "Expected '{' after 'else'")?;
        let branch = parse_block_like(parser)?;
        parser.consume(TokenType::RightBracket, "Expected '}' after else block")?;

        Some(branch)
    } else {
        None
    };

    Some(IfStatement {
        condition,
        then_branch,
        else_branch,
    })
}

fn parse_switch_stmt(parser: &mut Parser) -> Option<SwitchStatement> {
    parser.consume_keyword("switch")?;
    parser.consume(TokenType::LeftParen, "Expected '(' after 'switch'")?;
    let condition = expression_parser::parse_expression(parser)?;
    parser.consume(TokenType::RightParen, "Expected ')' after switch condition")?;
    parser.consume(TokenType::LeftBracket, "Expected '{' after switch(...)")?;

    let mut cases = Vec::new();
    let mut default_block = None;

    while !parser.check(TokenType::RightBracket) && !parser.is_at_end() {
        if parser.is_keyword("case") {
            parser.advance();
            let value = expression_parser::parse_expression(parser)?;
            parser.consume(TokenType::LeftBracket, "Expected '{' after case expression")?;
            let body = parse_block_like(parser)?;
            parser.consume(
                TokenType::RightBracket,
                "Expected '}' at the end of case block",
            )?;
            cases.push(SwitchCase { value, body });

            if parser.check(TokenType::Comma) {
                parser.advance();
            }
        } else if parser.is_keyword("default") {
            parser.advance();
            parser.consume(TokenType::LeftBracket, "Expected '{' after 'default'")?;
            let block = parse_block_like(parser)?;
            parser.consume(
                TokenType::RightBracket,
                "Expected '}' at the end of default block",
            )?;
            default_block = Some(block);

            if parser.check(TokenType::Comma) {
                parser.advance();
            }
        } else {
            eprintln!(
                "Error while parsing switch statement: unexpected token: {:?}",
                parser.peek()
            );
            parser.advance();
            break;
        }
    }

    parser.consume(TokenType::RightBracket, "Expected '}' after switch block")?;
    Some(SwitchStatement {
        condition,
        cases,
        default: default_block,
    })
}

/// Lit une suite de statements jusqu'à rencontrer la `}` ou la fin du fichier.
pub fn parse_block_like(parser: &mut Parser) -> Option<Vec<Statement>> {
    let mut statements = Vec::new();

    while !parser.check(TokenType::RightBracket) && !parser.is_at_end() {
        match parse_statement(parser) {
            Some(stmt) => statements.push(stmt),
            None => {
                eprintln!("Error while parsing statements in block");
                break;
            }
        }
    }

    Some(statements)
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
