//! MCP gateway HTTP surface
//!
//! Exposes the configured MCP servers as one aggregated JSON-RPC endpoint at
//! `POST /mcp` (and `/mcp/`), mirroring the streamable-HTTP surface the Python
//! LiteLLM proxy serves: `initialize`, `notifications/initialized`,
//! `tools/list`, and `tools/call`.
//!
//! Tools from every configured server are aggregated into a single list with
//! `mcp_{server}__{tool}` names, and `tools/call` routes a prefixed name back to
//! its origin server. Connections are established lazily on first use.

use crate::core::mcp::gateway::McpGateway;
use crate::core::mcp::protocol::{
    ClientInfo, InitializeResult, JSONRPC_VERSION, JsonRpcError, JsonRpcResponse, McpCapabilities,
    SUPPORTED_PROTOCOL_VERSION, ToolsCapability, methods,
};
use crate::core::mcp::tools::Tool;
use crate::server::state::AppState;
use actix_web::{HttpResponse, http::StatusCode, web};
use serde_json::{Map, Value, json};
use tracing::warn;

/// JSON-RPC error code returned when no MCP servers are configured.
const MCP_NOT_CONFIGURED_CODE: i32 = -32001;

/// Handle one JSON-RPC message for the aggregated MCP surface.
pub async fn handle_jsonrpc(state: web::Data<AppState>, body: web::Bytes) -> HttpResponse {
    let message: Value = match serde_json::from_slice(&body) {
        Ok(message) => message,
        Err(error) => {
            return json_rpc_error(
                StatusCode::BAD_REQUEST,
                JsonRpcError::parse_error().with_data(json!({"detail": error.to_string()})),
                None,
            );
        }
    };

    let Some(object) = message.as_object() else {
        return invalid_request("JSON-RPC message must be an object", None);
    };

    let id = object.get("id").filter(|id| !id.is_null()).cloned();

    if let Some(version) = object.get("jsonrpc").and_then(Value::as_str)
        && version != JSONRPC_VERSION
    {
        return invalid_request(
            &format!("unsupported jsonrpc version '{version}'; expected {JSONRPC_VERSION}"),
            id,
        );
    }

    let Some(method) = object.get("method").and_then(Value::as_str) else {
        return invalid_request("JSON-RPC message must carry a string method", id);
    };

    // Notifications carry no id and expect no response body.
    if id.is_none() {
        return HttpResponse::Accepted().finish();
    }

    let Some(gateway) = state.mcp_gateway.clone() else {
        return json_rpc_error(
            StatusCode::NOT_IMPLEMENTED,
            JsonRpcError::server_error(
                MCP_NOT_CONFIGURED_CODE,
                "MCP is not configured; set mcp.servers in the gateway config to enable it",
            ),
            id,
        );
    };

    let params = object.get("params").cloned().unwrap_or(Value::Null);

    match method {
        methods::INITIALIZE => initialize(id),
        methods::PING => success(json!({}), id),
        methods::LIST_TOOLS => list_tools(&gateway, id).await,
        methods::CALL_TOOL => call_tool(&gateway, &params, id).await,
        other => json_rpc_error(
            StatusCode::OK,
            JsonRpcError::method_not_found().with_data(json!({"method": other})),
            id,
        ),
    }
}

fn initialize(id: Option<Value>) -> HttpResponse {
    let result = InitializeResult {
        protocol_version: SUPPORTED_PROTOCOL_VERSION.to_string(),
        capabilities: McpCapabilities {
            tools: Some(ToolsCapability {
                list_changed: false,
            }),
            ..McpCapabilities::default()
        },
        server_info: ClientInfo::default(),
    };

    match serde_json::to_value(&result) {
        Ok(value) => success(value, id),
        Err(error) => internal_error(&error.to_string(), id),
    }
}

