//! OpenAI Request and Response Transformers
//!
//! Unified transformation layer for converting between unified LiteLLM types and OpenAI-specific formats

use crate::core::traits::transformer::Transform;
use crate::core::types::responses::{
    ChatChoice, ChatChunk, ChatDelta, ChatResponse, ChatStreamChoice, FinishReason, LogProbs,
    TokenLogProb, TopLogProb, Usage,
};
use crate::core::types::thinking::ThinkingContent;
use crate::core::types::{
    chat::ChatMessage, chat::ChatRequest, content::ContentPart, content::ImageUrl, message::MessageContent,
    message::MessageRole, tools::FunctionCall, tools::ResponseFormat, tools::Tool, tools::ToolCall,
    tools::ToolChoice,
};
use serde_json;

use super::error::OpenAIError;
use super::models::*;

/// OpenAI Request Transformer
pub struct OpenAIRequestTransformer;

impl OpenAIRequestTransformer {
    /// Transform ChatRequest to OpenAIChatRequest
    pub fn transform(request: ChatRequest) -> Result<OpenAIChatRequest, OpenAIError> {
        let messages = request
            .messages
            .into_iter()
            .map(Self::transform_message)
            .collect::<Result<Vec<_>, _>>()?;

        let tools = request
            .tools
            .map(|tools| tools.into_iter().map(Self::transform_tool).collect());

        let tool_choice = request
            .tool_choice
            .map(Self::transform_tool_choice)
            .and_then(|tc| serde_json::to_value(tc).ok());

        let response_format = request.response_format.map(Self::transform_response_format);

        Ok(OpenAIChatRequest {
            model: request.model,
            messages,
            temperature: request.temperature,
            top_p: request.top_p,
            n: request.n,
            stream: None, // Set by caller
            stop: request.stop,
            max_tokens: request.max_tokens,
            max_completion_tokens: request.max_completion_tokens,
            presence_penalty: request.presence_penalty,
            frequency_penalty: request.frequency_penalty,
            logit_bias: request.logit_bias,
            logprobs: request.logprobs,
            top_logprobs: request.top_logprobs,
            user: request.user,
            tools,
            tool_choice,
            parallel_tool_calls: request.parallel_tool_calls,
            response_format,
            seed: request.seed,
        })
    }

    /// Transform Message
    fn transform_message(message: ChatMessage) -> Result<OpenAIMessage, OpenAIError> {
        let role = match message.role {
            MessageRole::System => "system",
            MessageRole::User => "user",
            MessageRole::Assistant => "assistant",
            MessageRole::Tool => "tool",
            MessageRole::Function => "function",
        }
        .to_string();

        let content = match message.content {
            Some(MessageContent::Text(text)) => Some(serde_json::json!(text)),
            Some(MessageContent::Parts(parts)) => {
                let openai_parts = parts
                    .into_iter()
                    .map(Self::transform_content_part)
                    .collect::<Result<Vec<_>, _>>()?;
                Some(serde_json::to_value(openai_parts).map_err(|e| {
                    OpenAIError::Serialization {
                        provider: "openai",
                        message: format!("Failed to serialize content parts: {}", e),
                    }
                })?)
            }
            None => None,
        };

        Ok(OpenAIMessage {
            role,
            content,
            name: message.name,
            tool_calls: message
                .tool_calls
                .map(|calls| calls.into_iter().map(Self::transform_tool_call).collect()),
            tool_call_id: message.tool_call_id,
            function_call: message
                .function_call
                .map(Self::transform_function_call_response),
            reasoning: None,
            reasoning_details: None,
            reasoning_content: None,
        })
    }

