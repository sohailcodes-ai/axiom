use std::collections::HashMap;
use crate::serialization::{AxiomValue, SerializationError};
use crate::transport::{Transport, Request, TransportError, InMemoryTransport};
use crate::domain_analysis::DomainKind;

#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub domain: DomainKind,
    pub param_count: usize,
    pub param_types: Vec<String>,
    pub return_type: String,
}

#[derive(Debug, Clone)]
pub enum RuntimeError {
    Serialization(SerializationError),
    Transport(TransportError),
    FunctionNotFound(String),
    DomainViolation(String),
    ExecutionFailed(String),
    TypeError(String),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialization(e) => write!(f, "serialization error: {}", e),
            Self::Transport(e) => write!(f, "transport error: {}", e),
            Self::FunctionNotFound(name) => write!(f, "function not found: {}", name),
            Self::DomainViolation(msg) => write!(f, "domain violation: {}", msg),
            Self::ExecutionFailed(msg) => write!(f, "execution failed: {}", msg),
            Self::TypeError(msg) => write!(f, "type error: {}", msg),
        }
    }
}

impl std::error::Error for RuntimeError {}

impl From<SerializationError> for RuntimeError {
    fn from(e: SerializationError) -> Self {
        Self::Serialization(e)
    }
}

impl From<TransportError> for RuntimeError {
    fn from(e: TransportError) -> Self {
        Self::Transport(e)
    }
}

pub struct CrossDomainRuntime {
    functions: HashMap<String, FunctionSignature>,
    transport: Box<dyn Transport>,
    output: Vec<String>,
}

impl CrossDomainRuntime {
    pub fn new() -> Self {
        Self::with_transport(Box::new(InMemoryTransport::new()))
    }

    pub fn with_transport(transport: Box<dyn Transport>) -> Self {
        CrossDomainRuntime {
            functions: HashMap::new(),
            transport,
            output: Vec::new(),
        }
    }

    pub fn register_function(&mut self, sig: FunctionSignature) {
        self.functions.insert(sig.name.clone(), sig);
    }

    pub fn execute_cross_domain_call(
        &mut self,
        caller_domain: &str,
        function_name: &str,
        args: &[AxiomValue],
    ) -> Result<AxiomValue, RuntimeError> {
        let sig = self.functions.get(function_name)
            .ok_or_else(|| RuntimeError::FunctionNotFound(function_name.to_string()))?;

        let target_domain = match sig.domain {
            DomainKind::Server => "server".to_string(),
            DomainKind::Client => "client".to_string(),
        };

        if caller_domain == target_domain {
            return Err(RuntimeError::DomainViolation(format!(
                "function '{}' is in {} domain, cannot cross-call within same domain",
                function_name, caller_domain
            )));
        }

        if caller_domain == "server" && target_domain == "client" {
            return Err(RuntimeError::DomainViolation(format!(
                "server->client calls are not supported: use events/messages"
            )));
        }

        for (i, arg) in args.iter().enumerate() {
            if !arg.is_transmissible() {
                return Err(RuntimeError::TypeError(format!(
                    "argument {} of '{}' is not transmissible",
                    i + 1, function_name
                )));
            }
        }

        let request = Request {
            function_name: function_name.to_string(),
            arguments: args.to_vec(),
            source_domain: caller_domain.to_string(),
            target_domain: target_domain.clone(),
        };

        self.output.push(format!(
            "[RUNTIME] {}->{}: calling '{}' with {} args",
            caller_domain, target_domain, function_name, args.len()
        ));

        let response = self.transport.send_request(&request)?;

        if response.success {
            self.output.push(format!(
                "[RUNTIME] {}->{}: '{}' returned successfully",
                caller_domain, target_domain, function_name
            ));
            Ok(response.value.unwrap_or(AxiomValue::Unit))
        } else {
            let err_msg = response.error.unwrap_or_else(|| "unknown error".to_string());
            self.output.push(format!(
                "[RUNTIME] {}->{}: '{}' failed: {}",
                caller_domain, target_domain, function_name, err_msg
            ));
            Err(RuntimeError::ExecutionFailed(err_msg))
        }
    }

    pub fn get_output(&self) -> &[String] {
        &self.output
    }

    pub fn clear_output(&mut self) {
        self.output.clear();
    }
}

impl Default for CrossDomainRuntime {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = CrossDomainRuntime::new();
        assert!(runtime.functions.is_empty());
    }

    #[test]
    fn test_register_function() {
        let mut runtime = CrossDomainRuntime::new();
        runtime.register_function(FunctionSignature {
            name: "get_user".to_string(),
            domain: DomainKind::Server,
            param_count: 1,
            param_types: vec!["Int".to_string()],
            return_type: "User".to_string(),
        });
        assert!(runtime.functions.contains_key("get_user"));
    }

    #[test]
    fn test_unknown_function() {
        let mut runtime = CrossDomainRuntime::new();
        let result = runtime.execute_cross_domain_call(
            "client",
            "unknown_func",
            &[],
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_same_domain_violation() {
        let mut runtime = CrossDomainRuntime::new();
        runtime.register_function(FunctionSignature {
            name: "get_user".to_string(),
            domain: DomainKind::Server,
            param_count: 0,
            param_types: vec![],
            return_type: "Int".to_string(),
        });
        let result = runtime.execute_cross_domain_call(
            "server",
            "get_user",
            &[],
        );
        assert!(result.is_err());
    }
}
