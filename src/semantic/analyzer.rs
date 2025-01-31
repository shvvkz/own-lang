// semantic/analyzer.rs

use crate::parser::models::ast::AST;
use crate::parser::models::expression::Expression;
use crate::parser::models::statement::{ForStatement, FunctionDeclaration, Statement, SwitchStatement, VarAffection, WhileStatement};
use crate::parser::parser::Parser;
use crate::semantic::models::semantic::{Symbol, SymbolType, SymbolTable};
use crate::semantic::statement_analyzer::StatementAnalyzer;

pub struct SemanticAnalyzer {
    pub symbol_table: SymbolTable,
    pub errors: Vec<String>,
    pub current_function_return_type: Option<String>,
    pub ast: AST
}

impl SemanticAnalyzer {
    /// Crée un nouvel analyseur sémantique avec une table de symboles globale.
    pub fn new(input: String) -> Self {
        let mut parser= Parser::new(input);
        let ast = parser.parse_file();
        SemanticAnalyzer {
            symbol_table: SymbolTable::new(None),
            errors: Vec::new(),
            current_function_return_type: None,
            ast
        }
    }

    /// Lance l'analyse sémantique sur l'AST.
    pub fn analyze(&mut self) -> Vec<String> {
        let statements = self.ast.statements.clone();
        for stmt in &statements {
            self.analyze_statement(stmt);
        }
        return self.errors.clone();
    }

    /// Implémentation des autres méthodes comme `analyze_var_declaration`, `analyze_return_statement`, etc.
    /// Vous pouvez les définir ici ou dans des modules séparés si vous préférez.
    
    /// Analyse une déclaration de variable.
    pub fn analyze_var_declaration(&mut self, var_decl: &crate::parser::models::statement::VarDeclaration) {
        // Vérifier si le type existe
        if !self.is_type_defined(&var_decl.type_name) {
            self.errors.push(format!("Type '{}' is not defined.", var_decl.type_name));
        }

        // Vérifier si la variable est déjà définie dans le scope courant
        let symbol = Symbol {
            name: var_decl.name.clone(),
            symbol_type: SymbolType::Variable(var_decl.type_name.clone()),
        };
        if let Err(err) = self.symbol_table.define(var_decl.name.clone(), symbol) {
            self.errors.push(err);
        }

        // Vérifier l'initialisation si présente
        if let Some(expr) = &var_decl.init {
            let expr_type = self.get_expression_type(expr);
            if let Some(expr_type) = expr_type {
                if &expr_type != &var_decl.type_name {
                    self.errors.push(format!(
                        "Type mismatch in variable declaration '{}': expected '{}', found '{}'.",
                        var_decl.name, var_decl.type_name, expr_type
                    ));
                }
            }
        }
    }

    /// Analyse une instruction `return`.
    pub fn analyze_return_statement(&mut self, expr_opt: &Option<crate::parser::models::expression::Expression>) {
        // Vérifier si on est à l'intérieur d'une fonction
        let current_function_return_type = self.current_function_return_type.clone();
        if let Some(expected_return_type) = &current_function_return_type {
            if let Some(expr) = expr_opt {
                // Analyser l'expression de retour pour déterminer son type
                let expr_type = self.get_expression_type(expr);
                if let Some(expr_type) = expr_type {
                    if &expr_type != expected_return_type {
                        self.errors.push(format!(
                            "Type mismatch in return statement: expected '{}', found '{}'.",
                            expected_return_type, expr_type
                        ));
                    }
                }
            } else {
                // Si aucune expression n'est fournie, vérifier si le type de retour attendu est `void`
                if expected_return_type != "void" {
                    self.errors.push(format!(
                        "Return statement missing a value: expected '{}'.",
                        expected_return_type
                    ));
                }
            }
        } else {
            // Si on n'est pas à l'intérieur d'une fonction, une instruction `return` est invalide
            self.errors.push("Return statement not inside a function.".to_string());
        }
    }

    /// Analyse une affection de variable (assignment).
    pub fn analyze_var_affection(&mut self, var_affection: &VarAffection) {
        // Vérifier que la variable est déclarée
        if self.symbol_table.resolve(&var_affection.name).is_none() {
            self.errors.push(format!("Undefined variable '{}'.", var_affection.name));
            // Continuer l'analyse pour détecter d'autres erreurs
        }

        // Analyser l'expression assignée
        let expr_type = self.get_expression_type(&var_affection.value);

        // Vérifier que le type de l'expression correspond au type de la variable
        if let Some(var_symbol) = self.symbol_table.resolve(&var_affection.name) {
            match &var_symbol.symbol_type {
                SymbolType::Variable(var_type) => {
                    if let Some(expr_type) = expr_type {
                        if expr_type != *var_type {
                            self.errors.push(format!(
                                "Type mismatch in assignment to '{}': expected '{}', found '{}'.",
                                var_affection.name, var_type, expr_type
                            ));
                        }
                    }
                }
                _ => {
                    self.errors.push(format!("'{}' is not a variable.", var_affection.name));
                }
            }
        }
    }

