use crate::serialization::{AxiomValue, SerializationError};

#[derive(Debug, Clone)]
pub enum TransportError {
    Serialization(SerializationError),
    UnknownFunction(String),
    DomainViolation { from: String, to: String, reason: String },
    ExecutionFailed(String),
    TransportFailure(String),
    Timeout,
    ConnectionRefused,
}

impl std::fmt::Display for TransportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialization(e) => write!(f, "serialization error: {}", e),
            Self::UnknownFunction(name) => write!(f, "unknown function: {}", name),
            Self::DomainViolation { from, to, reason } => {
                write!(f, "domain violation: {} -> {}: {}", from, to, reason)
            }
            Self::ExecutionFailed(msg) => write!(f, "execution failed: {}", msg),
            Self::TransportFailure(msg) => write!(f, "transport failure: {}", msg),
            Self::Timeout => write!(f, "timeout"),
            Self::ConnectionRefused => write!(f, "connection refused"),
        }
    }
}

impl std::error::Error for TransportError {}

impl From<SerializationError> for TransportError {
    fn from(e: SerializationError) -> Self {
        Self::Serialization(e)
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub function_name: String,
    pub arguments: Vec<AxiomValue>,
    pub source_domain: String,
    pub target_domain: String,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub success: bool,
    pub value: Option<AxiomValue>,
    pub error: Option<String>,
}

impl Response {
    pub fn ok(value: AxiomValue) -> Self {
        Response {
            success: true,
            value: Some(value),
            error: None,
        }
    }

    pub fn err(error: String) -> Self {
        Response {
            success: false,
            value: None,
            error: Some(error),
        }
    }
}

pub trait Transport {
    fn send_request(&self, request: &Request) -> Result<Response, TransportError>;
}

pub struct InMemoryTransport;

impl InMemoryTransport {
    pub fn new() -> Self {
        InMemoryTransport
    }
}

impl Transport for InMemoryTransport {
    fn send_request(&self, request: &Request) -> Result<Response, TransportError> {
        let serialized_args: Vec<Vec<u8>> = request.arguments.iter()
            .map(|arg| {
                let bytes = crate::serialization::Serializer::serialize(arg)?;
                Ok(bytes)
            })
            .collect::<Result<Vec<_>, SerializationError>>()?;

        let deserialized_args: Vec<AxiomValue> = serialized_args.iter()
            .map(|bytes| {
                crate::serialization::Deserializer::deserialize(bytes)
            })
            .collect::<Result<Vec<_>, SerializationError>>()?;

        Ok(Response::err(format!(
            "InMemoryTransport: function '{}' not found ({} args)",
            request.function_name,
            deserialized_args.len()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_creation() {
        let req = Request {
            function_name: "get_user".to_string(),
            arguments: vec![AxiomValue::Integer(42)],
            source_domain: "client".to_string(),
            target_domain: "server".to_string(),
        };
        assert_eq!(req.function_name, "get_user");
        assert_eq!(req.arguments.len(), 1);
    }

    #[test]
    fn test_response_ok() {
        let resp = Response::ok(AxiomValue::Integer(42));
        assert!(resp.success);
        assert_eq!(resp.value, Some(AxiomValue::Integer(42)));
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_response_err() {
        let resp = Response::err("something went wrong".to_string());
        assert!(!resp.success);
        assert!(resp.value.is_none());
        assert_eq!(resp.error, Some("something went wrong".to_string()));
    }

    #[test]
    fn test_transport_error_display() {
        let err = TransportError::UnknownFunction("foo".to_string());
        assert_eq!(err.to_string(), "unknown function: foo");
    }

    #[test]
    fn test_domain_violation_display() {
        let err = TransportError::DomainViolation {
            from: "server".to_string(),
            to: "client".to_string(),
            reason: "not supported".to_string(),
        };
        assert!(err.to_string().contains("server"));
        assert!(err.to_string().contains("client"));
    }

    #[test]
    fn test_in_memory_transport_serializes() {
        let transport = InMemoryTransport::new();
        let req = Request {
            function_name: "test".to_string(),
            arguments: vec![AxiomValue::Integer(42)],
            source_domain: "client".to_string(),
            target_domain: "server".to_string(),
        };
        let result = transport.send_request(&req);
        assert!(result.is_ok());
    }
}
