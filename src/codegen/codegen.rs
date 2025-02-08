use crate::codegen::models::asm::{ASM, SectionCode};
use crate::parser::models::ast::AST;
use crate::parser::models::expression::{Expression, FunctionCall};
use crate::parser::models::statement::{
    ForStatement, FunctionDeclaration, IfStatement, Statement, SwitchCase, SwitchStatement,
    VarAffection, VarDeclaration, WhileStatement,
};
use std::collections::HashMap;

/// Le générateur de code produit l'assembleur NASM pour x86 à partir d'un AST.
/// 
/// Ce module parcourt l'AST et génère des instructions d'assembleur en organisant le
/// code en sections (data, bss, text, et sections spécifiques aux fonctions). Il gère également
/// l'allocation des variables locales et des paramètres ainsi que la génération de labels uniques.
pub struct CodeGenerator {
    pub asm: ASM,
    label_counter: usize,
    local_offset: i32,
    in_function: bool,
    local_vars: HashMap<String, i32>,
    string_literals: HashMap<String, String>,
    nb_for_boucle: usize,
    current_loop_var: Option<(String, String)>,
    current_section: SectionCode,
}

impl CodeGenerator {
    /// Crée un nouveau générateur de code avec des valeurs par défaut.
    pub fn new() -> Self {
        CodeGenerator {
            asm: ASM::new(),
            label_counter: 0,
            local_offset: 4, // On démarre à 4 pour la première variable locale.
            in_function: false,
            local_vars: HashMap::new(),
            string_literals: HashMap::new(),
            nb_for_boucle: 0,
            current_loop_var: None,
            current_section: SectionCode::new("".to_string()),
        }
    }

    /// Ajoute une instruction d'assembleur à la section de code en cours.
    ///
    /// # Arguments
    ///
    /// * `code` - Une chaîne de caractères représentant une instruction NASM.
    fn emit(&mut self, code: String) {
        self.current_section.code.push(code);
    }

    /// Génère le code assembleur à partir de l'AST fourni.
    ///
    /// Cette méthode parcourt les déclarations globales et les instructions pour générer
    /// les sections d'assembleur correspondantes (data, bss, text, et sections spécifiques).
    ///
    /// # Arguments
    ///
    /// * `ast` - L'AST (Abstract Syntax Tree) du programme à compiler.
    pub fn generate(&mut self, ast: &AST) {
        // Récupération des déclarations globales (hors fonctions)
        let global_vars: Vec<&VarDeclaration> = ast
            .statements
            .iter()
            .filter_map(|stmt| {
                if let Statement::VarDeclaration(var_decl) = stmt {
                    if !self.in_function {
                        return Some(var_decl);
                    }
                }
                None
            })
            .collect();

        // Génération de la section .data
        self.asm.section_data.push("section .data".to_string());
        self.asm
            .section_data
            .push("    format: db \"%d\", 10, 0".to_string());
        for (literal, label) in &self.string_literals {
            self.asm
                .section_data
                .push(format!("    {}: db \"{}\", 0", label, literal));
        }

        // Génération de la section .bss pour les variables globales
        self.asm.section_bss.push("section .bss".to_string());
        for var in &global_vars {
            self.asm
                .section_bss
                .push(format!("    {} resq 1", var.name));
        }

        // Génération de la section .text et du point d'entrée
        self.asm.section_text.push("section .text".to_string());
        self.asm.section_text.push("global _start".to_string());
        self.asm.section_text.push("extern printf".to_string());
        self.asm.section_text.push("".to_string());
        self.asm.section_text.push("_start:".to_string());
        self.asm.section_text.push("    jmp f_main".to_string());

        // Création de la section principale pour le code d'exécution (f_main)
        self.current_section = SectionCode::new("f_main:".to_string());

        // Initialisation des variables globales ayant une valeur d'initiation
        for var in &global_vars {
            if let Some(init_expr) = &var.init {
                self.generate_expression(init_expr);
                self.emit(format!("    mov [{}], rax", var.name));
            }
        }

        // Génération des autres instructions (hors déclarations globales)
        for stmt in &ast.statements {
            if let Statement::VarDeclaration(_) = stmt {
                continue;
            }
            self.generate_statement(stmt);
        }

        // Code pour terminer le programme (syscall exit)
        self.emit("    mov rax, 60".to_string());
        self.emit("    xor rdi, rdi".to_string());
        self.emit("    syscall".to_string());

        // Ajoute la section principale générée aux sections de code de l'ASM
        self.asm.sections_code.push(std::mem::replace(
            &mut self.current_section,
            SectionCode::new("".to_string()),
        ));
    }

