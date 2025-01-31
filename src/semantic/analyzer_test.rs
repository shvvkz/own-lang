#[cfg(test)]
mod analyzer_tests {
    use crate::lex::models::token_type::TokenType;
    use crate::parser::parser::Parser;
    use crate::semantic::analyzer::SemanticAnalyzer;
    use crate::lex::lexer::Lexer;
    use std::fs;
    use std::path::Path;

    // Helper function to read a source file and return its content as a String
    fn read_source_file(filename: &str) -> String {
        let path = Path::new("own_files").join(filename);
        fs::read_to_string(path).expect(&format!("Failed to read file {}", filename))
    }

    // Helper function to perform semantic analysis on source code
    fn analyze_source(source: &str) -> SemanticAnalyzer {
        let mut lexer = Lexer::new(source.to_string());
        let mut tokens = Vec::new();
    
        loop {
            let token = lexer.next_token();
            if token.token_type == TokenType::EOF {
                break;
            }
            tokens.push(token);
        }
        // Parsing
        let mut parser = Parser::new(tokens);
        let ast = parser.parse_file();
        // Semantic Analysis
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&ast);
        analyzer
    }

    #[test]
    fn test_var_declaration() {
        // Bon contenu
        let good_source = read_source_file("var_decl/var_decl_good.own");
        let good_analyzer = analyze_source(&good_source);
        assert!(
            good_analyzer.errors.is_empty(),
            "Aucune erreur sémantique ne devrait être présente dans var_decl_good.own"
        );

        // Mauvais contenu
        let bad_source = read_source_file("var_decl/var_decl_bad.own");
        let bad_analyzer = analyze_source(&bad_source);
        assert!(
            !bad_analyzer.errors.is_empty(),
            "Des erreurs sémantiques devraient être présentes dans var_decl_bad.own"
        );
        assert_eq!(
            bad_analyzer.errors.len(),
            1,
            "Une seule erreur devrait être détectée dans var_decl_bad.own"
        );
        assert_eq!(
            bad_analyzer.errors[0],
            "Type mismatch in variable declaration 'x': expected 'string', found 'int'.",
            "Erreur de type incorrecte détectée dans var_decl_bad.own"
        );
    }

    #[test]
    fn test_return_statement() {
        // Bon contenu
        let good_source = read_source_file("return/return_good.own");
        let good_analyzer = analyze_source(&good_source);
        assert!(
            good_analyzer.errors.is_empty(),
            "Aucune erreur sémantique ne devrait être présente dans return_good.own"
        );

        // Mauvais contenu
        let bad_source = read_source_file("return/return_bad.own");
        let bad_analyzer = analyze_source(&bad_source);
        assert!(
            !bad_analyzer.errors.is_empty(),
            "Des erreurs sémantiques devraient être présentes dans return_bad.own"
        );
        assert_eq!(
            bad_analyzer.errors.len(),
            1,
            "Une seule erreur devrait être détectée dans return_bad.own"
        );
        assert_eq!(
            bad_analyzer.errors[0],
            "Type mismatch in return statement: expected 'int', found 'string'.",
            "Erreur de type incorrecte détectée dans return_bad.own"
        );
    }

    #[test]
    fn test_var_affection() {
        // Bon contenu
        let good_source = read_source_file("var_affection/var_affection_good.own");
        let good_analyzer = analyze_source(&good_source);
        assert!(
            good_analyzer.errors.is_empty(),
            "Aucune erreur sémantique ne devrait être présente dans var_affection_good.own"
        );

        // Mauvais contenu
        let bad_source = read_source_file("var_affection/var_affection_bad.own");
        let bad_analyzer = analyze_source(&bad_source);
        assert!(
            !bad_analyzer.errors.is_empty(),
            "Des erreurs sémantiques devraient être présentes dans var_affection_bad.own"
        );
        assert_eq!(
            bad_analyzer.errors.len(),
            1,
            "Une seule erreur devrait être détectée dans var_affection_bad.own"
        );
        assert_eq!(
            bad_analyzer.errors[0],
            "Type mismatch in assignment to 'x': expected 'int', found 'string'.",
            "Erreur de type incorrecte détectée dans var_affection_bad.own"
        );
    }

    #[test]
    fn test_if_statement() {
        // Bon contenu
        let good_source = read_source_file("if/if_good.own");
        let good_analyzer = analyze_source(&good_source);
        assert!(
            good_analyzer.errors.is_empty(),
            "Aucune erreur sémantique ne devrait être présente dans if_good.own"
        );

        // Mauvais contenu
        let bad_source = read_source_file("if/if_bad.own");
        let bad_analyzer = analyze_source(&bad_source);
        assert!(
            !bad_analyzer.errors.is_empty(),
            "Des erreurs sémantiques devraient être présentes dans if_bad.own"
        );
        assert_eq!(
            bad_analyzer.errors.len(),
            1,
            "Une seule erreur devrait être détectée dans if_bad.own"
        );
        assert_eq!(
            bad_analyzer.errors[0],
            "Condition in 'if' statement must be of type 'bool', found 'int'.",
            "Erreur de type incorrecte détectée dans if_bad.own"
        );
    }

    #[test]
    fn test_for_statement() {
        // Bon contenu
        let good_source = read_source_file("for/for_good.own");
        let good_analyzer = analyze_source(&good_source);
        assert!(
            good_analyzer.errors.is_empty(),
            "Aucune erreur sémantique ne devrait être présente dans for_good.own"
        );

        // Mauvais contenu
        let bad_source = read_source_file("for/for_bad.own");
        let bad_analyzer = analyze_source(&bad_source);
        assert!(
            !bad_analyzer.errors.is_empty(),
            "Des erreurs sémantiques devraient être présentes dans for_bad.own"
        );
        assert_eq!(
            bad_analyzer.errors.len(),
            2,
            "Deux erreurs devraient être détectées dans for_bad.own"
        );
        assert!(bad_analyzer.errors.contains(&"Condition in 'for' statement must be of type 'bool', found 'int'.".to_string()));
        assert!(bad_analyzer.errors.contains(&"Type mismatch in assignment to 'i': expected 'int', found 'string'.".to_string()));
    }

    #[test]
    fn test_while_statement() {
        // Bon contenu
        let good_source = read_source_file("while/while_good.own");
        let good_analyzer = analyze_source(&good_source);
        assert!(
            good_analyzer.errors.is_empty(),
            "Aucune erreur sémantique ne devrait être présente dans while_good.own"
        );

        // Mauvais contenu
        let bad_source = read_source_file("while/while_bad.own");
        let bad_analyzer = analyze_source(&bad_source);
        assert!(
            !bad_analyzer.errors.is_empty(),
            "Des erreurs sémantiques devraient être présentes dans while_bad.own"
        );
        assert_eq!(
            bad_analyzer.errors.len(),
            1,
            "Une seule erreur devrait être détectée dans while_bad.own"
        );
        assert_eq!(
            bad_analyzer.errors[0],
            "Condition in 'while' statement must be of type 'bool', found 'string'.",
            "Erreur de type incorrecte détectée dans while_bad.own"
        );
    }

    #[test]
    fn test_switch_statement() {
        // Bon contenu
        let good_source = read_source_file("switch/switch_good.own");
        let good_analyzer = analyze_source(&good_source);
        assert!(
            good_analyzer.errors.is_empty(),
            "Aucune erreur sémantique ne devrait être présente dans switch_good.own"
        );

        // Mauvais contenu
        let bad_source = read_source_file("switch/switch_bad.own");
        let bad_analyzer = analyze_source(&bad_source);
        assert!(
            !bad_analyzer.errors.is_empty(),
            "Des erreurs sémantiques devraient être présentes dans switch_bad.own"
        );
        assert_eq!(
            bad_analyzer.errors.len(),
            1,
            "Une seule erreur devrait être détectée dans switch_bad.own"
        );
        assert_eq!(
            bad_analyzer.errors[0],
            "Case type 'string' does not match switch type 'int'.",
            "Erreur de type incorrecte détectée dans switch_bad.own"
        );
    }

    #[test]
    fn test_function_declaration() {
        // Bon contenu
        let good_source = read_source_file("function/function_good.own");
        let good_analyzer = analyze_source(&good_source);
        assert!(
            good_analyzer.errors.is_empty(),
            "Aucune erreur sémantique ne devrait être présente dans function_good.own"
        );

        // Mauvais contenu
        let bad_source = read_source_file("function/function_bad.own");
        let bad_analyzer = analyze_source(&bad_source);
        assert!(
            !bad_analyzer.errors.is_empty(),
            "Des erreurs sémantiques devraient être présentes dans function_bad.own"
        );
        assert_eq!(
            bad_analyzer.errors.len(),
            2,
            "Deux erreurs devraient être détectées dans function_bad.own"
        );
        assert!(bad_analyzer.errors.contains(&"Type mismatch in return statement: expected 'int', found 'string'.".to_string()));
        assert!(bad_analyzer.errors.contains(&"Type mismatch in variable declaration 'result': expected 'int', found 'float'.".to_string()));
    }

    // Vous pouvez ajouter d'autres tests pour couvrir plus de cas spécifiques
}
