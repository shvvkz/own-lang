use crate::parser::models::expression::Expression;

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    VarDeclaration(VarDeclaration),
    VarAffection(VarAffection),
    Return(Option<Expression>),
    If(IfStatement),
    Switch(SwitchStatement),
    While(WhileStatement),
    FunctionDeclaration(FunctionDeclaration),
    ExpressionStatement(Expression),
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
pub struct IfStatement {
    pub condition: Expression,
    pub then_branch: Vec<Statement>,
    pub else_branch: Option<Vec<Statement>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SwitchStatement {
    pub condition: Expression,
    pub cases: Vec<SwitchCase>,
    pub default: Option<Vec<Statement>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SwitchCase {
    pub value: Expression,
    pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct WhileStatement{
    pub condition: Expression,
    pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDeclaration{
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: String,
    pub body: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Parameter {
    pub name: String,
    pub type_name: String,
}
