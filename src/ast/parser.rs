use crate::ast::models::node::{AST, Statement, VarDecl, Expression};
use crate::lex::models::{token::Token, token_type::TokenType};

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0 }
    }

    /// Parcourt tous les tokens pour construire un AST complet (une liste de statements).
    pub fn parse(&mut self) -> AST {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            // parse_statement() va regarder le prochain token
            // et décider comment parser (varDecl, functionDecl, etc.)
            match self.parse_statement() {
                Some(stmt) => statements.push(stmt),
                None => {
                    // En cas d'erreur de parsing,
                    // on peut soit break, soit avancer, etc.
                    eprintln!("Error: could not parse statement. Stopping.");
                    break;
                }
            }
        }

        AST { statements }
    }

    /// Regarde le token courant et appelle le bon sous-parser
    fn parse_statement(&mut self) -> Option<Statement> {
        // Si le token courant est `let` => parse_var_decl
        if self.is_keyword("let") {
            // On délègue à parse_var_decl
            self.parse_var_decl().map(Statement::VarDecl)
        } else {
            // Sinon, on ne sait pas parser => on avance et on renvoie None
            eprintln!("Parser warning: unexpected token: {:?}", self.peek());
            self.advance();
            None
        }
    }

    /// Sous-parser pour une déclaration de variable
    /// ex: `let x: string = "az";`
    fn parse_var_decl(&mut self) -> Option<VarDecl> {
        // consomme "let"
        self.consume_keyword("let")?;

        // identifiant
        let name_token = self.consume(TokenType::Identifier, "Expected identifier after 'let'")?;
        let name = name_token.value;

        // consomme ":"
        self.consume(TokenType::Colon, "Expected ':' after identifier")?;

        // récupère le type (pour l'exemple, on s'attend à TokenType::Type 
        // ou un identifiant si vous gérez les types comme des identifiants)
        let type_token = self.consume(
            TokenType::Type,
            "Expected a type keyword (e.g. float, string, etc.) after ':'"
        )?;
        let type_name = type_token.value;

        // optionnel : `= expression`
        let mut init = None;
        if self.match_token(TokenType::Equals) {
            self.advance(); // consomme "="
            init = self.parse_expression();
        }

        // consomme ";"
        self.consume(TokenType::Semicolon, "Expected ';' at the end of variable declaration")?;

        Some(VarDecl {
            name,
            type_name,
            init,
        })
    }

    /// Sous-parser pour une expression ultra-simplifiée (ident ou littéral)
    fn parse_expression(&mut self) -> Option<Expression> {
        let token = self.advance();
        match token.token_type {
            TokenType::Identifier => Some(Expression::Ident(token.value)),
            TokenType::Int => {
                if let Ok(val) = token.value.parse::<i64>() {
                    Some(Expression::Int(val))
                } else {
                    eprintln!("Error: cannot parse int from '{}'", token.value);
                    None
                }
            }
            TokenType::Float => {
                if let Ok(val) = token.value.parse::<f64>() {
                    Some(Expression::Float(val))
                } else {
                    eprintln!("Error: cannot parse float from '{}'", token.value);
                    None
                }
            }
            TokenType::String => Some(Expression::Str(token.value)),
            TokenType::Bool => {
                let val = token.value == "true";
                Some(Expression::Bool(val))
            }
            _ => {
                eprintln!("Error: unexpected token in parse_expression: {:?}", token);
                None
            }
        }
    }

    // ------------------------------------------------------------
    //  Méthodes utilitaires (lecture, matching, consommation)
    // ------------------------------------------------------------

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len() 
            || self.tokens[self.position].token_type == TokenType::EOF
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.position].clone();
        self.position += 1;
        token
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.peek().token_type == token_type
        }
    }

    fn match_token(&self, token_type: TokenType) -> bool {
        self.check(token_type)
    }

    fn consume(&mut self, ttype: TokenType, err_msg: &str) -> Option<Token> {
        if self.check(ttype.clone()) {
            Some(self.advance())
        } else {
            eprintln!("Parser error: {}", err_msg);
            None
        }
    }

    // Helpers pour consommer un mot-clé spécifique
    fn is_keyword(&self, kw: &str) -> bool {
        if self.is_at_end() {
            return false;
        }
        let t = self.peek();
        t.token_type == TokenType::Keyword && t.value == kw
    }

    fn consume_keyword(&mut self, keyword: &str) -> Option<Token> {
        if self.is_keyword(keyword) {
            Some(self.advance())
        } else {
            eprintln!("Parser error: Expected keyword '{}'", keyword);
            None
        }
    }
}
