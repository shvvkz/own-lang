use crate::parser::models::ast::AST;
use crate::parser::models::expression::{BinaryExpression, Expression, FunctionCall};
use crate::parser::models::statement::{
    ForStatement, FunctionDeclaration, IfStatement, Statement, SwitchCase, SwitchStatement,
    VarAffection, VarDeclaration, WhileStatement,
};
use std::collections::HashMap;

/// CodeGenerator génère du véritable assembleur NASM pour x86_64 (syntaxe Intel).
/// Il gère les variables globales (dans .data) et locales (via un stack frame),
/// et traite la fonction built‑in `print` en adaptant le format printf selon le type.
pub struct CodeGenerator {
    pub asm: Vec<String>,
    label_counter: usize,
    // Pour la gestion des variables locales dans les fonctions :
    local_offset: usize,                // Offsets sur la pile (en octets)
    in_function: bool,                  // Indique si on est dans une fonction
    local_vars: HashMap<String, usize>, // Mapping local: variable -> offset
    // Pour connaître le type des variables globales
    pub global_types: HashMap<String, String>, // Mapping global: variable -> type (ex: "string")
}

impl CodeGenerator {
    /// Crée un nouveau générateur.
    pub fn new() -> Self {
        CodeGenerator {
            asm: Vec::new(),
            label_counter: 0,
            local_offset: 0,
            in_function: false,
            local_vars: HashMap::new(),
            global_types: HashMap::new(),
        }
    }

    /// Génère le code assembleur complet à partir de l'AST.
    /// Les déclarations globales (en dehors de fonctions) seront placées dans la section .data.
    pub fn generate(&mut self, ast: &AST) {
        // --- Section .data : variables globales et formats pour print ---
        self.asm.push("section .data".to_string());
        self.asm.push("print_format_str: db \"%s\\n\", 0".to_string());
        self.asm.push("print_format_int: db \"%d\\n\", 0".to_string());
        self.asm.push("print_format_float: db \"%f\\n\", 0".to_string());
        // Pour chaque déclaration globale, générer la donnée.
        for stmt in &ast.statements {
            if let Statement::VarDeclaration(var_decl) = stmt {
                self.generate_global_var(&var_decl);
            }
        }
        self.asm.push("".to_string());

        // --- Section .text ---
        self.asm.push("section .text".to_string());
        self.asm.push("extern printf".to_string());
        self.asm.push("extern exit".to_string());
        self.asm.push("global _start".to_string());
        self.asm.push("_start:".to_string());

        // Générer le code pour les autres statements (hors globales).
        for stmt in &ast.statements {
            match stmt {
                Statement::VarDeclaration(_) => { /* déjà en .data */ }
                _ => self.generate_statement(&stmt),
            }
        }

        // Fin du programme : appel à exit (pour que libc flush stdout).
        self.asm.push("mov rdi, 0".to_string());
        self.asm.push("call exit".to_string());
    }

    /// Génère le code pour un statement.
    pub fn generate_statement(&mut self, stmt: &Statement) {
        use Statement::*;
        match stmt {
            VarDeclaration(var_decl) => {
                if self.in_function {
                    self.generate_local_var_declaration(var_decl);
                } else {
                    self.generate_global_var(var_decl);
                }
            }
            VarAffection(var_affection) => self.generate_var_affection(var_affection),
            ExpressionStatement(expr) => {
                self.generate_expression(expr);
            }
            Return(expr_opt) => self.generate_return(expr_opt),
            If(if_stmt) => self.generate_if_statement(if_stmt),
            For(for_stmt) => self.generate_for_statement(for_stmt),
            While(while_stmt) => self.generate_while_statement(while_stmt),
            Switch(switch_stmt) => self.generate_switch_statement(switch_stmt),
            FunctionDeclaration(func_decl) => self.generate_function_declaration(func_decl),
        }
    }

