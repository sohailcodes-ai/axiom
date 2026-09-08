use std::collections::HashMap;
use crate::serialization::{AxiomValue, Serializer, Deserializer};
use crate::transport::{Request, Response, TransportError};
use crate::network::TcpTransportServer;
use crate::domain_analysis::DomainKind;
use crate::vm::{VM, VMProgram, VMFunction, VMOpcode, VMInstruction, VMConstant};

pub struct AxiomServer {
    program: VMProgram,
    function_domains: HashMap<String, DomainKind>,
    address: String,
}

impl AxiomServer {
    pub fn new(program: VMProgram, function_domains: HashMap<String, DomainKind>, address: &str) -> Self {
        AxiomServer {
            program,
            function_domains,
            address: address.to_string(),
        }
    }

    pub fn serve(&self) -> Result<(), Box<dyn std::error::Error>> {
        let program = self.program.clone();
        let function_domains = self.function_domains.clone();

        let server = TcpTransportServer::new(&self.address, move |request| {
            Self::handle_request(&program, &function_domains, request)
        });

        eprintln!("[AXIOM SERVER] listening on {}", self.address);
        server.serve()?;
        Ok(())
    }

    fn handle_request(
        program: &VMProgram,
        function_domains: &HashMap<String, DomainKind>,
        request: &Request,
    ) -> Result<AxiomValue, String> {
        let func = program.functions.iter().find(|f| f.name == request.function_name);

        let func = match func {
            Some(f) => f,
            None => return Err(format!("function '{}' not found", request.function_name)),
        };

        let expected_domain = function_domains.get(&request.function_name);
        let target_domain = match request.target_domain.as_str() {
            "server" => Some(DomainKind::Server),
            "client" => Some(DomainKind::Client),
            _ => None,
        };

        if let (Some(expected), Some(actual)) = (expected_domain, target_domain) {
            if *expected != actual {
                return Err(format!(
                    "function '{}' belongs to {} domain, not {}",
                    request.function_name, expected, actual
                ));
            }
        }

        if func.param_count != request.arguments.len() {
            return Err(format!(
                "function '{}' expects {} arguments, got {}",
                request.function_name, func.param_count, request.arguments.len()
            ));
        }

        let mut vm = VM::new(program.clone());
        let mut local_values = vec![crate::vm::Value::Unit; func.locals_count];
        for (i, arg) in request.arguments.iter().enumerate() {
            if i < local_values.len() {
                local_values[i] = arg.clone().into();
            }
        }

        let main_idx = program.functions.iter().position(|f| f.name == request.function_name);
        if let Some(idx) = main_idx {
            let frame = crate::vm::StackFrame {
                function_idx: idx,
                locals: local_values,
                ip: 0,
                stack: Vec::new(),
            };
            vm.frames.push(frame);

            loop {
                if vm.frames.is_empty() {
                    break;
                }
                if let Err(e) = vm.execute_instruction() {
                    return Err(e);
                }
            }

            if let Some(frame) = vm.frames.first() {
                if let Some(value) = frame.stack.last() {
                    return Ok(value.clone().into());
                }
            }
        }

        Ok(AxiomValue::Unit)
    }
}

pub fn run_server(
    source: &str,
    filename: &str,
    address: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut lexer = crate::lexer::Lexer::new(source, filename);
    let tokens = lexer.tokenize();

    let mut parser = crate::parser::Parser::new(tokens, filename);
    let ast = parser.parse_program()?;

    let mut resolver = crate::name_resolver::NameResolver::new(filename);
    resolver.resolve_program(&ast);
    if !resolver.errors.is_empty() {
        for error in &resolver.errors {
            eprintln!("{}", error);
        }
        return Err("name resolution failed".into());
    }

    let mut checker = crate::type_checker::TypeChecker::new(filename);
    checker.check_program(&ast);
    if !checker.errors.is_empty() {
        for error in &checker.errors {
            eprintln!("{}", error);
        }
        return Err("type checking failed".into());
    }

    let mut domain_analyzer = crate::domain_analysis::DomainAnalyzer::new(filename, checker.functions.clone());
    domain_analyzer.analyze_program(&ast);
    if !domain_analyzer.errors.is_empty() {
        for error in &domain_analyzer.errors {
            eprintln!("{}", error);
        }
        return Err("domain analysis failed".into());
    }

    let mut lowerer = crate::hir_lower::HIRLowerer::new(filename);
    let domain_map = domain_analyzer.get_all_function_domains();
    lowerer.set_domain_map(domain_map.clone());
    let struct_fields = domain_analyzer.get_struct_fields().clone();
    lowerer.set_struct_fields(&struct_fields);
    let hir = lowerer.lower_program(&ast);
    if !lowerer.errors.is_empty() {
        for error in &lowerer.errors {
            eprintln!("{}", error);
        }
        return Err("HIR lowering failed".into());
    }

    let mut codegen = crate::codegen::CodeGenerator::new(filename);
    let program = codegen.generate(&hir);

    let server = AxiomServer::new(program, domain_map, address);
    server.serve()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let program = VMProgram {
            functions: vec![],
            global_names: vec![],
        };
        let domains = HashMap::new();
        let server = AxiomServer::new(program, domains, "127.0.0.1:8080");
        assert_eq!(server.address, "127.0.0.1:8080");
    }
}