    /// Génère le code pour une instruction (statement) donnée.
    ///
    /// Cette méthode délègue la génération du code à des méthodes spécialisées selon le type de statement.
    ///
    /// # Arguments
    ///
    /// * `stmt` - Une référence à un statement de l'AST.
    pub fn generate_statement(&mut self, stmt: &Statement) {
        use Statement::*;
        match stmt {
            VarDeclaration(var_decl) => {
                if self.in_function {
                    self.generate_local_var_declaration(var_decl);
                }
            }
            VarAffection(var_affection) => self.generate_var_affection(var_affection),
            ExpressionStatement(expr) => self.generate_expression(expr),
            Return(expr_opt) => self.generate_return(expr_opt),
            If(if_stmt) => self.generate_if_statement(if_stmt),
            For(for_stmt) => self.generate_for_statement(for_stmt),
            While(while_stmt) => self.generate_while_statement(while_stmt),
            Switch(switch_stmt) => self.generate_switch_statement(switch_stmt),
            FunctionDeclaration(func_decl) => self.generate_function_declaration(func_decl),
        }
    }

    /// Génère le code pour la déclaration d'une variable locale.
    ///
    /// Cette méthode gère l'initialisation et l'allocation d'un offset négatif pour la variable.
    ///
    /// # Arguments
    ///
    /// * `var_decl` - Une référence à une déclaration de variable.
    fn generate_local_var_declaration(&mut self, var_decl: &VarDeclaration) {
        // Génère le code pour l'initialisation de la variable, ou 0 par défaut.
        if let Some(init_expr) = &var_decl.init {
            self.generate_expression(init_expr);
        } else {
            self.emit("    mov rax, 0".to_string());
        }
        // Affecte un offset négatif si la variable n'est pas déjà définie.
        if !self.local_vars.contains_key(&var_decl.name) {
            self.local_vars
                .insert(var_decl.name.clone(), -self.local_offset);
            self.emit(format!("    mov [rbp - {}], rax", self.local_offset));
            self.local_offset += 4; // On suppose des int sur 4 octets.
        } else {
            let off = self.local_vars[&var_decl.name];
            if off < 0 {
                self.emit(format!("    mov [rbp - {}], rax", -off));
            } else {
                self.emit(format!("    mov [rbp + {}], rax", off));
            }
        }
    }

    /// Génère le code pour l'affectation d'une variable (locale ou globale).
    ///
    /// # Arguments
    ///
    /// * `var_affection` - Une référence à une affectation de variable.
    fn generate_var_affection(&mut self, var_affection: &VarAffection) {
        self.generate_expression(&var_affection.value);
        // Si la variable correspond à celle d'une boucle for, utiliser le nom interne
        if let Some((user_var, internal_var)) = &self.current_loop_var {
            if &var_affection.name == user_var {
                self.emit(format!("    mov [{}], rax", internal_var));
                return;
            }
        }
        // Traitement standard pour une variable locale ou globale
        if self.in_function {
            if let Some(offset) = self.local_vars.get(&var_affection.name) {
                if *offset < 0 {
                    self.emit(format!("    mov [rbp - {}], rax", -offset));
                } else {
                    self.emit(format!("    mov [rbp + {}], rax", offset));
                }
            } else {
                self.emit(format!("    mov [{}], rax", var_affection.name));
            }
        } else {
            self.emit(format!("    mov [{}], rax", var_affection.name));
        }
    }

    /// Génère le code pour une instruction de retour.
    ///
    /// Si une expression est fournie, elle est évaluée avant d'exécuter l'épilogue de fonction.
    ///
    /// # Arguments
    ///
    /// * `expr_opt` - Option contenant l'expression à retourner.
    fn generate_return(&mut self, expr_opt: &Option<Expression>) {
        if let Some(expr) = expr_opt {
            self.generate_expression(expr);
        }
        self.emit("    mov rsp, rbp".to_string());
        self.emit("    pop rbp".to_string());
        self.emit("    ret".to_string());
    }