    /// Analyse une instruction `if`.
    pub fn analyze_if_statement(&mut self, if_stmt: &crate::parser::models::statement::IfStatement) {
        // Analyser la condition
        let cond_type = self.get_expression_type(&if_stmt.condition);
        if let Some(cond_type) = cond_type {
            if cond_type != "bool" {
                self.errors.push(format!(
                    "Condition in 'if' statement must be of type 'bool', found '{}'.",
                    cond_type
                ));
            }
        } else {
            self.errors.push("Unable to determine the type of the condition in 'if' statement.".to_string());
        }

        // Analyser le bloc `then`
        self.enter_scope();
        for stmt in &if_stmt.then_branch {
            self.analyze_statement(stmt);
        }
        self.exit_scope();

        // Analyser le bloc `else` s'il existe
        if let Some(else_branch) = &if_stmt.else_branch {
            self.enter_scope();
            for stmt in else_branch {
                self.analyze_statement(stmt);
            }
            self.exit_scope();
        }
    }

    pub fn analyze_for_statement(&mut self, for_stmt: &ForStatement) {
        self.enter_scope();

        // Analyser l'initialisation
        self.analyze_statement(&for_stmt.init);

        // Analyser la condition
        // La condition doit être une expression retournant un booléen
        match &*for_stmt.cond {
            Statement::ExpressionStatement(expr) => {
                let cond_type = self.get_expression_type(expr);
                if let Some(cond_type) = cond_type {
                    if cond_type != "bool" {
                        self.errors.push(format!(
                            "Condition in 'for' statement must be of type 'bool', found '{}'.",
                            cond_type
                        ));
                    }
                } else {
                    self.errors.push("Unable to determine the type of the condition in 'for' statement.".to_string());
                }
            }
            _ => {
                self.errors.push("Condition in 'for' statement must be an expression statement.".to_string());
            }
        }

        // Analyser l'incrément
        self.analyze_statement(&for_stmt.incr);

        // Analyser le corps de la boucle
        for stmt in &for_stmt.body {
            self.analyze_statement(stmt);
        }

        self.exit_scope();
    }

    /// Analyse une boucle `while`.
    pub fn analyze_while_statement(&mut self, while_stmt: &WhileStatement) {
        // Analyser la condition
        let cond_type = self.get_expression_type(&while_stmt.condition);
        if let Some(cond_type) = cond_type {
            if cond_type != "bool" {
                self.errors.push(format!(
                    "Condition in 'while' statement must be of type 'bool', found '{}'.",
                    cond_type
                ));
            }
        } else {
            self.errors.push("Unable to determine the type of the condition in 'while' statement.".to_string());
        }

        // Analyser le corps de la boucle dans un nouveau scope
        self.enter_scope();
        for stmt in &while_stmt.body {
            self.analyze_statement(stmt);
        }
        self.exit_scope();
    }

    /// Analyse une instruction `switch`.
    pub fn analyze_switch_statement(&mut self, switch_stmt: &SwitchStatement) {
        // Analyser l'expression du switch
        let switch_type = self.get_expression_type(&switch_stmt.condition);
        if let Some(switch_type) = switch_type {
            // Analyser chaque cas
            for case in &switch_stmt.cases {
                let case_type = self.get_expression_type(&case.value);
                if let Some(case_type) = case_type {
                    if case_type != switch_type {
                        self.errors.push(format!(
                            "Case type '{}' does not match switch type '{}'.",
                            case_type, switch_type
                        ));
                    }
                } else {
                    self.errors.push("Unable to determine the type of a case in 'switch' statement.".to_string());
                }

                // Analyser le corps du cas dans un nouveau scope
                self.enter_scope();
                for stmt in &case.body {
                    self.analyze_statement(stmt);
                }
                self.exit_scope();
            }

            // Analyser le corps du `default` s'il existe
            if let Some(default_body) = &switch_stmt.default {
                self.enter_scope();
                for stmt in default_body {
                    self.analyze_statement(stmt);
                }
                self.exit_scope();
            }
        } else {
            self.errors.push("Unable to determine the type of the condition in 'switch' statement.".to_string());
        }
    }

