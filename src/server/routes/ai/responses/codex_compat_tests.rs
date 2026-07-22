use serde_json::{Value, json};

use super::build_chat_request;
use crate::core::models::openai::responses_api::{
    ResponseInputItem, ResponseOutputItem, ResponseTool, ResponsesApiRequest,
};
use crate::core::types::codex::CODEX_PROTOCOL_BASELINE;

#[test]
fn codex_wire_round_trips_tier_one_items() {
    let value = json!({
        "model": "test-model",
        "input": [
            {"type":"message","id":"msg_1","role":"user","content":"run it"},
            {"type":"function_call","id":"fc_1","call_id":"call_1","name":"lookup","namespace":"demo","arguments":"{}","status":"completed"},
            {"type":"function_call_output","call_id":"call_1","output":"ok"},
            {"type":"custom_tool_call","id":"ct_1","call_id":"call_2","name":"shell","input":"pwd","status":"completed"},
            {"type":"custom_tool_call_output","call_id":"call_2","name":"shell","output":[{"type":"input_text","text":"/tmp"}]}
        ]
    });
    let request: ResponsesApiRequest = serde_json::from_value(value.clone()).unwrap();
    let encoded = serde_json::to_value(request).unwrap();
    assert_eq!(encoded["input"], value["input"]);
}

#[test]
fn codex_wire_accepts_flat_and_legacy_function_tools() {
    let flat: ResponseTool = serde_json::from_value(json!({
        "type":"function","name":"flat","description":"flat tool",
        "parameters":{"type":"object"},"strict":true
    }))
    .unwrap();
    let nested: ResponseTool = serde_json::from_value(json!({
        "type":"function","function":{"name":"nested"}
    }))
    .unwrap();
    assert!(matches!(flat, ResponseTool::Function(_)));
    assert!(matches!(nested, ResponseTool::Function(_)));
}

#[test]
fn codex_wire_preserves_custom_tool_and_unknown_types() {
    let custom: ResponseTool = serde_json::from_value(json!({
        "type":"custom","name":"shell","description":"run command",
        "format":{"type":"grammar","syntax":"lark","definition":"start: /.+/"}
    }))
    .unwrap();
    assert!(matches!(custom, ResponseTool::Custom(_)));

    let unknown: ResponseInputItem =
        serde_json::from_value(json!({"type":"future_item","secret":"kept"})).unwrap();
    assert_eq!(unknown.feature_name(), "future_item");
    assert_eq!(serde_json::to_value(unknown).unwrap()["secret"], "kept");
}

#[test]
fn codex_wire_rejects_tier_two_before_provider_conversion() {
    let request: ResponsesApiRequest = serde_json::from_value(json!({
        "model":"test-model",
        "input":[{"type":"local_shell_call","call_id":"call_1","status":"completed","action":{}}]
    }))
    .unwrap();
    assert_eq!(
        build_chat_request(&request).unwrap_err(),
        "unsupported Codex feature: local_shell_call"
    );
}

#[test]
fn codex_wire_round_trips_custom_output_item() {
    let value = json!({
        "type":"custom_tool_call","id":"ct_1","call_id":"call_1",
        "name":"shell","input":"pwd","status":"completed"
    });
    let item: ResponseOutputItem = serde_json::from_value(value.clone()).unwrap();
    assert!(matches!(item, ResponseOutputItem::CustomToolCall(_)));
    assert_eq!(serde_json::to_value(item).unwrap(), value);
}

#[test]
fn codex_wire_fixture_pins_protocol_source() {
    assert_eq!(
        CODEX_PROTOCOL_BASELINE,
        "6e5a2d6b8d148a5554fdceb6f399ca45bd1c78d9"
    );
    let _: Value = json!({"source": CODEX_PROTOCOL_BASELINE});
}
