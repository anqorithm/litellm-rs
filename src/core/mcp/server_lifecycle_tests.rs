use super::*;
use crate::core::mcp::protocol::{JsonRpcError, ToolsCapability};
use serde_json::json;
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

struct MockResponse {
    status: &'static str,
    body: String,
}

struct MockLifecycleServer {
    url: String,
    requests: oneshot::Receiver<Vec<Value>>,
    task: JoinHandle<io::Result<()>>,
}

impl MockLifecycleServer {
    async fn start(responses: Vec<MockResponse>) -> io::Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
        let address = listener.local_addr()?;
        let (request_sender, requests) = oneshot::channel();

        let task = tokio::spawn(async move {
            let mut captured = Vec::with_capacity(responses.len());
            for response in responses {
                let (mut socket, _) = listener.accept().await?;
                captured.push(read_json_body(&mut socket).await?);
                let wire_response = format!(
                    "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    response.status,
                    response.body.len(),
                    response.body
                );
                socket.write_all(wire_response.as_bytes()).await?;
            }

            request_sender.send(captured).map_err(|_| {
                io::Error::new(
                    io::ErrorKind::BrokenPipe,
                    "lifecycle request receiver was dropped",
                )
            })
        });

        Ok(Self {
            url: format!("http://{address}/mcp"),
            requests,
            task,
        })
    }

    async fn captured_requests(self) -> TestResult<Vec<Value>> {
        let requests = self.requests.await?;
        self.task.await??;
        Ok(requests)
    }
}

async fn read_json_body(socket: &mut TcpStream) -> io::Result<Value> {
    let mut request = Vec::new();
    let mut buffer = [0_u8; 2048];

    loop {
        let bytes_read = socket.read(&mut buffer).await?;
        if bytes_read == 0 {
            return Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "mock MCP server received an incomplete HTTP request",
            ));
        }
        request.extend_from_slice(&buffer[..bytes_read]);

        let Some(header_end) = request.windows(4).position(|window| window == b"\r\n\r\n") else {
            continue;
        };
        let headers = String::from_utf8_lossy(&request[..header_end]);
        let content_length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().ok())
                    .flatten()
            })
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    "mock MCP request is missing content-length",
                )
            })?;
        let body_start = header_end + 4;
        if request.len() < body_start + content_length {
            continue;
        }

        return serde_json::from_slice(&request[body_start..body_start + content_length])
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error));
    }
}

fn valid_initialize_result() -> Value {
    json!({
        "protocolVersion": SUPPORTED_PROTOCOL_VERSION,
        "capabilities": {"tools": {"listChanged": true}},
        "serverInfo": {"name": "mock-mcp", "version": "1.0.0"}
    })
}

fn initialize_response(result: Value) -> JsonRpcResponse {
    JsonRpcResponse::success(result, json!(1))
}

fn test_server(url: String) -> McpServer {
    McpServer {
        config: McpServerConfig::new("test", url).with_timeout(2_000),
        state: RwLock::new(ServerState::Disconnected),
        http_client: get_client_with_timeout(Duration::from_secs(2)),
        custom_headers: reqwest::header::HeaderMap::new(),
        tools_cache: RwLock::new(None),
        tools_baseline_hash: RwLock::new(None),
        capabilities: RwLock::new(None),
        request_id: std::sync::atomic::AtomicU64::new(1),
    }
}

#[test]
fn initialize_response_fails_closed_on_invalid_shapes() {
    let mut missing_capabilities = valid_initialize_result();
    missing_capabilities
        .as_object_mut()
        .expect("fixture should be an object")
        .remove("capabilities");

    let invalid_responses = [
        initialize_response(missing_capabilities),
        initialize_response(json!({
            "protocolVersion": SUPPORTED_PROTOCOL_VERSION,
            "capabilities": [],
            "serverInfo": {"name": "mock-mcp", "version": "1.0.0"}
        })),
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: None,
            id: Some(json!(1)),
        },
        JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(valid_initialize_result()),
            error: Some(JsonRpcError::internal_error()),
            id: Some(json!(1)),
        },
    ];

    for response in invalid_responses {
        assert!(matches!(
            parse_initialize_response(response),
            Err(McpError::ProtocolError { .. })
        ));
    }
}

#[test]
fn initialize_response_rejects_unsupported_protocol_version() {
    let mut result = valid_initialize_result();
    result["protocolVersion"] = json!("2025-03-26");

    let error = parse_initialize_response(initialize_response(result))
        .expect_err("unsupported version should fail");
    assert!(matches!(error, McpError::ProtocolError { .. }));
    assert!(error.to_string().contains("2025-03-26"));
}

#[tokio::test]
async fn connect_sends_initialized_notification_before_committing_state() -> TestResult {
    let initialize_body = serde_json::to_string(&initialize_response(valid_initialize_result()))?;
    let mock = MockLifecycleServer::start(vec![
        MockResponse {
            status: "200 OK",
            body: initialize_body,
        },
        MockResponse {
            status: "202 Accepted",
            body: String::new(),
        },
    ])
    .await?;
    let server = test_server(mock.url.clone());

    server.connect().await?;
    assert_eq!(server.state().await, ServerState::Connected);
    assert_eq!(
        server
            .capabilities()
            .await
            .and_then(|caps| caps.tools)
            .map(|tools: ToolsCapability| tools.list_changed),
        Some(true)
    );

    let requests = mock.captured_requests().await?;
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[0]["method"], methods::INITIALIZE);
    assert_eq!(
        requests[0]["params"]["protocolVersion"],
        SUPPORTED_PROTOCOL_VERSION
    );
    assert!(requests[0].get("id").is_some());
    assert_eq!(requests[1]["method"], methods::INITIALIZED);
    assert!(requests[1].get("id").is_none());
    Ok(())
}

#[tokio::test]
async fn connect_does_not_publish_capabilities_when_notification_fails() -> TestResult {
    let initialize_body = serde_json::to_string(&initialize_response(valid_initialize_result()))?;
    let mock = MockLifecycleServer::start(vec![
        MockResponse {
            status: "200 OK",
            body: initialize_body,
        },
        MockResponse {
            status: "500 Internal Server Error",
            body: String::new(),
        },
    ])
    .await?;
    let server = test_server(mock.url.clone());

    let result = server.connect().await;
    assert!(matches!(result, Err(McpError::TransportError { .. })));
    assert_eq!(server.state().await, ServerState::Failed);
    assert!(server.capabilities().await.is_none());

    let requests = mock.captured_requests().await?;
    assert_eq!(requests.len(), 2);
    assert_eq!(requests[1]["method"], methods::INITIALIZED);
    assert!(requests[1].get("id").is_none());
    Ok(())
}
