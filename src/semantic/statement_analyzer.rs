// semantic/statement_analyzer.rs

use crate::parser::models::statement::Statement;
use crate::semantic::analyzer::SemanticAnalyzer;

pub trait StatementAnalyzer {
    /// Analyse un statement.
    fn analyze_statement(&mut self, stmt: &Statement);
}

impl StatementAnalyzer for SemanticAnalyzer {
    fn analyze_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::VarDeclaration(var_decl) => {
                self.analyze_var_declaration(var_decl);
            }
            Statement::Return(expr_opt) => {
                self.analyze_return_statement(expr_opt);
            }
            Statement::ExpressionStatement(expr) => {
                self.analyze_expression(expr);
            }
            Statement::If(if_stmt) => {
                self.analyze_if_statement(if_stmt);
            }
            Statement::For(for_stmt) => {
                self.analyze_for_statement(for_stmt);
            }
            Statement::While(while_stmt) => {
                self.analyze_while_statement(while_stmt);
            }
            Statement::Switch(switch_stmt) => {
                self.analyze_switch_statement(switch_stmt);
            }
            Statement::VarAffection(var_affection) => {
                self.analyze_var_affection(var_affection);
            }
            Statement::FunctionDeclaration(func_decl) => {
                self.analyze_function_declaration(func_decl);
            }
        }
    }
}
