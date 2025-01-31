use crate::parser::models::expression::{Expression, BinaryExpression};
use crate::lex::models::token_type::TokenType;
use super::parser::Parser;

/// ✨ Parses a full expression by starting with the highest-level function
/// and returning the resulting `Expression`.
pub fn parse_expression(parser: &mut Parser) -> Option<Expression> {
    parse_equality(parser)
}

/// ⚖️ Parses equality operators (`==`, `!=`).
pub fn parse_equality(parser: &mut Parser) -> Option<Expression> {
    let mut expr = parse_comparison(parser)?;
    while parser.check_operator(&["==", "!="]) {
        let op_token = parser.advance();
        let op = op_token.value;
        let right = parse_comparison(parser)?;
        expr = Expression::Binary(Box::new(BinaryExpression { left: expr, op, right }));
    }
    Some(expr)
}

/// 🔍 Parses comparison operators (`<`, `<=`, `>`, `>=`).
pub fn parse_comparison(parser: &mut Parser) -> Option<Expression> {
    let mut expr = parse_term(parser)?;
    while parser.check_operator(&["<", "<=", ">", ">="]) {
        let op_token = parser.advance();
        let op = op_token.value;
        let right = parse_term(parser)?;
        expr = Expression::Binary(Box::new(BinaryExpression { left: expr, op, right }));
    }
    Some(expr)
}

/// ➕ Parses addition and subtraction operators (`+`, `-`).
pub fn parse_term(parser: &mut Parser) -> Option<Expression> {
    let mut expr = parse_factor(parser)?;
    while parser.check_operator(&["+", "-"]) {
        let op_token = parser.advance();
        let op = op_token.value;
        let right = parse_factor(parser)?;
        expr = Expression::Binary(Box::new(BinaryExpression { left: expr, op, right }));
    }
    Some(expr)
}

/// ✖️ Parses multiplication, division, and modulo operators (`*`, `/`, `%`).
pub fn parse_factor(parser: &mut Parser) -> Option<Expression> {
    let mut expr = parse_unary(parser)?;
    while parser.check_operator(&["*", "/", "%"]) {
        let op_token = parser.advance();
        let op = op_token.value;
        let right = parse_unary(parser)?;
        expr = Expression::Binary(Box::new(BinaryExpression { left: expr, op, right }));
    }
    Some(expr)
}

/// 🚀 Parses unary operators like `-` and `!`.
pub fn parse_unary(parser: &mut Parser) -> Option<Expression> {
    if parser.check_operator(&["-","!"]) {
        let op_token = parser.advance();
        let op = op_token.value;
        let right = parse_unary(parser)?;
        return Some(Expression::Binary(Box::new(BinaryExpression {
            left: Expression::Int(0),
            op,
            right,
        })));
    }
    parse_primary(parser)
}

/// 🏷️ Parses primary elements: parentheses, literals, and identifiers.
pub fn parse_primary(parser: &mut Parser) -> Option<Expression> {
    if parser.check(TokenType::LeftParen) {
        parser.advance();
        let expr = parse_expression(parser)?;
        parser.consume(TokenType::RightParen, "Expected ')'")?;
        return Some(expr);
    }
    let token = parser.advance();
    match token.token_type {
        TokenType::Int => {
            if let Ok(val) = token.value.parse::<i64>() {
                Some(Expression::Int(val))
            } else {
                eprintln!("Cannot parse int from '{}'", token.value);
                None
            }
        }
        TokenType::Float => {
            if let Ok(val) = token.value.parse::<f64>() {
                Some(Expression::Float(val))
            } else {
                eprintln!("Cannot parse float from '{}'", token.value);
                None
            }
        }
        TokenType::Identifier => Some(Expression::Ident(token.value)),
        TokenType::Bool => {
            let b = token.value == "true";
            Some(Expression::Bool(b))
        }
        TokenType::String => Some(Expression::Str(token.value)),
        _ => {
            eprintln!("Unexpected token in parse_primary: {:?}", token);
            None
        }
    }
}
