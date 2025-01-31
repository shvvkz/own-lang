use std::collections::HashMap;

/// Représente un symbole dans la table des symboles.
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub symbol_type: SymbolType,
}

/// Types de symboles possibles.
#[derive(Debug, Clone)]
pub enum SymbolType {
    Variable(String),
    Function {
        parameters: Vec<String>,
        return_type: String,
    },
}

/// Représente une table de symboles avec un environnement parent pour gérer les scopes.
#[derive(Debug, Clone)]
pub struct SymbolTable {
    pub symbols: HashMap<String, Symbol>,
    pub parent: Option<Box<SymbolTable>>,
}

impl SymbolTable {
    /// Crée une nouvelle table de symboles.
    pub fn new(parent: Option<Box<SymbolTable>>) -> Self {
        SymbolTable {
            symbols: HashMap::new(),
            parent,
        }
    }

    /// Définit un nouveau symbole dans la table courante.
    pub fn define(&mut self, name: String, symbol: Symbol) -> Result<(), String> {
        if self.symbols.contains_key(&name) {
            return Err(format!("Symbol '{}' already defined in the current scope.", name));
        }
        self.symbols.insert(name, symbol);
        Ok(())
    }

    /// Résout un symbole en cherchant dans la table courante et les tables parentes.
    pub fn resolve(&self, name: &str) -> Option<&Symbol> {
        if let Some(symbol) = self.symbols.get(name) {
            return Some(symbol);
        }
        if let Some(ref parent) = self.parent {
            return parent.resolve(name);
        }
        None
    }
}