    /// Génère le code pour une instruction if-else.
    ///
    /// Cette méthode émet le code pour tester la condition, exécuter la branche then ou else selon le cas,
    /// et utilise des labels pour délimiter les blocs.
    ///
    /// # Arguments
    ///
    /// * `if_stmt` - Une référence à une instruction if.
    fn generate_if_statement(&mut self, if_stmt: &IfStatement) {
        self.generate_expression(&if_stmt.condition);
        let else_label = self.new_label();
        let end_label = self.new_label();
        self.emit("    cmp rax, 0".to_string());
        self.emit(format!("    je {}", else_label));
        for stmt in &if_stmt.then_branch {
            self.generate_statement(stmt);
        }
        self.emit(format!("    jmp {}", end_label));
        self.emit(format!("{}:", else_label));
        if let Some(else_branch) = &if_stmt.else_branch {
            for stmt in else_branch {
                self.generate_statement(stmt);
            }
        }
        self.emit(format!("{}:", end_label));
    }

    /// Génère le code pour une boucle for.
    ///
    /// La méthode gère l'initialisation, la condition, le corps et l'incrémentation.
    /// Elle génère un nom interne pour la variable de boucle (par exemple "i1").
    ///
    /// # Arguments
    ///
    /// * `for_stmt` - Une référence à une instruction for.
    fn generate_for_statement(&mut self, for_stmt: &ForStatement) {
        if let Statement::VarDeclaration(var_decl) = &*for_stmt.init {
            let user_var = var_decl.name.clone();
            self.nb_for_boucle += 1;
            // Nom interne sans underscore (exemple "i1")
            let internal_var = format!("{}_{}", user_var, self.nb_for_boucle);
            self.current_loop_var = Some((user_var.clone(), internal_var.clone()));
            if let Some(init_expr) = &var_decl.init {
                self.generate_expression(init_expr);
            } else {
                self.emit("    mov rax, 0".to_string());
            }
            self.emit(format!("    mov [{}], rax", internal_var));
            self.asm
                .section_bss
                .push(format!("    {} resq 1", internal_var));
        } else {
            self.generate_statement(&for_stmt.init);
        }

        let start_label = self.new_label();
        self.emit(format!("{}:", start_label));

        if let Statement::ExpressionStatement(cond_expr) = &*for_stmt.cond {
            self.generate_expression(cond_expr);
            let exit_label = self.new_label();
            self.emit("    cmp rax, 0".to_string());
            self.emit(format!("    je {}", exit_label));
            for stmt in &for_stmt.body {
                self.generate_statement(stmt);
            }
            self.generate_statement(&for_stmt.incr);
            self.emit(format!("    jmp {}", start_label));
            self.emit(format!("{}:", exit_label));
        } else {
            self.emit("    ; For-loop condition must be an expression statement".to_string());
        }

        self.current_loop_var = None;
    }

    /// Génère le code pour une boucle while.
    ///
    /// # Arguments
    ///
    /// * `while_stmt` - Une référence à une instruction while.
    fn generate_while_statement(&mut self, while_stmt: &WhileStatement) {
        let start_label = self.new_label();
        self.emit(format!("{}:", start_label));
        self.generate_expression(&while_stmt.condition);
        let exit_label = self.new_label();
        self.emit("    cmp rax, 0".to_string());
        self.emit(format!("    je {}", exit_label));
        for stmt in &while_stmt.body {
            self.generate_statement(stmt);
        }
        self.emit(format!("    jmp {}", start_label));
        self.emit(format!("{}:", exit_label));
    }

