use std::fmt;

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
impl fmt::Display for Expression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expression::Ident(s) => write!(f, "{}", s),
            Expression::Int(i) => write!(f, "{}", i),
            Expression::Float(fl) => write!(f, "{}", fl),
            Expression::Str(s) => write!(f, "\"{}\"", s),
            Expression::Bool(b) => write!(f, "{}", b),
            Expression::Binary(b) => write!(f, "{}", b),
            Expression::FunctionCall(fc) => write!(f, "{}", fc),
        }
    }
}

impl AsRef<str> for Expression {
    fn as_ref(&self) -> &str {
        match self {
            Expression::Ident(s) => s,
            Expression::Str(s) => s,
            _ => panic!("Cannot convert this expression to &str"),
        }
    }
}

impl fmt::Display for BinaryExpression {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({} {} {})", self.left, self.op, self.right)
    }
}

impl fmt::Display for FunctionCall {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let args: Vec<String> = self.arguments.iter().map(|arg| arg.to_string()).collect();
        write!(f, "{}({})", self.name, args.join(", "))
    }
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
