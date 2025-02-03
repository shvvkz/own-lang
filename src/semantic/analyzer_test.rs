#[cfg(test)]
mod analyzer_tests {
    use std::fs;
    use std::path::Path;

    
    use crate::semantic::analyzer::SemanticAnalyzer;

    // Helper function to read a source file and return its content as a String
    fn read_source_file(filename: &str) -> String {
        let path = Path::new("own_files").join(filename);
        fs::read_to_string(path).expect(&format!("Failed to read file {}", filename))
    }

    // Helper function to perform semantic analysis on source code
    fn analyze_source(source: &str) -> SemanticAnalyzer {
        let mut analyzer = SemanticAnalyzer::new(source.to_string());
        analyzer.analyze();
        analyzer
    }

    #[test]
    fn test_var_declaration() {
        // Bonne déclaration de variable
        let good_source = read_source_file("var_decl/var_decl_good.own");
        let good_analyzer = analyze_source(&good_source);
        assert!(
            good_analyzer.errors.is_empty(),
            "No semantic errors expected in var_decl_good.own, got: {:?}",
            good_analyzer.errors
        );

        // Mauvaise déclaration de variable (mismatch de type)
        let bad_source = read_source_file("var_decl/var_decl_bad.own");
        let bad_analyzer = analyze_source(&bad_source);
        // Par exemple, notre analyseur produit :
        // "Type mismatch in variable declaration 'x': expected 'string', found 'int'."
        assert!(
            !bad_analyzer.errors.is_empty(),
            "Semantic errors expected in var_decl_bad.own"
        );
        assert_eq!(
            bad_analyzer.errors.len(),
            1,
            "Exactly one error expected in var_decl_bad.own, got: {:?}",
            bad_analyzer.errors
        );
        assert_eq!(
            bad_analyzer.errors[0],
            "Type mismatch in variable declaration 'x': expected 'string', found 'int'."
        );
    }

    #[test]
    fn test_return_statement() {
        // Bonne utilisation de return (enveloppé dans une fonction)
        let good_source = read_source_file("return/return_good.own");
        let good_analyzer = analyze_source(&good_source);
        assert!(
            good_analyzer.errors.is_empty(),
            "No semantic errors expected in return_good.own, got: {:?}",
            good_analyzer.errors
        );

        // Mauvaise utilisation de return (mismatch de type)
        let bad_source = read_source_file("return/return_bad.own");
        let bad_analyzer = analyze_source(&bad_source);
        assert!(
            !bad_analyzer.errors.is_empty(),
            "Semantic errors expected in return_bad.own"
        );
        assert_eq!(
            bad_analyzer.errors.len(),
            1,
            "Exactly one error expected in return_bad.own, got: {:?}",
            bad_analyzer.errors
        );
        assert_eq!(
            bad_analyzer.errors[0],
            "Type mismatch in return statement: expected 'int', found 'string'."
        );
    }

    #[test]
    fn test_var_affection() {
        // Bonne affection de variable
        let good_source = read_source_file("var_affection/var_affection_good.own");
        let good_analyzer = analyze_source(&good_source);
        assert!(
            good_analyzer.errors.is_empty(),
            "No semantic errors expected in var_affection_good.own, got: {:?}",
            good_analyzer.errors
        );

        // Mauvaise affection de variable (mismatch de type)
        let bad_source = read_source_file("var_affection/var_affection_bad.own");
        let bad_analyzer = analyze_source(&bad_source);
        assert!(
            !bad_analyzer.errors.is_empty(),
            "Semantic errors expected in var_affection_bad.own"
        );
        assert_eq!(
            bad_analyzer.errors.len(),
            1,
            "Exactly one error expected in var_affection_bad.own, got: {:?}",
            bad_analyzer.errors
        );
        assert_eq!(
            bad_analyzer.errors[0],
            "Type mismatch in assignment to 'x': expected 'int', found 'string'."
        );
    }

    #[test]
    fn test_if_statement() {
        // Pour éviter l'erreur "Return statement not inside a function", le fichier if_good.own
        // doit être enveloppé dans une fonction.
        let good_source = read_source_file("if/if_good.own");
        let good_analyzer = analyze_source(&good_source);
        assert!(
            good_analyzer.errors.is_empty(),
            "No semantic errors expected in if_good.own, got: {:?}",
            good_analyzer.errors
        );

        // Mauvais if: condition non booléenne
        let bad_source = read_source_file("if/if_bad.own");
        let bad_analyzer = analyze_source(&bad_source);
        assert!(
            !bad_analyzer.errors.is_empty(),
            "Semantic errors expected in if_bad.own"
        );
        assert_eq!(
            bad_analyzer.errors.len(),
            1,
            "Exactly one error expected in if_bad.own, got: {:?}",
            bad_analyzer.errors
        );
        assert_eq!(
            bad_analyzer.errors[0],
            "Condition in 'if' statement must be of type 'bool', found 'int'."
        );
    }

