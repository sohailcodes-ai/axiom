mod tokens;
mod lexer;
mod ast;
mod parser;
mod errors;
mod name_resolver;
mod type_checker;
mod domain_analysis;
mod hir;
mod hir_lower;
mod vm;
mod codegen;

use std::env;
use std::fs;
use std::process;

use lexer::Lexer;
use parser::Parser;
use name_resolver::NameResolver;
use type_checker::TypeChecker;
use domain_analysis::DomainAnalyzer;
use hir_lower::HIRLowerer;
use codegen::CodeGenerator;
use vm::VM;

fn compile_and_run(source: &str, filename: &str) -> i32 {
    // Lex
    let mut lexer = Lexer::new(source, filename);
    let tokens = lexer.tokenize();
    
    // Parse
    let mut parser = Parser::new(tokens, filename);
    let ast = match parser.parse_program() {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("parse error: {}", e);
            return 1;
        }
    };
    
    // Name resolution
    let mut resolver = NameResolver::new(filename);
    resolver.resolve_program(&ast);
    if !resolver.errors.is_empty() {
        for error in &resolver.errors {
            eprintln!("{}", error);
        }
        return 1;
    }
    
    // Type checking
    let mut checker = TypeChecker::new(filename);
    checker.check_program(&ast);
    if !checker.errors.is_empty() {
        for error in &checker.errors {
            eprintln!("{}", error);
        }
        return 1;
    }

    // Domain analysis
    let mut domain_analyzer = DomainAnalyzer::new(filename, checker.functions.clone());
    domain_analyzer.analyze_program(&ast);
    if !domain_analyzer.errors.is_empty() {
        for error in &domain_analyzer.errors {
            eprintln!("{}", error);
        }
        return 1;
    }

    // Lower to HIR
    let mut lowerer = HIRLowerer::new(filename);
    let domain_map = domain_analyzer.get_all_function_domains();
    lowerer.set_domain_map(domain_map);
    let struct_fields = domain_analyzer.get_struct_fields().clone();
    lowerer.set_struct_fields(&struct_fields);
    let hir = lowerer.lower_program(&ast);
    if !lowerer.errors.is_empty() {
        for error in &lowerer.errors {
            eprintln!("{}", error);
        }
        return 1;
    }
    
    // Generate bytecode
    let mut codegen = CodeGenerator::new(filename);
    let program = codegen.generate(&hir);
    if !codegen.errors.is_empty() {
        for error in &codegen.errors {
            eprintln!("{}", error);
        }
        return 1;
    }
    
    // Run VM
    let mut vm = VM::new(program);
    let exit_code = vm.run();
    
    for line in &vm.output {
        println!("{}", line);
    }
    
    if !vm.errors.is_empty() {
        for error in &vm.errors {
            eprintln!("{}", error);
        }
        return 1;
    }
    
    exit_code
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        eprintln!("usage: axiom <command> <file>");
        eprintln!("commands:");
        eprintln!("  run <file.ax>    compile and run");
        eprintln!("  lex <file.ax>    tokenize");
        eprintln!("  parse <file.ax>  parse");
        process::exit(1);
    }
    
    let command = &args[1];
    
    match command.as_str() {
        "run" => {
            if args.len() < 3 {
                eprintln!("usage: axiom run <file.ax>");
                process::exit(1);
            }
            
            let filename = &args[2];
            let source = match fs::read_to_string(filename) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error reading {}: {}", filename, e);
                    process::exit(1);
                }
            };
            
            let exit_code = compile_and_run(&source, filename);
            process::exit(exit_code);
        }
        "lex" => {
            if args.len() < 3 {
                eprintln!("usage: axiom lex <file.ax>");
                process::exit(1);
            }
            
            let filename = &args[2];
            let source = match fs::read_to_string(filename) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error reading {}: {}", filename, e);
                    process::exit(1);
                }
            };
            
            let mut lexer = Lexer::new(&source, filename);
            let tokens = lexer.tokenize();
            
            for token in &tokens {
                println!("{:?}", token);
            }
        }
        "parse" => {
            if args.len() < 3 {
                eprintln!("usage: axiom parse <file.ax>");
                process::exit(1);
            }
            
            let filename = &args[2];
            let source = match fs::read_to_string(filename) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error reading {}: {}", filename, e);
                    process::exit(1);
                }
            };
            
            let mut lexer = Lexer::new(&source, filename);
            let tokens = lexer.tokenize();
            
            let mut parser = Parser::new(tokens, filename);
            match parser.parse_program() {
                Ok(ast) => {
                    println!("parse successful");
                    println!("{:#?}", ast);
                }
                Err(e) => {
                    eprintln!("parse error: {}", e);
                    process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("unknown command: {}", command);
            process::exit(1);
        }
    }
}
