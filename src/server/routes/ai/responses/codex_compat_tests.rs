use crate::core::models::openai::responses_api::{
    ResponseInputItem, ResponseTool, ResponsesApiRequest,
};
use crate::core::types::codex::wire::CODEX_PROTOCOL_BASELINE;
use actix_web::{body::to_bytes, http::StatusCode, test as actix_test, web};
use serde_json::{Value, json};
fn codex_request(value: Value) -> ResponsesApiRequest {
    serde_json::from_value(value).unwrap()
}
fn tier_two_items() -> [Value; 10] {
    [
        json!({"type":"additional_tools","role":"developer","tools":[]}),
        json!({"type":"local_shell_call","call_id":"c1","status":"completed","action":{}}),
        json!({"type":"mcp_tool_call_output","call_id":"c1","output":{"content":[]}}),
        json!({"type":"tool_search_call","call_id":"c1","status":"completed","execution":"client","arguments":{}}),
        json!({"type":"tool_search_output","call_id":"c1","status":"completed","execution":"client","tools":[]}),
        json!({"type":"web_search_call","id":"i1","status":"completed"}),
        json!({"type":"image_generation_call","id":"i1","status":"completed","result":"data"}),
        json!({"type":"compaction","id":"i1","encrypted_content":"opaque"}),
        json!({"type":"compaction_trigger"}),
        json!({"type":"context_compaction","id":"i1","encrypted_content":"opaque"}),
    ]
}
#[test]
fn codex_wire_round_trips_every_tier_one_field() {
    let input: Value = serde_json::from_str(r#"[
      {"type":"message","id":"msg_1","phase":"commentary","role":"user","content":"run"},
      {"type":"function_call","id":"fc_1","call_id":"c1","name":"lookup","namespace":"demo","arguments":"{}","status":"completed"},
      {"type":"function_call_output","id":"out_1","call_id":"c1","output":"ok"},
      {"type":"custom_tool_call","id":"ct_1","call_id":"c2","name":"shell","namespace":"tools","input":"pwd","status":"completed"},
      {"type":"custom_tool_call_output","id":"out_2","call_id":"c2","name":"shell","output":[{"type":"input_text","text":"/tmp"},{"type":"input_image","image_url":"image","detail":"high"},{"type":"input_audio","audio_url":"audio"},{"type":"encrypted_content","encrypted_content":"opaque"}]}
    ]"#).unwrap();
    assert_eq!(input.as_array().unwrap().len(), 5, "fixture count drifted");
    let encoded = serde_json::to_value(codex_request(json!({"model":"m","input":input}))).unwrap();
    assert_eq!(encoded["input"], input);
    assert_eq!(
        CODEX_PROTOCOL_BASELINE,
        "6e5a2d6b8d148a5554fdceb6f399ca45bd1c78d9"
    );
}
#[test]
fn codex_wire_handles_missing_null_and_empty_optional_fields() {
    for item in [
        json!({"type":"message","role":"user","content":""}),
        json!({"type":"message","id":null,"phase":null,"role":"user","content":"x"}),
        json!({"type":"custom_tool_call","id":"","call_id":"","name":"","namespace":"","input":""}),
    ] {
        let encoded =
            serde_json::to_value(serde_json::from_value::<ResponseInputItem>(item).unwrap())
                .unwrap();
        assert!(encoded["id"].is_null() || encoded["id"] == "");
    }
}
#[test]
fn codex_wire_accepts_flat_and_legacy_function_tools() {
    for value in [
        json!({"type":"function","name":"flat","parameters":{"type":"object"},"strict":true,"defer_loading":false}),
        json!({"type":"function","function":{"name":"nested"}}),
    ] {
        let expected = value.clone();
        let tool: ResponseTool = serde_json::from_value(value).unwrap();
        assert!(matches!(
            tool,
            ResponseTool::Function(_) | ResponseTool::CodexFunction(_)
        ));
        assert_eq!(serde_json::to_value(tool).unwrap(), expected);
    }
}
#[test]
fn codex_wire_distinguishes_tier_two_and_redacts_unknown_payload() {
    for value in tier_two_items() {
        assert!(matches!(
            serde_json::from_value::<ResponseInputItem>(value).unwrap(),
            ResponseInputItem::Unsupported(_)
        ));
    }
    let known: ResponseInputItem = serde_json::from_value(json!({"type":"local_shell_call","id":"i1","call_id":"c1","status":"completed","action":{"secret":"drop"}})).unwrap();
    assert_eq!(
        serde_json::to_value(known).unwrap(),
        json!({"type":"local_shell_call","id":"i1","call_id":"c1","status":"completed"})
    );
    let unknown: ResponseInputItem = serde_json::from_value(json!({"type":"future_item","id":"i2","namespace":"demo","secret":"drop","payload":{"token":"drop"}})).unwrap();
    assert!(matches!(unknown, ResponseInputItem::Unknown(_)));
    assert_eq!(
        serde_json::to_value(unknown).unwrap(),
        json!({"type":"future_item","id":"i2","namespace":"demo"})
    );
}
#[actix_web::test]
async fn codex_wire_http_rejects_before_provider_dispatch() {
    let mut config = crate::config::Config::default();
    config.gateway.storage.database.enabled = false;
    config.gateway.storage.redis.enabled = false;
    let server = crate::server::HttpServer::new(&config).await.unwrap();
    let state = web::Data::new(server.state().clone());
    super::PROVIDER_DISPATCH_COUNT.store(0, std::sync::atomic::Ordering::Relaxed);
    let mut fixtures = vec![
        json!({"model":"m","input":[{"type":"function_call","call_id":"c","name":"f","arguments":"{}"}]}),
        json!({"model":"m","input":[{"type":"future\nsecret=abcdefghijklmnop","secret":"drop"}]}),
        json!({"model":"m","input":"x","additional_tools":[{"type":"function","name":"f"}]}),
    ];
    fixtures.extend(tier_two_items().map(|item| json!({"model":"m","input":[item]})));
    for tool in [
        json!({"type":"custom","name":"shell","description":"d","format":{}}),
        json!({"type":"function","name":"f","defer_loading":true}),
        json!({"type":"namespace"}),
        json!({"type":"tool_search"}),
        json!({"type":"image_generation"}),
        json!({"type":"web_search"}),
        json!({"type":"file_search"}),
        json!({"type":"code_interpreter"}),
        json!({"type":"computer_use_preview","display_width":1,"display_height":1,"environment":"browser"}),
        json!({"type":"mcp","server_label":"s","server_url":"https://example.com"}),
    ] {
        assert!(!matches!(
            serde_json::from_value::<ResponseTool>(tool.clone()).unwrap(),
            ResponseTool::Unknown(_)
        ));
        fixtures.push(json!({"model":"m","input":"x","tools":[tool]}));
    }
    for value in fixtures {
        let mut payload = codex_request(value);
        payload.store = Some(false);
        let req = actix_test::TestRequest::post()
            .insert_header(("x-codex-upstream-counter", "1"))
            .to_http_request();
        let response = super::create_response(state.clone(), req, web::Json(payload))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body: Value =
            serde_json::from_slice(&to_bytes(response.into_body()).await.unwrap()).unwrap();
        assert_eq!(body["error"]["type"], "invalid_request_error");
        assert_eq!(body["error"]["code"], "unsupported_codex_feature");
        assert!(
            body["error"]["message"]
                .as_str()
                .unwrap()
                .contains("provider=unselected")
        );
        assert!(!body.to_string().contains("abcdefghijklmnop"));
    }
    assert_eq!(
        super::PROVIDER_DISPATCH_COUNT.load(std::sync::atomic::Ordering::Relaxed),
        0
    );
}
