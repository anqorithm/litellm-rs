use super::{build_chat_request, openai_errors, unsupported_codex_feature};
use crate::core::models::openai::responses_api::{
    ResponseInputItem, ResponseTool, ResponsesApiRequest,
};
use crate::core::types::codex::wire::CODEX_PROTOCOL_BASELINE;
use actix_web::{body::to_bytes, http::StatusCode};
use serde_json::{Value, json};
fn codex_request(value: Value) -> ResponsesApiRequest {
    serde_json::from_value(value).unwrap()
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
fn codex_wire_distinguishes_tier_two_and_redacts_unknown_payload() {
    let known: ResponseInputItem = serde_json::from_value(json!({
        "type":"local_shell_call","id":"i1","call_id":"c1","status":"completed","action":{"secret":"drop"}
    })).unwrap();
    assert!(matches!(known, ResponseInputItem::Unsupported(_)));
    assert_eq!(
        serde_json::to_value(known).unwrap(),
        json!({
            "type":"local_shell_call","id":"i1","call_id":"c1","status":"completed"
        })
    );
    let unknown: ResponseInputItem = serde_json::from_value(json!({
        "type":"future_item","id":"i2","namespace":"demo","secret":"drop","payload":{"token":"drop"}
    }))
    .unwrap();
    assert!(matches!(unknown, ResponseInputItem::Unknown(_)));
    assert_eq!(
        serde_json::to_value(unknown).unwrap(),
        json!({
            "type":"future_item","id":"i2","namespace":"demo"
        })
    );
}
#[test]
fn codex_wire_accepts_flat_and_legacy_function_tools() {
    for value in [
        json!({"type":"function","name":"flat","parameters":{"type":"object"},"strict":true}),
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
fn codex_wire_fails_closed_before_provider_conversion() {
    for value in [
        json!({"model":"m","input":[{"type":"function_call","call_id":"c","name":"f","arguments":"{}"}]}),
        json!({"model":"m","input":"x","tools":[{"type":"custom","name":"shell","description":"d","format":{}}]}),
        json!({"model":"m","input":"x","additional_tools":[{"type":"function","name":"f"}]}),
    ] {
        let request = codex_request(value);
        assert!(unsupported_codex_feature(&request).is_some());
        assert!(build_chat_request(&request).is_err());
    }
}
#[actix_web::test]
async fn codex_wire_returns_stable_redacted_http_error() {
    let response = openai_errors::unsupported_codex_feature("future\nsecret=abcdefghijklmnop", "m");
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let bytes = to_bytes(response.into_body()).await.unwrap();
    let body: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(body["error"]["type"], "invalid_request_error");
    assert_eq!(body["error"]["code"], "unsupported_codex_feature");
    assert_eq!(
        body["error"]["message"],
        "unsupported Codex feature: future_secret: [REDACTED]; model=m; provider=unselected"
    );
    assert!(!body.to_string().contains("abcdefghijklmnop"));
}