async fn list_tools(gateway: &McpGateway, id: Option<Value>) -> HttpResponse {
    let mut tools = Vec::new();
    let mut first_error = None;

    for (server, result) in gateway.list_all_tools().await {
        match result {
            Ok(list) => tools.extend(
                list.tools
                    .iter()
                    .map(|tool| prefixed_tool(&server, tool))
                    .collect::<Vec<_>>(),
            ),
            Err(error) => {
                warn!("MCP server '{}' tools/list failed: {}", server, error);
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
    }

    if tools.is_empty()
        && let Some(error) = first_error
    {
        return json_rpc_error(StatusCode::OK, JsonRpcError::from_mcp_error(&error), id);
    }

    tools.sort_by(|left, right| tool_name(left).cmp(tool_name(right)));
    success(json!({"tools": tools}), id)
}

async fn call_tool(gateway: &McpGateway, params: &Value, id: Option<Value>) -> HttpResponse {
    let Some(name) = params.get("name").and_then(Value::as_str) else {
        return json_rpc_error(
            StatusCode::OK,
            JsonRpcError::invalid_params().with_data(json!({"detail": "params.name is required"})),
            id,
        );
    };

    let arguments = params
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| json!({}));

    match gateway.call_prefixed_tool(name, arguments).await {
        Ok(result) => match serde_json::to_value(&result) {
            Ok(value) => success(value, id),
            Err(error) => internal_error(&error.to_string(), id),
        },
        Err(error) => json_rpc_error(StatusCode::OK, JsonRpcError::from_mcp_error(&error), id),
    }
}

/// Render one tool with its server-qualified name.
///
/// The prefix must match what [`McpGateway::call_prefixed_tool`] parses so a
/// listed name can be called back without translation.
fn prefixed_tool(server: &str, tool: &Tool) -> Value {
    let mut entry = Map::new();
    entry.insert(
        "name".to_string(),
        Value::String(format!("mcp_{server}__{}", tool.name)),
    );
    if let Some(description) = &tool.description {
        entry.insert(
            "description".to_string(),
            Value::String(description.clone()),
        );
    }
    entry.insert(
        "inputSchema".to_string(),
        tool.input_schema.to_json_schema(),
    );
    Value::Object(entry)
}

fn tool_name(tool: &Value) -> &str {
    tool.get("name").and_then(Value::as_str).unwrap_or_default()
}

fn success(result: Value, id: Option<Value>) -> HttpResponse {
    HttpResponse::Ok().json(JsonRpcResponse::success(result, id.unwrap_or(Value::Null)))
}

fn json_rpc_error(status: StatusCode, error: JsonRpcError, id: Option<Value>) -> HttpResponse {
    HttpResponse::build(status).json(JsonRpcResponse::error(error, id))
}

fn invalid_request(detail: &str, id: Option<Value>) -> HttpResponse {
    json_rpc_error(
        StatusCode::BAD_REQUEST,
        JsonRpcError::invalid_request().with_data(json!({"detail": detail})),
        id,
    )
}

fn internal_error(detail: &str, id: Option<Value>) -> HttpResponse {
    json_rpc_error(
        StatusCode::INTERNAL_SERVER_ERROR,
        JsonRpcError::internal_error().with_data(json!({"detail": detail})),
        id,
    )
}

/// Configure the MCP JSON-RPC endpoint.
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.route("/mcp", web::post().to(handle_jsonrpc))
        .route("/mcp/", web::post().to(handle_jsonrpc));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use crate::config::models::mcp::GatewayMcpServerConfig;
    use crate::core::mcp::tools::{PropertySchema, ToolInputSchema};
    use crate::server::HttpServer as GatewayHttpServer;
    use actix_web::{App, test as actix_test};

    fn base_config() -> Config {
        let mut config = Config::default();
        config.gateway.auth.enable_jwt = false;
        config.gateway.auth.enable_api_key = false;
        config.gateway.auth.allow_anonymous = true;
        config.gateway.storage.database.enabled = false;
        config.gateway.storage.redis.enabled = false;
        config.gateway.pricing.source = None;
        config
    }

    async fn state_for(config: &Config) -> AppState {
        match GatewayHttpServer::new(config).await {
            Ok(server) => server.state().clone(),
            Err(error) => panic!("gateway server should initialize for MCP route tests: {error}"),
        }
    }

    async fn state_without_mcp() -> AppState {
        state_for(&base_config()).await
    }

    async fn state_with_mcp() -> AppState {
        let mut config = base_config();
        config.gateway.mcp.servers.insert(
            "vertus_tools".to_string(),
            GatewayMcpServerConfig {
                url: "https://1.1.1.1/mcp".to_string(),
                ..GatewayMcpServerConfig::default()
            },
        );
        state_for(&config).await
    }

    async fn post_jsonrpc(state: AppState, payload: Value) -> (StatusCode, Value) {
        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(state))
                .configure(configure_routes),
        )
        .await;

        let request = actix_test::TestRequest::post()
            .uri("/mcp/")
            .set_json(&payload)
            .to_request();
        let response = actix_test::call_service(&app, request).await;
        let status = response.status();
        if status == StatusCode::ACCEPTED {
            return (status, Value::Null);
        }
        (status, actix_test::read_body_json(response).await)
    }

    #[tokio::test]
    async fn tools_list_without_configured_servers_returns_structured_error() {
        let (status, body) = post_jsonrpc(
            state_without_mcp().await,
            json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list"}),
        )
        .await;

        assert_eq!(status, StatusCode::NOT_IMPLEMENTED);
        assert_eq!(body["jsonrpc"], "2.0");
        assert_eq!(body["id"], 1);
        assert_eq!(body["error"]["code"], MCP_NOT_CONFIGURED_CODE);
        assert!(
            body["error"]["message"]
                .as_str()
                .unwrap_or_default()
                .contains("mcp.servers")
        );
        assert!(body.get("result").is_none());
    }

    #[tokio::test]
    async fn malformed_body_returns_parse_error() {
        let app = actix_test::init_service(
            App::new()
                .app_data(web::Data::new(state_without_mcp().await))
                .configure(configure_routes),
        )
        .await;

        let request = actix_test::TestRequest::post()
            .uri("/mcp/")
            .insert_header(("content-type", "application/json"))
            .set_payload("{not json")
            .to_request();
        let response = actix_test::call_service(&app, request).await;

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body: Value = actix_test::read_body_json(response).await;
        assert_eq!(body["error"]["code"], -32700);
    }

    #[tokio::test]
    async fn message_without_method_returns_invalid_request() {
        let (status, body) = post_jsonrpc(
            state_without_mcp().await,
            json!({"jsonrpc": "2.0", "id": 7}),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], -32600);
        assert_eq!(body["id"], 7);
    }

    #[tokio::test]
    async fn wrong_jsonrpc_version_returns_invalid_request() {
        let (status, body) = post_jsonrpc(
            state_without_mcp().await,
            json!({"jsonrpc": "1.0", "id": 1, "method": "tools/list"}),
        )
        .await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["error"]["code"], -32600);
    }

    #[tokio::test]
    async fn initialized_notification_is_accepted_without_body() {
        let (status, _body) = post_jsonrpc(
            state_without_mcp().await,
            json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
        )
        .await;

        assert_eq!(status, StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn initialize_reports_tools_capability_when_servers_are_configured() {
        let state = state_with_mcp().await;
        assert!(state.mcp_gateway.is_some());

        let (status, body) = post_jsonrpc(
            state,
            json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "initialize",
                "params": {
                    "protocolVersion": SUPPORTED_PROTOCOL_VERSION,
                    "capabilities": {},
                    "clientInfo": {"name": "test-client", "version": "1.0.0"}
                }
            }),
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(
            body["result"]["protocolVersion"],
            SUPPORTED_PROTOCOL_VERSION
        );
        assert_eq!(
            body["result"]["capabilities"]["tools"]["listChanged"],
            false
        );
        assert_eq!(body["result"]["serverInfo"]["name"], "litellm-rs");
    }

    #[tokio::test]
    async fn unknown_method_returns_method_not_found() {
        let (status, body) = post_jsonrpc(
            state_with_mcp().await,
            json!({"jsonrpc": "2.0", "id": 2, "method": "resources/list"}),
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["error"]["code"], -32601);
        assert_eq!(body["error"]["data"]["method"], "resources/list");
    }

    #[tokio::test]
    async fn call_tool_without_name_returns_invalid_params() {
        let (status, body) = post_jsonrpc(
            state_with_mcp().await,
            json!({"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {}}),
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["error"]["code"], -32602);
    }

    #[tokio::test]
    async fn call_tool_with_unprefixed_name_reports_tool_not_found() {
        let (status, body) = post_jsonrpc(
            state_with_mcp().await,
            json!({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": {"name": "search", "arguments": {}}
            }),
        )
        .await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["error"]["code"], -32001);
    }

    #[test]
    fn prefixed_tool_qualifies_name_and_keeps_schema() {
        let tool = Tool::new("search")
            .with_description("Search docs")
            .with_schema(ToolInputSchema::object().with_property(
                "query",
                PropertySchema::string(),
                true,
            ));

        let value = prefixed_tool("vertus_tools", &tool);

        assert_eq!(value["name"], "mcp_vertus_tools__search");
        assert_eq!(value["description"], "Search docs");
        assert_eq!(value["inputSchema"]["type"], "object");
        assert_eq!(value["inputSchema"]["required"][0], "query");
    }

    #[test]
    fn prefixed_tool_omits_absent_description() {
        let value = prefixed_tool("vertus_tools", &Tool::new("search"));
        assert!(value.get("description").is_none());
    }

    #[test]
    fn prefixed_tool_name_round_trips_through_gateway_parsing() {
        let value = prefixed_tool("vertus_tools", &Tool::new("search"));
        let name = value["name"].as_str().unwrap_or_default();

        assert!(name.starts_with("mcp_"));
        let (server, tool) = name
            .trim_start_matches("mcp_")
            .split_once("__")
            .unwrap_or_default();
        assert_eq!(server, "vertus_tools");
        assert_eq!(tool, "search");
    }
}
