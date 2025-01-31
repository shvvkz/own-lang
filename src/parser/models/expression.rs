#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Binary(Box<BinaryExpression>),
    FunctionCall(Box<FunctionCall>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryExpression {
    pub left: Expression,
    pub op: String,
    pub right: Expression,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: Vec<Expression>,
}
