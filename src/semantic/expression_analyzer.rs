// semantic/expression_analyzer.rs

use crate::parser::models::expression::Expression;
use crate::semantic::models::semantic::{Symbol, SymbolType, SymbolTable};
use crate::semantic::analyzer::SemanticAnalyzer;
use std::collections::HashMap;

pub trait ExpressionAnalyzer {
    /// Analyse une expression et retourne son type.
    fn get_expression_type(&mut self, expr: &Expression) -> Option<String>;
}

impl ExpressionAnalyzer for SemanticAnalyzer {
    fn get_expression_type(&mut self, expr: &Expression) -> Option<String> {
        match expr {
            Expression::Ident(name) => {
                self.symbol_table.resolve(name).map(|symbol| match &symbol.symbol_type {
                    SymbolType::Variable(type_name) => type_name.clone(),
                    SymbolType::Function { return_type, .. } => return_type.clone(),
                })
            },
            Expression::Int(_) => Some("int".to_string()),
            Expression::Float(_) => Some("float".to_string()),
            Expression::Bool(_) => Some("bool".to_string()),
            Expression::Str(_) => Some("string".to_string()),
            Expression::Binary(bin_expr) => {
                let left_type = self.get_expression_type(&bin_expr.left)?;
                let right_type = self.get_expression_type(&bin_expr.right)?;

                if left_type != right_type {
                    self.errors.push(format!(
                        "Type mismatch in binary expression: '{}' and '{}'.",
                        left_type, right_type
                    ));
                    return None;
                }


                match bin_expr.op.as_str() {
                    "+" | "-" | "*" | "/" => Some(left_type.clone()),
                    "==" | "!=" | "<" | "<=" | ">" | ">=" => Some("bool".to_string()),
                    _ => {
                        self.errors.push(format!(
                            "Unknown binary operator '{}'.",
                            bin_expr.op
                        ));
                        None
                    }
                }
            },
            Expression::FunctionCall(call) => {
                if let Some(symbol) = self.symbol_table.resolve(&call.name) {
                    match &symbol.symbol_type {
                        SymbolType::Function { return_type, .. } => Some(return_type.clone()),
                        _ => {
                            self.errors.push(format!("'{}' is not a function.", call.name));
                            None
                        }
                    }
                } else {
                    self.errors.push(format!("Undefined function '{}'.", call.name));
                    None
                }
            },
            // Gérer d'autres types d'expressions si nécessaire
            _ => {
                self.errors.push(format!(
                    "Unsupported expression type: {:?}.",
                    expr
                ));
                None
            }
        }
    }
}