    /// Génère une variable globale dans la section .data.
    fn generate_global_var(&mut self, var_decl: &VarDeclaration) {
        let label = format!("_var_{}", var_decl.name);
        // Enregistrer le type dans global_types
        self.global_types
            .insert(var_decl.name.clone(), var_decl.type_name.clone());
        let init_val = if let Some(expr) = &var_decl.init {
            match expr {
                Expression::Int(val) => val.to_string(),
                Expression::Float(val) => val.to_string(),
                Expression::Bool(b) => if *b { "1".to_string() } else { "0".to_string() },
                Expression::Str(s) => format!("\"{}\"", s),
                _ => "0".to_string(),
            }
        } else {
            if var_decl.type_name == "string" {
                "\"\"".to_string()
            } else {
                "0".to_string()
            }
        };

        if var_decl.type_name == "string" {
            self.asm.push(format!("{}: db {}, 0", label, init_val));
        } else {
            self.asm.push(format!("{}: dq {}", label, init_val));
        }
    }

    /// Génère une variable locale dans une fonction.
    fn generate_local_var_declaration(&mut self, var_decl: &VarDeclaration) {
        if let Some(init_expr) = &var_decl.init {
            self.generate_expression(init_expr);
        } else {
            self.asm.push("mov rax, 0".to_string());
        }
        self.local_vars.insert(var_decl.name.clone(), self.local_offset);
        self.asm.push(format!("mov [rbp - {}], rax", self.local_offset));
        self.asm.push(format!("; Local variable {} at [rbp - {}]", var_decl.name, self.local_offset));
        self.local_offset += 8;
    }

    /// Génère le code pour une affection de variable (assignment).
    fn generate_var_affection(&mut self, var_affection: &VarAffection) {
        self.generate_expression(&var_affection.value);
        if self.in_function {
            if let Some(offset) = self.local_vars.get(&var_affection.name) {
                self.asm.push(format!("mov [rbp - {}], rax", offset));
            } else {
                self.asm.push(format!("mov rax, [rel _var_{}]", var_affection.name));
                self.asm.push(format!("mov [rel _var_{}], rax", var_affection.name));
            }
        } else {
            self.asm.push(format!("mov rax, [rel _var_{}]", var_affection.name));
            self.asm.push(format!("mov [rel _var_{}], rax", var_affection.name));
        }
    }

    /// Génère le code pour une instruction return.
    fn generate_return(&mut self, expr_opt: &Option<Expression>) {
        if let Some(expr) = expr_opt {
            self.generate_expression(expr);
        }
        self.asm.push("jmp _exit".to_string());
    }

    /// Génère le code pour une instruction if.
    fn generate_if_statement(&mut self, if_stmt: &IfStatement) {
        self.generate_expression(&if_stmt.condition);
        let else_label = self.new_label();
        let end_label = self.new_label();
        self.asm.push("cmp rax, 0".to_string());
        self.asm.push(format!("je {}", else_label));
        for stmt in &if_stmt.then_branch {
            self.generate_statement(stmt);
        }
        self.asm.push(format!("jmp {}", end_label));
        self.asm.push(format!("{}:", else_label));
        if let Some(else_branch) = &if_stmt.else_branch {
            for stmt in else_branch {
                self.generate_statement(stmt);
            }
        }
        self.asm.push(format!("{}:", end_label));
    }

    /// Génère le code pour une boucle for.
    /// Syntaxe attendue : for (init; cond; incr;) { body }
    fn generate_for_statement(&mut self, for_stmt: &ForStatement) {
        self.generate_statement(&for_stmt.init);
        let start_label = self.new_label();
        self.asm.push(format!("{}:", start_label));
        if let Statement::ExpressionStatement(cond_expr) = &*for_stmt.cond {
            self.generate_expression(cond_expr);
            let exit_label = self.new_label();
            self.asm.push("cmp rax, 0".to_string());
            self.asm.push(format!("je {}", exit_label));
            for stmt in &for_stmt.body {
                self.generate_statement(stmt);
            }
            self.generate_statement(&for_stmt.incr);
            self.asm.push(format!("jmp {}", start_label));
            self.asm.push(format!("{}:", exit_label));
        } else {
            self.asm.push("; For-loop condition must be an expression statement".to_string());
        }
    }

