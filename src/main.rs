mod codegen;
mod lex;
mod parser;
mod semantic;

use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::process::Command;
use codegen::codegen::CodeGenerator;
use semantic::analyzer::SemanticAnalyzer;
use crate::parser::models::ast::AST;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <path_to_file>", args[0]);
        std::process::exit(1);
    }
    let input_path = &args[1];
    let source = fs::read_to_string(input_path)
        .expect("Failed to read source file");

    // Semantic Analysis
    let mut analyzer = SemanticAnalyzer::new(source);
    let errors = analyzer.analyze();
    if !errors.is_empty() {
        println!("Semantic analysis failed: {:?}", errors);
        std::process::exit(1);
    }
    let ast: AST = analyzer.ast;

    println!("{:?}", ast);

    // Code Generation
    let mut codegen = CodeGenerator::new();
    codegen.generate(&ast);
    let asm_code = codegen.asm.join("\n");

    // Write assembly code to output.asm
    let asm_file = "output.asm";
    let mut file = File::create(asm_file).expect("Failed to create output.asm");
    file.write_all(asm_code.as_bytes()).expect("Failed to write assembly code");
    println!("Assembly code written to {}", asm_file);

    // Assemble with nasm (format elf64)
    let object_file = "output.o";
    let nasm_status = Command::new("nasm")
        .args(&["-f", "elf64", asm_file, "-o", object_file])
        .status()
        .expect("Failed to execute nasm");
    if !nasm_status.success() {
        eprintln!("nasm failed to assemble the code.");
        std::process::exit(1);
    }
    println!("Object file generated: {}", object_file);

    // Link with ld to produce the executable, linking with libc
    let executable_file = format!("{}.owne", input_path.trim_end_matches(".own"));
    let ld_status = Command::new("ld")
        .args(&[object_file, "-o", &executable_file, "-lc", "--dynamic-linker", "/lib64/ld-linux-x86-64.so.2"])
        .status()
        .expect("Failed to execute ld");
    if !ld_status.success() {
        eprintln!("Linker failed to produce the executable.");
        std::process::exit(1);
    }
    println!("Executable generated: {}", executable_file);

    // Clean up intermediate files
    fs::remove_file(asm_file).expect("Failed to remove asm file");
    fs::remove_file(object_file).expect("Failed to remove object file");
}
