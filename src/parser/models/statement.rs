use crate::parser::models::expression::Expression;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    VarDeclaration(VarDeclaration),
    VarAffection(VarAffection),
    Return(Option<Expression>),
    FunctionDecl(FunctionDecl),
    // Autres statements (If, While, etc.)
}

#[derive(Debug, PartialEq, Clone)]
pub struct VarDeclaration {
    pub name: String,
    pub type_name: String,
    pub init: Option<Expression>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct VarAffection {
    pub name: String,
    pub value: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Return {
    pub value: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDecl {
    pub name: String,
    pub params: Vec<Parameter>,
    pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter {
    pub name: String,
    pub type_name: String,
}
