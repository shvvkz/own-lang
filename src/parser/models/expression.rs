#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Binary(Box<BinaryExpression>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryExpression {
    pub left: Expression,
    pub op: String,
    pub right: Expression,
}