    /// Génère le code pour une instruction switch.
    ///
    /// # Arguments
    ///
    /// * `switch_stmt` - Une référence à une instruction switch.
    fn generate_switch_statement(&mut self, switch_stmt: &SwitchStatement) {
        self.generate_expression(&switch_stmt.condition);
        let end_label = self.new_label();
        for case in &switch_stmt.cases {
            let case_label = self.new_label();
            self.generate_expression(&case.value);
            self.emit("    mov rbx, rax".to_string());
            self.generate_expression(&switch_stmt.condition);
            self.emit("    cmp rax, rbx".to_string());
            self.emit(format!("    jne {}", case_label));
            for stmt in &case.body {
                self.generate_statement(stmt);
            }
            self.emit(format!("    jmp {}", end_label));
            self.emit(format!("{}:", case_label));
        }
        if let Some(default_body) = &switch_stmt.default {
            for stmt in default_body {
                self.generate_statement(stmt);
            }
        }
        self.emit(format!("{}:", end_label));
    }

    /// Génère la définition d'une fonction.
    ///
    /// La méthode produit le prologue (avec allocation de pile), l'insertion des paramètres,
    /// la génération du corps de la fonction et l'épilogue.
    ///
    /// # Arguments
    ///
    /// * `func_decl` - Une référence à la déclaration de la fonction.
    fn generate_function_declaration(&mut self, func_decl: &FunctionDeclaration) {
        // Sauvegarde de la section courante
        let saved_section = std::mem::replace(
            &mut self.current_section,
            SectionCode::new(format!("f_{}:", func_decl.name)),
        );

        // Prologue de fonction
        self.emit("    push rbp".to_string());
        self.emit("    mov rbp, rsp".to_string());
        // Allocation fixe de 16 octets pour les variables locales
        self.emit("    sub rsp, 16".to_string());

        // Insertion des paramètres dans la table des variables.
        // Correction : le premier paramètre est à [rbp+16] (puisque [rbp+8] contient l'adresse de retour)
        let mut param_offset = 16;
        for param in &func_decl.parameters {
            self.local_vars.insert(param.name.clone(), param_offset);
            param_offset += 4; // on suppose des int sur 4 octets
        }

        self.in_function = true;
        // Pour les variables locales, on démarre à 4 (les premiers locaux seront à [rbp - 4], [rbp - 8], etc.)
        self.local_offset = 4;

        // Génération du corps de la fonction
        for stmt in &func_decl.body {
            self.generate_statement(stmt);
        }

        // Épilogue de fonction
        self.emit("    mov rsp, rbp".to_string());
        self.emit("    pop rbp".to_string());
        self.emit("    ret".to_string());
        self.in_function = false;
        self.local_vars.clear();

        // Remet la section précédente et ajoute la fonction générée aux sections
        let function_section = std::mem::replace(&mut self.current_section, saved_section);
        self.asm.sections_code.push(function_section);
    }