    /// Transform content part
    fn transform_content_part(part: ContentPart) -> Result<OpenAIContentPart, OpenAIError> {
        match part {
            ContentPart::Text { text } => Ok(OpenAIContentPart::Text { text }),
            ContentPart::ImageUrl { image_url } => Ok(OpenAIContentPart::ImageUrl {
                image_url: OpenAIImageUrl {
                    url: image_url.url,
                    detail: image_url.detail,
                },
            }),
            ContentPart::Audio { audio } => Ok(OpenAIContentPart::InputAudio {
                input_audio: OpenAIInputAudio {
                    data: audio.data,
                    format: audio.format.unwrap_or("mp3".to_string()),
                },
            }),
            ContentPart::Image {
                source,
                detail,
                image_url,
            } => Ok(OpenAIContentPart::ImageUrl {
                image_url: image_url
                    .map(|img_url| OpenAIImageUrl {
                        url: img_url.url,
                        detail: img_url.detail,
                    })
                    .unwrap_or(OpenAIImageUrl {
                        url: format!("data:{};base64,{}", source.media_type, source.data),
                        detail: detail.clone(),
                    }),
            }),
            // Handle new content types
            ContentPart::Document { .. } => Err(OpenAIError::InvalidRequest {
                provider: "openai",
                message: "Document content not supported by OpenAI".to_string(),
            }),
            ContentPart::ToolResult { .. } => Err(OpenAIError::InvalidRequest {
                provider: "openai",
                message: "ToolResult should be handled separately".to_string(),
            }),
            ContentPart::ToolUse { .. } => Err(OpenAIError::InvalidRequest {
                provider: "openai",
                message: "ToolUse should be handled separately".to_string(),
            }),
        }
    }

    /// Transform tool call
    fn transform_tool_call(tool_call: ToolCall) -> OpenAIToolCall {
        OpenAIToolCall {
            id: tool_call.id,
            tool_type: "function".to_string(),
            function: OpenAIFunctionCall {
                name: tool_call.function.name,
                arguments: tool_call.function.arguments,
            },
        }
    }

    /// Transform function call response
    fn transform_function_call_response(function_call: FunctionCall) -> OpenAIFunctionCall {
        OpenAIFunctionCall {
            name: function_call.name,
            arguments: function_call.arguments,
        }
    }

    /// Transform tool definition
    fn transform_tool(tool: Tool) -> OpenAITool {
        OpenAITool {
            tool_type: "function".to_string(),
            function: Some(OpenAIFunction {
                name: tool.function.name,
                description: tool.function.description,
                parameters: tool.function.parameters,
            }),
        }
    }

    /// Transform tool choice
    fn transform_tool_choice(choice: ToolChoice) -> OpenAIToolChoice {
        match choice {
            ToolChoice::String(s) => match s.as_str() {
                "none" => OpenAIToolChoice::none(),
                "auto" => OpenAIToolChoice::auto(),
                "required" => OpenAIToolChoice::required(),
                _ => OpenAIToolChoice::auto(),
            },
            ToolChoice::Specific {
                choice_type,
                function,
            } => {
                if choice_type == "function" {
                    if let Some(func) = function {
                        OpenAIToolChoice::Function {
                            r#type: "function".to_string(),
                            function: OpenAIFunctionChoice { name: func.name },
                        }
                    } else {
                        OpenAIToolChoice::auto()
                    }
                } else {
                    OpenAIToolChoice::auto()
                }
            }
        }
    }

    /// Transform response format
    fn transform_response_format(format: ResponseFormat) -> OpenAIResponseFormat {
        OpenAIResponseFormat {
            format_type: format.format_type,
            json_schema: format.json_schema,
        }
    }
}

/// OpenAI Response Transformer
pub struct OpenAIResponseTransformer;

