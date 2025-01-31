use crate::parser::models::statement::Statement;

#[derive(Debug, PartialEq, Clone)]
pub struct AST {
    pub statements: Vec<Statement>,
}