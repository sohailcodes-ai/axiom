use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use crate::serialization::{Serializer, Deserializer, AxiomValue};
use crate::transport::{Transport, Request, Response, TransportError};

const MAX_REQUEST_SIZE: usize = 10 * 1024 * 1024;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

pub struct TcpTransport {
    address: String,
    timeout: Duration,
}

impl TcpTransport {
    pub fn new(address: &str) -> Self {
        TcpTransport {
            address: address.to_string(),
            timeout: DEFAULT_TIMEOUT,
        }
    }

    pub fn with_timeout(address: &str, timeout: Duration) -> Self {
        TcpTransport {
            address: address.to_string(),
            timeout,
        }
    }

    fn send_bytes(&self, data: &[u8]) -> Result<Vec<u8>, TransportError> {
        let mut stream = TcpStream::connect(&self.address)
            .map_err(|_| TransportError::ConnectionRefused)?;

        stream.set_read_timeout(Some(self.timeout))
            .map_err(|e| TransportError::TransportFailure(e.to_string()))?;
        stream.set_write_timeout(Some(self.timeout))
            .map_err(|e| TransportError::TransportFailure(e.to_string()))?;

        let len = data.len() as u32;
        stream.write_all(&len.to_le_bytes())
            .map_err(|e| TransportError::TransportFailure(e.to_string()))?;
        stream.write_all(data)
            .map_err(|e| TransportError::TransportFailure(e.to_string()))?;

        let mut len_buf = [0u8; 4];
        stream.read_exact(&mut len_buf)
            .map_err(|e| TransportError::TransportFailure(e.to_string()))?;
        let response_len = u32::from_le_bytes(len_buf) as usize;

        if response_len > MAX_REQUEST_SIZE {
            return Err(TransportError::TransportFailure(
                "response too large".to_string()
            ));
        }

        let mut response_buf = vec![0u8; response_len];
        stream.read_exact(&mut response_buf)
            .map_err(|e| TransportError::TransportFailure(e.to_string()))?;

        Ok(response_buf)
    }
}

impl Transport for TcpTransport {
    fn send_request(&self, request: &Request) -> Result<Response, TransportError> {
        let serialized_args: Result<Vec<Vec<u8>>, _> = request.arguments.iter()
            .map(|arg| Serializer::serialize(arg))
            .collect();
        let serialized_args = serialized_args?;

        let request_data = format!(
            "{}|{}|{}|{}",
            request.function_name,
            request.source_domain,
            request.target_domain,
            serialized_args.len()
        );

        let mut payload = Vec::new();
        payload.extend_from_slice(request_data.as_bytes());
        payload.push(0);
        for arg_bytes in &serialized_args {
            payload.extend_from_slice(arg_bytes);
        }

        let response_bytes = self.send_bytes(&payload)?;

        let response_str = String::from_utf8_lossy(&response_bytes);
        if let Some(pos) = response_str.find('|') {
            let success_str = &response_str[..pos];
            let rest = &response_str[pos + 1..];

            if success_str == "ok" {
                let value = Deserializer::deserialize(rest.as_bytes())
                    .map_err(TransportError::Serialization)?;
                Ok(Response::ok(value))
            } else {
                Ok(Response::err(rest.to_string()))
            }
        } else {
            Err(TransportError::TransportFailure(
                "invalid response format".to_string()
            ))
        }
    }
}

pub struct TcpTransportServer {
    address: String,
    handler: Arc<dyn Fn(&Request) -> Result<AxiomValue, String> + Send + Sync>,
}

impl TcpTransportServer {
    pub fn new<F>(address: &str, handler: F) -> Self
    where
        F: Fn(&Request) -> Result<AxiomValue, String> + Send + Sync + 'static,
    {
        TcpTransportServer {
            address: address.to_string(),
            handler: Arc::new(handler),
        }
    }

    pub fn serve(&self) -> Result<(), TransportError> {
        let listener = TcpListener::bind(&self.address)
            .map_err(|e| TransportError::TransportFailure(e.to_string()))?;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let handler = Arc::clone(&self.handler);
                    thread::spawn(move || {
                        Self::handle_connection(stream, handler);
                    });
                }
                Err(e) => {
                    eprintln!("connection failed: {}", e);
                }
            }
        }
        Ok(())
    }

    fn handle_connection(
        mut stream: TcpStream,
        handler: Arc<dyn Fn(&Request) -> Result<AxiomValue, String> + Send + Sync>,
    ) {
        let mut len_buf = [0u8; 4];
        if stream.read_exact(&mut len_buf).is_err() {
            return;
        }
        let request_len = u32::from_le_bytes(len_buf) as usize;

        if request_len > MAX_REQUEST_SIZE {
            return;
        }

        let mut request_buf = vec![0u8; request_len];
        if stream.read_exact(&mut request_buf).is_err() {
            return;
        }

        let separator_pos = request_buf.iter().position(|&b| b == 0);
        let separator_pos = match separator_pos {
            Some(pos) => pos,
            None => return,
        };

        let header = match String::from_utf8(request_buf[..separator_pos].to_vec()) {
            Ok(s) => s,
            Err(_) => return,
        };

        let parts: Vec<&str> = header.split('|').collect();
        if parts.len() != 4 {
            return;
        }

        let function_name = parts[0].to_string();
        let source_domain = parts[1].to_string();
        let target_domain = parts[2].to_string();

        let args_data = &request_buf[separator_pos + 1..];
        let mut arguments = Vec::new();
        let mut pos = 0;
        while pos < args_data.len() {
            match Deserializer::deserialize(&args_data[pos..]) {
                Ok(value) => {
                    arguments.push(value);
                    break;
                }
                Err(_) => break,
            }
        }

        let request = Request {
            function_name,
            arguments,
            source_domain,
            target_domain,
        };

        let response = match handler(&request) {
            Ok(value) => {
                let serialized = Serializer::serialize(&value).unwrap_or_default();
                let mut resp = b"ok|".to_vec();
                resp.extend_from_slice(&serialized);
                resp
            }
            Err(e) => {
                format!("err|{}", e).into_bytes()
            }
        };

        let resp_len = response.len() as u32;
        let _ = stream.write_all(&resp_len.to_le_bytes());
        let _ = stream.write_all(&response);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tcp_transport_creation() {
        let transport = TcpTransport::new("127.0.0.1:9000");
        assert_eq!(transport.address, "127.0.0.1:9000");
    }

    #[test]
    fn test_tcp_transport_with_timeout() {
        let transport = TcpTransport::with_timeout("127.0.0.1:9000", Duration::from_secs(5));
        assert_eq!(transport.timeout, Duration::from_secs(5));
    }

    #[test]
    fn test_request_serialization() {
        let request = Request {
            function_name: "test".to_string(),
            arguments: vec![AxiomValue::Integer(42)],
            source_domain: "client".to_string(),
            target_domain: "server".to_string(),
        };

        let mut payload = Vec::new();
        let header = format!("{}|{}|{}|{}",
            request.function_name,
            request.source_domain,
            request.target_domain,
            1
        );
        payload.extend_from_slice(header.as_bytes());
        payload.push(0);

        assert!(!payload.is_empty());
    }
}