    /// Génère le code pour une expression.
    ///
    /// Cette méthode traite les littéraux, les identifiants, les opérations binaires, 
    /// ainsi que les appels de fonctions (notamment "print" et les appels génériques).
    ///
    /// # Arguments
    ///
    /// * `expr` - Une référence à une expression.
    fn generate_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Str(s) => {
                let label = self.get_or_create_string_literal(s);
                self.emit(format!("    lea rax, [rel {}]", label));
            }
            Expression::Int(val) => self.emit(format!("    mov rax, {}", val)),
            Expression::Float(val) => self.emit(format!("    mov rax, {}", val)),
            Expression::Bool(val) => {
                self.emit(format!("    mov rax, {}", if *val { 1 } else { 0 }))
            }
            Expression::Ident(name) => {
                // Si la variable correspond à celle d'une boucle for, on utilise le nom interne
                if let Some((user_var, internal_var)) = &self.current_loop_var {
                    if name == user_var {
                        self.emit(format!("    mov rax, [{}]", internal_var));
                        return;
                    }
                }
                // Sinon, récupère l'offset dans les variables locales ou accède à la variable globale
                if let Some(off) = self.local_vars.get(name) {
                    if *off >= 0 {
                        self.emit(format!("    mov rax, [rbp + {}]", off));
                    } else {
                        self.emit(format!("    mov rax, [rbp - {}]", -off));
                    }
                } else {
                    self.emit(format!("    mov rax, [{}]", name));
                }
            }
            Expression::Binary(bin_expr) => {
                self.generate_expression(&bin_expr.left);
                self.emit("    push rax".to_string());
                self.generate_expression(&bin_expr.right);
                self.emit("    pop rbx".to_string());
                self.emit("    xchg rax, rbx".to_string());
                match bin_expr.op.as_str() {
                    "+" => self.emit("    add rax, rbx".to_string()),
                    "-" => self.emit("    sub rax, rbx".to_string()),
                    "*" => self.emit("    imul rax, rbx".to_string()),
                    "/" => {
                        self.emit("    cqo".to_string());
                        self.emit("    idiv rbx".to_string());
                    }
                    "==" => {
                        self.emit("    cmp rax, rbx".to_string());
                        self.emit("    sete al".to_string());
                        self.emit("    movzx rax, al".to_string());
                    }
                    "!=" => {
                        self.emit("    cmp rax, rbx".to_string());
                        self.emit("    setne al".to_string());
                        self.emit("    movzx rax, al".to_string());
                    }
                    "<" => {
                        self.emit("    cmp rax, rbx".to_string());
                        self.emit("    setl al".to_string());
                        self.emit("    movzx rax, al".to_string());
                    }
                    "<=" => {
                        self.emit("    cmp rax, rbx".to_string());
                        self.emit("    setle al".to_string());
                        self.emit("    movzx rax, al".to_string());
                    }
                    ">" => {
                        self.emit("    cmp rax, rbx".to_string());
                        self.emit("    setg al".to_string());
                        self.emit("    movzx rax, al".to_string());
                    }
                    ">=" => {
                        self.emit("    cmp rax, rbx".to_string());
                        self.emit("    setge al".to_string());
                        self.emit("    movzx rax, al".to_string());
                    }
                    "&&" => {
                        self.emit("    and rax, rbx".to_string());
                    }
                    "||" => {
                        self.emit("    or rax, rbx".to_string());
                    }
                    "%" => {
                        self.emit("    cqo".to_string());
                        self.emit("    idiv rbx".to_string());
                        self.emit("    mov rax, rdx".to_string());
                    }
                    _ => self.emit("    ; Unsupported binary operator".to_string()),
                }
            }
            Expression::FunctionCall(call) if call.name == "print" => {
                if call.arguments.len() == 1 {
                    self.generate_expression(&call.arguments[0]);
                    match &call.arguments[0] {
                        Expression::Str(_) => self.emit("    lea rdi, [rel format]".to_string()),
                        Expression::Int(_) => self.emit("    lea rdi, [rel format]".to_string()),
                        Expression::Float(_) => self.emit("    lea rdi, [rel format]".to_string()),
                        _ => self.emit("    lea rdi, [rel format]".to_string()),
                    }
                    self.emit("    mov rsi, rax".to_string());
                    self.emit("    xor rax, rax".to_string());
                    self.emit("    call printf".to_string());
                }
            }
            Expression::FunctionCall(call) => {
                // Traitement générique des appels de fonction autres que print.
                // Pour chaque argument, on génère son code et on le pousse sur la pile.
                for arg in &call.arguments {
                    self.generate_expression(arg);
                    self.emit("    push rax".to_string());
                }
                // Appel de la fonction. On suppose que le label de la fonction est préfixé par "f_"
                self.emit(format!("    call f_{}", call.name));
                // Nettoyage de la pile (8 octets par argument)
                if !call.arguments.is_empty() {
                    self.emit(format!("    add rsp, {}", 8 * call.arguments.len()));
                }
            }
            _ => {
                // Autres cas d'expressions…
            }
        }
    }

    /// Récupère ou crée un label pour un littéral de chaîne.
    ///
    /// Si la chaîne existe déjà, retourne le label associé, sinon en crée un nouveau.
    ///
    /// # Arguments
    ///
    /// * `s` - La chaîne littérale.
    fn get_or_create_string_literal(&mut self, s: &str) -> String {
        if let Some(label) = self.string_literals.get(s) {
            return label.clone();
        }
        let label = format!("str_{}", self.string_literals.len());
        self.string_literals.insert(s.to_string(), label.clone());
        label
    }

    /// Génère un nouveau label unique pour le code assembleur.
    fn new_label(&mut self) -> String {
        let label = format!("L{}", self.label_counter);
        self.label_counter += 1;
        label
    }
}