    /// Analyse une déclaration de fonction.
    pub fn analyze_function_declaration(&mut self, func_decl: &FunctionDeclaration) {
        // Construire le type de la fonction
        let param_types: Vec<String> = func_decl
            .parameters
            .iter()
            .map(|p| p.type_name.clone())
            .collect();
        let func_type = SymbolType::Function {
            parameters: param_types,
            return_type: func_decl.return_type.clone(),
        };

        // Ajouter la fonction à la table des symboles
        let symbol = Symbol {
            name: func_decl.name.clone(),
            symbol_type: func_type,
        };
        if let Err(err) = self.symbol_table.define(func_decl.name.clone(), symbol) {
            self.errors.push(err);
        }

        // Créer un nouveau scope pour les paramètres et le corps de la fonction
        self.enter_scope();

        // Ajouter les paramètres à la table des symboles
        for param in &func_decl.parameters {
            // Vérifier si le type du paramètre est défini
            if !self.is_type_defined(&param.type_name) {
                self.errors.push(format!(
                    "Type '{}' is not defined for parameter '{}'.",
                    param.type_name, param.name
                ));
            }

            let param_symbol = Symbol {
                name: param.name.clone(),
                symbol_type: SymbolType::Variable(param.type_name.clone()),
            };
            if let Err(err) = self.symbol_table.define(param.name.clone(), param_symbol) {
                self.errors.push(err);
            }
        }

        // Définir le type de retour courant
        let previous_return_type = self.current_function_return_type.take();
        self.current_function_return_type = Some(func_decl.return_type.clone());

        // Analyser le corps de la fonction
        for stmt in &func_decl.body {
            self.analyze_statement(stmt);
        }

        // Restaurer le type de retour précédent
        self.current_function_return_type = previous_return_type;

        self.exit_scope();
    }

    pub fn get_expression_type(&mut self, expr: &Expression) -> Option<String> {
        match expr {
            Expression::Ident(name) => {
                self.symbol_table.resolve(name).map(|symbol| match &symbol.symbol_type {
                    SymbolType::Variable(type_name) => type_name.clone(),
                    SymbolType::Function { return_type, .. } => return_type.clone(),
                    // Gérer d'autres types de symboles si nécessaire
                })
            },
            Expression::Int(_) => Some("int".to_string()),
            Expression::Float(_) => Some("float".to_string()),
            Expression::Bool(_) => Some("bool".to_string()),
            Expression::Str(_) => Some("string".to_string()),
            Expression::Binary(bin_expr) => {
                let left_type = self.get_expression_type(&bin_expr.left)?;
                let right_type = self.get_expression_type(&bin_expr.right)?;
                
                // Vérifier que les types des opérandes correspondent
                if left_type != right_type {
                    self.errors.push(format!(
                        "Type mismatch in binary expression: '{}' and '{}'.",
                        left_type, right_type
                    ));
                    return None;
                }

                // Déterminer le type résultant basé sur l'opérateur
                match bin_expr.op.as_str() {
                    "+" | "-" | "*" | "/" => Some(left_type.clone()), // Supposons que ces opérateurs retournent le même type que les opérandes
                    "==" | "!=" | "<" | "<=" | ">" | ">=" => Some("bool".to_string()), // Comparaisons retournent bool
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

    pub fn analyze_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Ident(name) => {
                if self.symbol_table.resolve(name).is_none() {
                    self.errors.push(format!("Undefined variable '{}'.", name));
                }
            }
            Expression::Binary(bin_expr) => {
                self.analyze_expression(&bin_expr.left);
                self.analyze_expression(&bin_expr.right);
                // Ici, vous pourriez vérifier que les opérandes sont compatibles avec l'opérateur
            }
            Expression::FunctionCall(call) => {
                if let Some(symbol) = self.symbol_table.resolve(&call.name) {
                    match &symbol.symbol_type {
                        SymbolType::Function { parameters, return_type: _ } => {
                            if parameters.len() != call.arguments.len() {
                                self.errors.push(format!(
                                    "Function '{}' expects {} arguments, but {} were provided.",
                                    call.name,
                                    parameters.len(),
                                    call.arguments.len()
                                ));
                            }
                            // Vérifier les types des arguments si vous avez un système de types
                            for arg in &call.arguments {
                                self.analyze_expression(arg);
                            }
                        }
                        _ => {
                            self.errors
                                .push(format!("'{}' is not a function.", call.name));
                        }
                    }
                } else {
                    self.errors
                        .push(format!("Undefined function '{}'.", call.name));
                }
            }
            // Gérez d'autres types d'expressions (Int, Float, Str, Bool, etc.) si nécessaire
            _ => {}
        }
    }

    /// Vérifie si un type est défini.
    fn is_type_defined(&self, type_name: &str) -> bool {
        // Liste des types de base, incluant 'void'
        let predefined_types = vec!["int", "float", "bool", "string", "void"];
        predefined_types.contains(&type_name)
    }

    /// Entre dans un nouveau scope en créant une nouvelle table de symboles.
    fn enter_scope(&mut self) {
        let new_table = SymbolTable::new(Some(Box::new(self.symbol_table.clone())));
        self.symbol_table = new_table;
    }

    /// Sorte du scope actuel en revenant à la table de symboles parente.
    fn exit_scope(&mut self) {
        if let Some(parent) = self.symbol_table.parent.clone() {
            self.symbol_table = *parent;
        } else {
            // Si pas de parent, on reste dans le scope global
        }
    }
}