    /// Génère le code pour une boucle while.
    fn generate_while_statement(&mut self, while_stmt: &WhileStatement) {
        let start_label = self.new_label();
        self.asm.push(format!("{}:", start_label));
        self.generate_expression(&while_stmt.condition);
        let exit_label = self.new_label();
        self.asm.push("cmp rax, 0".to_string());
        self.asm.push(format!("je {}", exit_label));
        for stmt in &while_stmt.body {
            self.generate_statement(stmt);
        }
        self.asm.push(format!("jmp {}", start_label));
        self.asm.push(format!("{}:", exit_label));
    }

    /// Génère le code pour une instruction switch.
    fn generate_switch_statement(&mut self, switch_stmt: &SwitchStatement) {
        self.generate_expression(&switch_stmt.condition);
        let end_label = self.new_label();
        for case in &switch_stmt.cases {
            let case_label = self.new_label();
            self.generate_expression(&case.value);
            self.asm.push("mov rbx, rax".to_string());
            self.generate_expression(&switch_stmt.condition);
            self.asm.push("cmp rax, rbx".to_string());
            self.asm.push(format!("jne {}", case_label));
            for stmt in &case.body {
                self.generate_statement(stmt);
            }
            self.asm.push(format!("jmp {}", end_label));
            self.asm.push(format!("{}:", case_label));
        }
        if let Some(default_body) = &switch_stmt.default {
            for stmt in default_body {
                self.generate_statement(stmt);
            }
        }
        self.asm.push(format!("{}:", end_label));
    }

    /// Génère le code pour une déclaration de fonction.
    fn generate_function_declaration(&mut self, func_decl: &FunctionDeclaration) {
        self.asm.push(format!("{}:", func_decl.name));
        self.asm.push("push rbp".to_string());
        self.asm.push("mov rbp, rsp".to_string());
        self.in_function = true;
        self.local_offset = 8;
        self.local_vars.clear();
        for stmt in &func_decl.body {
            self.generate_statement(stmt);
        }
        self.asm.push("mov rsp, rbp".to_string());
        self.asm.push("pop rbp".to_string());
        self.asm.push("ret".to_string());
        self.in_function = false;
    }