    #[test]
    fn test_for_statement() {
        // Pour éviter les erreurs de retour hors fonction, le contenu for_good.own doit être placé
        // dans une fonction.
        let good_source = read_source_file("for/for_good.own");
        let good_analyzer = analyze_source(&good_source);
        assert!(
            good_analyzer.errors.is_empty(),
            "No semantic errors expected in for_good.own, got: {:?}",
            good_analyzer.errors
        );

        // Mauvais for: conditions et incréments de mauvais types
        let bad_source = read_source_file("for/for_bad.own");
        let bad_analyzer = analyze_source(&bad_source);
        assert!(
            !bad_analyzer.errors.is_empty(),
            "Semantic errors expected in for_bad.own"
        );
        // Ici, nous attendons par exemple 3 erreurs. Ajustez ce nombre selon ce que génère réellement votre analyseur.
        assert_eq!(
            bad_analyzer.errors.len(),
            2,
            "Expected 2 errors in for_bad.own, got: {:?}",
            bad_analyzer.errors
        );
        // Vérifier qu'une des erreurs concerne la condition
        assert!(bad_analyzer
            .errors
            .iter()
            .any(|e| e.contains("Condition in 'for' statement must be of type 'bool'")));
        // Vérifier qu'une autre erreur concerne l'incrément
        assert!(bad_analyzer
            .errors
            .iter()
            .any(|e| e.contains("Type mismatch in assignment to 'i'")));
    }

    #[test]
    fn test_while_statement() {
        let good_source = read_source_file("while/while_good.own");
        let good_analyzer = analyze_source(&good_source);
        assert!(
            good_analyzer.errors.is_empty(),
            "No semantic errors expected in while_good.own, got: {:?}",
            good_analyzer.errors
        );

        let bad_source = read_source_file("while/while_bad.own");
        let bad_analyzer = analyze_source(&bad_source);
        assert!(
            !bad_analyzer.errors.is_empty(),
            "Semantic errors expected in while_bad.own"
        );
        assert_eq!(
            bad_analyzer.errors.len(),
            1,
            "Expected 1 error in while_bad.own, got: {:?}",
            bad_analyzer.errors
        );
        assert_eq!(
            bad_analyzer.errors[0],
            "Condition in 'while' statement must be of type 'bool', found 'string'."
        );
    }

    #[test]
    fn test_switch_statement() {
        let good_source = read_source_file("switch/switch_good.own");
        let good_analyzer = analyze_source(&good_source);
        assert!(
            good_analyzer.errors.is_empty(),
            "No semantic errors expected in switch_good.own, got: {:?}",
            good_analyzer.errors
        );

        let bad_source = read_source_file("switch/switch_bad.own");
        let bad_analyzer = analyze_source(&bad_source);
        assert!(
            !bad_analyzer.errors.is_empty(),
            "Semantic errors expected in switch_bad.own"
        );
        assert_eq!(
            bad_analyzer.errors.len(),
            1,
            "Expected 1 error in switch_bad.own, got: {:?}",
            bad_analyzer.errors
        );
        assert_eq!(
            bad_analyzer.errors[0],
            "Case type 'string' does not match switch type 'int'."
        );
    }

    #[test]
    fn test_function_declaration() {
        let good_source = read_source_file("function/function_good.own");
        let good_analyzer = analyze_source(&good_source);
        assert!(
            good_analyzer.errors.is_empty(),
            "No semantic errors expected in function_good.own, got: {:?}",
            good_analyzer.errors
        );

        let bad_source = read_source_file("function/function_bad.own");
        let bad_analyzer = analyze_source(&bad_source);
        assert!(
            !bad_analyzer.errors.is_empty(),
            "Semantic errors expected in function_bad.own"
        );
        // Dans function_bad.own, on attend par exemple 2 erreurs :
        // - Une erreur pour le type de la déclaration de variable 'result'
        // - Une erreur pour le return qui ne correspond pas
        assert_eq!(
            bad_analyzer.errors.len(),
            2,
            "Expected 2 errors in function_bad.own, got: {:?}",
            bad_analyzer.errors
        );
        assert!(bad_analyzer
            .errors
            .iter()
            .any(|e| e.contains("Type mismatch in variable declaration 'result'")));
        assert!(bad_analyzer
            .errors
            .iter()
            .any(|e| e.contains("Type mismatch in return statement")));
    }
}
