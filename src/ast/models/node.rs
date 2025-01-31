#[derive(Debug, PartialEq, Clone)]
pub struct AST {
    pub statements: Vec<Statement>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    VarDecl(VarDecl),
    FunctionDecl(FunctionDecl),

    // Plus tard, vous pourriez ajouter :
    // Return(Expression),
    // If { ... },
    // While { ... },
    // etc.
}

#[derive(Debug, PartialEq, Clone)]
pub struct VarDecl {
    pub name: String,
    pub type_name: String,
    pub init: Option<Expression>,
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

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Ident(String),
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
}