impl OpenAIResponseTransformer {
    /// Transform OpenAIChatResponse to ChatResponse
    pub fn transform(response: OpenAIChatResponse) -> Result<ChatResponse, OpenAIError> {
        let choices = response
            .choices
            .into_iter()
            .map(Self::transform_choice)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ChatResponse {
            id: response.id,
            object: response.object,
            created: response.created,
            model: response.model,
            choices,
            usage: response.usage.map(Self::transform_usage),
            system_fingerprint: response.system_fingerprint,
        })
    }

    /// Transform stream chunk
    pub fn transform_stream_chunk(chunk: OpenAIStreamChunk) -> Result<ChatChunk, OpenAIError> {
        let choices = chunk
            .choices
            .into_iter()
            .map(Self::transform_stream_choice)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(ChatChunk {
            id: chunk.id,
            object: chunk.object,
            created: chunk.created,
            model: chunk.model,
            choices,
            usage: chunk.usage.map(Self::transform_usage),
            system_fingerprint: chunk.system_fingerprint,
        })
    }

    /// Transform choice
    fn transform_choice(choice: OpenAIChoice) -> Result<ChatChoice, OpenAIError> {
        Ok(ChatChoice {
            index: choice.index,
            message: Self::transform_message_response(choice.message)?,
            logprobs: choice.logprobs.and_then(|lp| {
                serde_json::from_value::<OpenAILogprobs>(lp)
                    .ok()
                    .map(Self::transform_logprobs)
            }),
            finish_reason: choice.finish_reason.map(Self::transform_finish_reason),
        })
    }

    /// Transform stream choice
    fn transform_stream_choice(
        choice: OpenAIStreamChoice,
    ) -> Result<ChatStreamChoice, OpenAIError> {
        Ok(ChatStreamChoice {
            index: choice.index,
            delta: Self::transform_delta(choice.delta)?,
            logprobs: choice.logprobs.and_then(|lp| {
                serde_json::from_value::<OpenAILogprobs>(lp)
                    .ok()
                    .map(Self::transform_logprobs)
            }),
            finish_reason: choice.finish_reason.map(Self::transform_finish_reason),
        })
    }

    /// Transform message response
    fn transform_message_response(message: OpenAIMessage) -> Result<ChatMessage, OpenAIError> {
        let role = match message.role.as_str() {
            "system" => MessageRole::System,
            "user" => MessageRole::User,
            "assistant" => MessageRole::Assistant,
            "tool" => MessageRole::Tool,
            "function" => MessageRole::Function,
            _ => MessageRole::User,
        };

        // Extract thinking content from reasoning fields
        // Priority: reasoning_content (DeepSeek) > reasoning (OpenAI)
        let thinking = message
            .reasoning_content
            .as_ref()
            .filter(|s| !s.is_empty())
            .or(message.reasoning.as_ref().filter(|s| !s.is_empty()))
            .map(|text| ThinkingContent::Text {
                text: text.clone(),
                signature: None,
            });

        // Parse content (don't include reasoning in content anymore)
        let content = match message.content {
            Some(value) => {
                if value.is_null() {
                    None
                } else if let Some(text) = value.as_str() {
                    if text.is_empty() {
                        None
                    } else {
                        Some(MessageContent::Text(text.to_string()))
                    }
                } else if let Some(array) = value.as_array() {
                    let parts: Vec<OpenAIContentPart> = serde_json::from_value(
                        serde_json::Value::Array(array.clone()),
                    )
                    .map_err(|e| OpenAIError::ResponseParsing {
                        provider: "openai",
                        message: format!("Failed to parse content parts: {}", e),
                    })?;
                    let content_parts = parts
                        .into_iter()
                        .map(Self::transform_content_part_response)
                        .collect::<Result<Vec<_>, _>>()?;
                    Some(MessageContent::Parts(content_parts))
                } else {
                    None
                }
            }
            None => None,
        };

        Ok(ChatMessage {
            role,
            content,
            thinking,
            name: message.name,
            tool_calls: message.tool_calls.map(|calls| {
                calls
                    .into_iter()
                    .map(Self::transform_tool_call_response)
                    .collect()
            }),
            tool_call_id: message.tool_call_id,
            function_call: message
                .function_call
                .map(Self::transform_function_call_from_response),
        })
    }

    /// Transform delta
    fn transform_delta(delta: OpenAIDelta) -> Result<ChatDelta, OpenAIError> {
        Ok(ChatDelta {
            role: delta.role.map(|r| match r.as_str() {
                "system" => MessageRole::System,
                "user" => MessageRole::User,
                "assistant" => MessageRole::Assistant,
                "tool" => MessageRole::Tool,
                "function" => MessageRole::Function,
                _ => MessageRole::Assistant,
            }),
            content: delta.content,
            thinking: None,
            tool_calls: None,
            function_call: None,
        })
    }

    /// Transform content part response
    fn transform_content_part_response(
        part: OpenAIContentPart,
    ) -> Result<ContentPart, OpenAIError> {
        match part {
            OpenAIContentPart::Text { text } => Ok(ContentPart::Text { text }),
            OpenAIContentPart::ImageUrl { image_url } => Ok(ContentPart::ImageUrl {
                image_url: ImageUrl {
                    url: image_url.url,
                    detail: image_url.detail,
                },
            }),
            OpenAIContentPart::InputAudio { input_audio } => Ok(ContentPart::Audio {
                audio: crate::core::types::content::AudioData {
                    data: input_audio.data,
                    format: Some(input_audio.format),
                },
            }),
        }
    }

    /// Transform tool call response
    fn transform_tool_call_response(tool_call: OpenAIToolCall) -> ToolCall {
        ToolCall {
            id: tool_call.id,
            tool_type: "function".to_string(),
            function: FunctionCall {
                name: tool_call.function.name,
                arguments: tool_call.function.arguments,
            },
        }
    }

    /// Transform function call from response
    fn transform_function_call_from_response(function_call: OpenAIFunctionCall) -> FunctionCall {
        FunctionCall {
            name: function_call.name,
            arguments: function_call.arguments,
        }
    }

    /// Transform usage
    fn transform_usage(usage: OpenAIUsage) -> Usage {
        Usage {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
            thinking_usage: None,
            prompt_tokens_details: usage.prompt_tokens_details.map(|details| {
                crate::core::types::responses::PromptTokensDetails {
                    cached_tokens: details.cached_tokens,
                    audio_tokens: details.audio_tokens,
                }
            }),
            completion_tokens_details: usage.completion_tokens_details.map(|details| {
                crate::core::types::responses::CompletionTokensDetails {
                    reasoning_tokens: details.reasoning_tokens,
                    audio_tokens: details.audio_tokens,
                }
            }),
        }
    }

    /// Transform logprobs
    fn transform_logprobs(logprobs: OpenAILogprobs) -> LogProbs {
        LogProbs {
            content: logprobs
                .content
                .map(|content| {
                    content
                        .into_iter()
                        .map(|token| TokenLogProb {
                            token: token.token,
                            logprob: token.logprob,
                            bytes: token.bytes,
                            top_logprobs: Some(
                                token
                                    .top_logprobs
                                    .into_iter()
                                    .map(|top| TopLogProb {
                                        token: top.token,
                                        logprob: top.logprob,
                                        bytes: top.bytes,
                                    })
                                    .collect(),
                            ),
                        })
                        .collect()
                })
                .unwrap_or_default(),
            refusal: logprobs.refusal.map(|_| "filtered".to_string()),
        }
    }

    /// Transform finish reason
    fn transform_finish_reason(reason: String) -> FinishReason {
        match reason.as_str() {
            "stop" => FinishReason::Stop,
            "length" => FinishReason::Length,
            "function_call" => FinishReason::FunctionCall,
            "tool_calls" => FinishReason::ToolCalls,
            "content_filter" => FinishReason::ContentFilter,
            _ => FinishReason::Stop,
        }
    }
}

/// OpenAI Transformer (compatible with old interface)
pub struct OpenAITransformer;

impl Transform<ChatRequest, OpenAIChatRequest> for OpenAITransformer {
    type Error = OpenAIError;

    fn transform(input: ChatRequest) -> Result<OpenAIChatRequest, Self::Error> {
        OpenAIRequestTransformer::transform(input)
    }
}

impl Transform<OpenAIChatResponse, ChatResponse> for OpenAITransformer {
    type Error = OpenAIError;

    fn transform(input: OpenAIChatResponse) -> Result<ChatResponse, Self::Error> {
        OpenAIResponseTransformer::transform(input)
    }
}


#[cfg(test)]
#[path = "transformer_tests.rs"]
mod tests;