    /// Génère le code pour une expression.
    pub fn generate_expression(&mut self, expr: &Expression) {
        use Expression::*;
        match expr {
            Int(val) => {
                self.asm.push(format!("mov rax, {}", val));
            }
            Float(val) => {
                self.asm.push(format!("mov rax, {}", val));
            }
            Bool(val) => {
                self.asm.push(format!("mov rax, {}", if *val { "1" } else { "0" }));
            }
            Expression::Str(s) => {
                let label = self.new_label();
                self.asm.push(format!("{}: db \"{}\", 0", label, s));
                self.asm.push(format!("lea rax, [{}]", label));
            }
            Ident(name) => {
                if self.in_function {
                    if let Some(offset) = self.local_vars.get(name) {
                        self.asm.push(format!("mov rax, [rbp - {}]", offset));
                    } else {
                        if let Some(global_type) = self.global_types.get(name) {
                            if global_type == "string" {
                                self.asm.push(format!("lea rax, [rel _var_{}]", name));
                            } else {
                                self.asm.push(format!("mov rax, [rel _var_{}]", name));
                            }
                        } else {
                            self.asm.push(format!("mov rax, [rel _var_{}]", name));
                        }
                    }
                } else {
                    if let Some(global_type) = self.global_types.get(name) {
                        if global_type == "string" {
                            self.asm.push(format!("lea rax, [rel _var_{}]", name));
                        } else {
                            self.asm.push(format!("mov rax, [rel _var_{}]", name));
                        }
                    } else {
                        self.asm.push(format!("mov rax, [rel _var_{}]", name));
                    }
                }
            }
            Binary(bin_expr) => {
                self.generate_expression(&bin_expr.left);
                self.asm.push("push rax".to_string());
                self.generate_expression(&bin_expr.right);
                self.asm.push("pop rbx".to_string());
                match bin_expr.op.as_str() {
                    "+" => self.asm.push("add rax, rbx".to_string()),
                    "-" => self.asm.push("sub rax, rbx".to_string()),
                    "*" => self.asm.push("imul rax, rbx".to_string()),
                    "/" => {
                        self.asm.push("mov rdx, 0".to_string());
                        self.asm.push("idiv rbx".to_string());
                    }
                    "==" => {
                        self.asm.push("cmp rax, rbx".to_string());
                        self.asm.push("sete al".to_string());
                        self.asm.push("movzx rax, al".to_string());
                    }
                    "!=" => {
                        self.asm.push("cmp rax, rbx".to_string());
                        self.asm.push("setne al".to_string());
                        self.asm.push("movzx rax, al".to_string());
                    }
                    "<" => {
                        self.asm.push("cmp rax, rbx".to_string());
                        self.asm.push("setl al".to_string());
                        self.asm.push("movzx rax, al".to_string());
                    }
                    "<=" => {
                        self.asm.push("cmp rax, rbx".to_string());
                        self.asm.push("setle al".to_string());
                        self.asm.push("movzx rax, al".to_string());
                    }
                    ">" => {
                        self.asm.push("cmp rax, rbx".to_string());
                        self.asm.push("setg al".to_string());
                        self.asm.push("movzx rax, al".to_string());
                    }
                    ">=" => {
                        self.asm.push("cmp rax, rbx".to_string());
                        self.asm.push("setge al".to_string());
                        self.asm.push("movzx rax, al".to_string());
                    }
                    _ => {
                        self.asm.push("; Unsupported binary operator".to_string());
                    }
                }
            }
            FunctionCall(call) => {
                if call.name == "print" {
                    if call.arguments.len() != 1 {
                        self.asm.push("; Error: print expects 1 argument".to_string());
                    } else {
                        self.generate_expression(&call.arguments[0]);
                        // Choisir le format en fonction du type de l'argument
                        match &call.arguments[0] {
                            Expression::Str(_) => {
                                self.asm.push("lea rdi, [rel print_format_str]".to_string());
                            },
                            Expression::Int(_) => {
                                self.asm.push("lea rdi, [rel print_format_int]".to_string());
                            },
                            Expression::Float(_) => {
                                self.asm.push("lea rdi, [rel print_format_float]".to_string());
                            },
                            Expression::Bool(_) => {
                                self.asm.push("lea rdi, [rel print_format_int]".to_string());
                            },
                            Expression::Ident(name) => {
                                if let Some(ty) = self.global_types.get(name) {
                                    if ty == "string" {
                                        self.asm.push("lea rdi, [rel print_format_str]".to_string());
                                    } else if ty == "float" {
                                        self.asm.push("lea rdi, [rel print_format_float]".to_string());
                                    } else {
                                        self.asm.push("lea rdi, [rel print_format_int]".to_string());
                                    }
                                } else {
                                    self.asm.push("lea rdi, [rel print_format_int]".to_string());
                                }
                            },
                            _ => {
                                self.asm.push("lea rdi, [rel print_format_int]".to_string());
                            }
                        }
                        self.asm.push("mov rsi, rax".to_string());
                        self.asm.push("xor rax, rax".to_string());
                        self.asm.push("call printf".to_string());
                    }
                } else {
                    for arg in &call.arguments {
                        self.generate_expression(arg);
                        self.asm.push("push rax".to_string());
                    }
                    self.asm.push(format!("call {}", call.name));
                    if !call.arguments.is_empty() {
                        self.asm.push(format!("add rsp, {}", call.arguments.len() * 8));
                    }
                }
            }
        }
    }

    /// Crée un nouveau label unique.
    fn new_label(&mut self) -> String {
        let label = format!("L{}", self.label_counter);
        self.label_counter += 1;
        label
    }
}
