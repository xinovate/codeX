//! Chat Completions API endpoint client.
//!
//! This client transparently converts between the Responses API protocol
//! (used internally by Codex) and the Chat Completions API protocol
//! (used by Chinese providers like Volcengine, Doubao, Kimi).
//!
//! Request flow:  ResponsesApiRequest → Chat Completions body → HTTP POST
//! Response flow: Chat SSE chunks → Responses API events → ResponseStream

use crate::auth::SharedAuthProvider;
use crate::common::ResponseEvent;
use crate::common::ResponseStream;
use crate::common::ResponsesApiRequest;
use crate::endpoint::session::EndpointSession;
use crate::error::ApiError;
use crate::provider::Provider;
use crate::requests::Compression;
use crate::telemetry::SseTelemetry;
use codex_client::HttpTransport;
use codex_client::RequestCompression;
use codex_client::RequestTelemetry;
use codex_protocol::protocol::TokenUsage;
use http::HeaderMap;
use http::HeaderValue;
use http::Method;
use serde_json::{Map, Value, json};
use std::sync::Arc;
use std::sync::OnceLock;
use tracing::instrument;

/// Client for Chinese Chat Completions API providers.
///
/// Wraps `EndpointSession` and performs protocol conversion:
/// - Converts Responses API requests to Chat Completions format
/// - Converts Chat Completions SSE responses to Responses API events
pub struct ChatCompletionsClient<T: HttpTransport> {
    session: EndpointSession<T>,
}

impl<T: HttpTransport> ChatCompletionsClient<T> {
    pub fn new(transport: T, provider: Provider, auth: SharedAuthProvider) -> Self {
        Self {
            session: EndpointSession::new(transport, provider, auth),
        }
    }

    pub fn with_telemetry(
        self,
        request: Option<Arc<dyn RequestTelemetry>>,
        _sse: Option<Arc<dyn SseTelemetry>>,
    ) -> Self {
        Self {
            session: self.session.with_request_telemetry(request),
        }
    }

    /// Path for Chat Completions API.
    fn path() -> &'static str {
        "chat/completions"
    }

    /// Stream a request using the Chat Completions API with protocol conversion.
    #[instrument(
        name = "chat_completions.stream_request",
        level = "info",
        skip_all,
        fields(
            transport = "chat_completions_http",
            http.method = "POST",
            api.path = "chat/completions"
        )
    )]
    pub async fn stream_request(
        &self,
        request: ResponsesApiRequest,
        extra_headers: HeaderMap,
        compression: Compression,
        turn_state: Option<Arc<OnceLock<String>>>,
    ) -> Result<ResponseStream, ApiError> {
        // 1. Serialize the Responses API request
        let mut body = serde_json::to_value(&request)
            .map_err(|e| ApiError::Stream(format!("failed to encode request: {e}")))?;

        // 2. Convert Responses → Chat Completions request body
        convert_request_body(&mut body);

        let request_compression = match compression {
            Compression::None => RequestCompression::None,
            Compression::Zstd => RequestCompression::Zstd,
        };

        let mut headers = extra_headers;
        headers.insert(
            http::header::ACCEPT,
            HeaderValue::from_static("text/event-stream"),
        );

        // 3. Send to chat/completions endpoint
        let stream_response = self
            .session
            .stream_with(
                Method::POST,
                Self::path(),
                headers,
                Some(body),
                |req| {
                    req.compression = request_compression;
                },
            )
            .await?;

        // 4. Parse SSE with Chat→Responses conversion
        let idle_timeout = self.session.provider().stream_idle_timeout;
        Ok(spawn_chat_completions_stream(
            stream_response,
            idle_timeout,
            turn_state,
        ))
    }
}

/// Spawn a task that reads Chat Completions SSE chunks, converts them to
/// Responses API events, and sends them through a channel.
fn spawn_chat_completions_stream(
    stream_response: codex_client::StreamResponse,
    idle_timeout: std::time::Duration,
    _turn_state: Option<Arc<OnceLock<String>>>,
) -> ResponseStream {
    use codex_client::ByteStream;
    use eventsource_stream::Eventsource;
    use futures::StreamExt;
    use tokio::sync::mpsc;
    use tokio::time::timeout;

    let (tx_event, rx_event) = mpsc::channel::<Result<ResponseEvent, ApiError>>(1600);

    tokio::spawn(async move {
        let stream: ByteStream = stream_response.bytes;
        let mut event_stream = stream.eventsource();
        let mut response_id: Option<String> = None;
        let mut accumulated_text = String::new();
        let mut item_added_sent = false;
        let mut _last_model: Option<String> = None;

        // Extract model from response headers if available
        if let Some(model) = stream_response
            .headers
            .get("openai-model")
            .and_then(|v| v.to_str().ok())
        {
            let _ = tx_event
                .send(Ok(ResponseEvent::ServerModel(model.to_string())))
                .await;
            _last_model = Some(model.to_string());
        }

        loop {
            let result = timeout(idle_timeout, event_stream.next()).await;
            let sse = match result {
                Ok(Some(Ok(sse))) => sse,
                Ok(Some(Err(e))) => {
                    let _ = tx_event
                        .send(Err(ApiError::Stream(format!("SSE error: {e}"))))
                        .await;
                    return;
                }
                Ok(None) => {
                    // Stream ended — if we haven't sent completed, send it now
                    let _ = tx_event
                        .send(Err(ApiError::Stream(
                            "stream closed before response.completed".into(),
                        )))
                        .await;
                    return;
                }
                Err(_) => {
                    let _ = tx_event
                        .send(Err(ApiError::Stream(
                            "idle timeout waiting for SSE".into(),
                        )))
                        .await;
                    return;
                }
            };

            let data = sse.data.trim();
            if data == "[DONE]" {
                // End of stream — emit response.completed if not already sent
                let resp_id = response_id.unwrap_or_default();
                let _ = tx_event
                    .send(Ok(ResponseEvent::Completed {
                        response_id: resp_id,
                        token_usage: None,
                    }))
                    .await;
                return;
            }

            // Parse the Chat Completions chunk
            let chat_chunk: Value = match serde_json::from_str(data) {
                Ok(v) => v,
                Err(e) => {
                    tracing::debug!("Failed to parse Chat SSE chunk: {e}, data: {data}");
                    continue;
                }
            };

            // Track response ID
            if let Some(id) = chat_chunk.get("id").and_then(|v| v.as_str()) {
                response_id = Some(id.to_string());
            }

            // Process choices
            if let Some(choices) = chat_chunk
                .get("choices")
                .and_then(|v| v.as_array())
                .and_then(|a| a.first())
            {
                if let Some(delta) = choices.get("delta") {
                    // Role → output_item.added
                    if let Some(role) = delta.get("role").and_then(|v| v.as_str()) {
                        if !item_added_sent {
                            let _ = tx_event
                                .send(Ok(ResponseEvent::OutputItemAdded(
                                    codex_protocol::models::ResponseItem::Message {
                                        id: None,
                                        role: role.to_string(),
                                        content: vec![],
                                        end_turn: None,
                                        phase: None,
                                    },
                                )))
                                .await;
                            item_added_sent = true;
                        }
                    }

                    // Content → output_text.delta
                    if let Some(content) = delta.get("content").and_then(|v| v.as_str()) {
                        if !content.is_empty() {
                            accumulated_text.push_str(content);
                            let _ = tx_event
                                .send(Ok(ResponseEvent::OutputTextDelta(
                                    content.to_string(),
                                )))
                                .await;
                        }
                    }

                    // Reasoning content → reasoning_text.delta (Volcengine-specific)
                    if let Some(reasoning) =
                        delta.get("reasoning_content").and_then(|v| v.as_str())
                    {
                        if !reasoning.is_empty() {
                            let _ = tx_event
                                .send(Ok(ResponseEvent::ReasoningContentDelta {
                                    delta: reasoning.to_string(),
                                    content_index: 0,
                                }))
                                .await;
                        }
                    }

                    // Tool calls delta → function_call_arguments.delta
                    // TODO: Full tool call streaming support requires emitting
                    // OutputItemAdded(FunctionCall) before the first delta, and
                    // OutputItemDone(FunctionCall) after the last delta.
                    // For now, we emit ToolCallInputDelta which the core session
                    // can handle if an active_tool_argument_diff_consumer is set.
                    if let Some(tool_calls) = delta.get("tool_calls") {
                        if let Some(arr) = tool_calls.as_array() {
                            for tc in arr {
                                if let Some(index) = tc.get("index").and_then(|v| v.as_i64()) {
                                    let _ = tx_event
                                        .send(Ok(ResponseEvent::ToolCallInputDelta {
                                            item_id: format!("tool_call_{index}"),
                                            call_id: tc.get("id")
                                                .and_then(|v| v.as_str())
                                                .map(String::from),
                                            delta: tc.get("function")
                                                .and_then(|f| f.get("arguments"))
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("")
                                                .to_string(),
                                        }))
                                        .await;
                                }
                            }
                        }
                    }
                }

                // Finish reason → output_item.done + response.completed
                if let Some(finish_reason) = choices
                    .get("finish_reason")
                    .and_then(|v| v.as_str())
                {
                    let _status = match finish_reason {
                        "stop" => "completed",
                        "length" => "incomplete",
                        "tool_calls" => "requires_action",
                        _ => "completed",
                    };

                    // Check if this is a tool_calls response with complete tool_calls info
                    if finish_reason == "tool_calls" {
                        if let Some(message) = choices.get("message") {
                            if let Some(tool_calls) = message.get("tool_calls") {
                                if let Some(arr) = tool_calls.as_array() {
                                    for tc in arr {
                                        let id = tc.get("id")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        let name = tc.get("function")
                                            .and_then(|f| f.get("name"))
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        let arguments = tc.get("function")
                                            .and_then(|f| f.get("arguments"))
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();

                                        let _ = tx_event
                                            .send(Ok(ResponseEvent::OutputItemDone(
                                                codex_protocol::models::ResponseItem::FunctionCall {
                                                    id: None,
                                                    name,
                                                    namespace: None,
                                                    arguments,
                                                    call_id: id,
                                                },
                                            )))
                                            .await;
                                    }
                                }
                            }
                        }
                    } else {
                        // Emit output_item.done for regular message
                        let _ = tx_event
                            .send(Ok(ResponseEvent::OutputItemDone(
                                codex_protocol::models::ResponseItem::Message {
                                    id: None,
                                    role: "assistant".to_string(),
                                    content: vec![codex_protocol::models::ContentItem::OutputText {
                                        text: accumulated_text.clone(),
                                    }],
                                    end_turn: None,
                                    phase: None,
                                },
                            )))
                            .await;
                    }

                    // Extract usage from the chunk if available
                    let token_usage = chat_chunk.get("usage").and_then(|usage| {
                        if usage.is_null() || !usage.is_object() {
                            return None;
                        }
                        Some(TokenUsage {
                            input_tokens: usage
                                .get("prompt_tokens")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0),
                            cached_input_tokens: 0,
                            output_tokens: usage
                                .get("completion_tokens")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0),
                            reasoning_output_tokens: usage
                                .get("completion_tokens_details")
                                .and_then(|d| d.get("reasoning_tokens"))
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0),
                            total_tokens: usage
                                .get("total_tokens")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0),
                        })
                    });

                    // Emit response.completed
                    let _ = tx_event
                        .send(Ok(ResponseEvent::Completed {
                            response_id: response_id.clone().unwrap_or_default(),
                            token_usage,
                        }))
                        .await;

                    return;
                }
            }
        }
    });

    ResponseStream { rx_event }
}

// ============================================================
// Protocol conversion: Responses API → Chat Completions API
// ============================================================

/// Convert Responses API content item types to Chat Completions API types.
///
/// - `input_text` → `text`
/// - `output_text` → `text`
/// - `input_image` → `image_url`
fn convert_content_types(content: &mut Value) {
    if let Some(arr) = content.as_array_mut() {
        for item in arr {
            if let Some(obj) = item.as_object_mut() {
                if let Some(t) = obj.get_mut("type") {
                    match t.as_str() {
                        Some("input_text" | "output_text") => {
                            *t = Value::String("text".to_string());
                        }
                        Some("input_image") => {
                            *t = Value::String("image_url".to_string());
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

/// Convert a Responses API request body to a Chat Completions API request body.
///
/// This is the same logic as `model-provider-info::china_provider_conversions`,
/// inlined here to avoid circular dependency between `codex-api` and
/// `codex-model-provider-info`.
fn convert_request_body(responses_body: &mut Value) {
    let Some(obj) = responses_body.as_object_mut() else {
        return;
    };

    let mut chat_obj = Map::new();

    // 1. Convert input → messages
    if let Some(input) = obj.remove("input") {
        let mut messages = if let Value::Array(items) = input {
            items
        } else {
            vec![input]
        };

        // 2. Convert "developer" role to "system" (Responses API uses "developer",
        //    but Chat Completions API only supports "system"/"user"/"assistant"/"tool")
        //    Also convert content item types: "input_text" → "text", "input_image" → "image_url"
        for msg in &mut messages {
            if let Some(obj) = msg.as_object_mut() {
                if let Some(role) = obj.get_mut("role") {
                    if role.as_str() == Some("developer") {
                        *role = Value::String("system".to_string());
                    }
                }
                // Convert content item types within arrays
                if let Some(content) = obj.get_mut("content") {
                    convert_content_types(content);
                }
            }
        }

        // 3. Prepend instructions as system message
        if let Some(instructions) = obj.remove("instructions") {
            if let Some(text) = instructions.as_str() {
                if !text.is_empty() {
                    messages.insert(0, json!({
                        "role": "system",
                        "content": text
                    }));
                }
            }
        }

        chat_obj.insert("messages".to_string(), Value::Array(messages));
    }

    // 3. Convert max_output_tokens → max_tokens
    if let Some(max_output_tokens) = obj.remove("max_output_tokens") {
        chat_obj.insert("max_tokens".to_string(), max_output_tokens);
    }

    // 4. Convert text.format → response_format
    if let Some(text) = obj.remove("text") {
        if let Some(format) = text.get("format") {
            chat_obj.insert("response_format".to_string(), format.clone());
        }
    }

    // 5. Remove Responses-specific fields that Chat API doesn't understand
    obj.remove("store");
    obj.remove("include");
    obj.remove("prompt_cache_key");
    obj.remove("service_tier");
    obj.remove("client_metadata");
    obj.remove("reasoning");
    obj.remove("parallel_tool_calls");

    // 6. Convert tools from Responses API format to Chat Completions format
    //    Responses:  {"type":"function", "name":"x", "description":"...", "parameters":{...}}
    //    Chat:      {"type":"function", "function":{"name":"x", "description":"...", "parameters":{...}}}
    if let Some(tools) = obj.remove("tools") {
        if let Some(arr) = tools.as_array() {
            let converted: Vec<Value> = arr
                .iter()
                .filter_map(|tool| {
                    if let Some(t) = tool.as_object() {
                        if t.get("type").and_then(|v| v.as_str()) == Some("function") {
                            // Move name/description/parameters/strict into a nested "function" object
                            let func = json!({
                                "name": t.get("name").cloned().unwrap_or(Value::Null),
                                "description": t.get("description").cloned().unwrap_or(Value::Null),
                                "parameters": t.get("parameters").cloned().unwrap_or(Value::Null),
                                "strict": t.get("strict").cloned().unwrap_or(Value::Null),
                            });
                            let mut chat_tool = json!({"type": "function", "function": func});
                            if let Some(obj) = chat_tool.as_object_mut() {
                                for (k, v) in t {
                                    if !matches!(k.as_str(), "type" | "name" | "description" | "parameters" | "strict") {
                                        obj.insert(k.clone(), v.clone());
                                    }
                                }
                            }
                            return Some(chat_tool);
                        }
                    }
                    // Drop non-function tools (local_shell, web_search, etc.)
                    // Chat Completions API only supports type:"function"
                    None
                })
                .collect();
            chat_obj.insert("tools".to_string(), Value::Array(converted));
        } else {
            chat_obj.insert("tools".to_string(), tools);
        }
    }

    // 7. Pass through remaining fields (model, temperature, tool_choice, etc.)
    let remaining = std::mem::take(obj);
    for (key, value) in remaining {
        chat_obj.insert(key, value);
    }

    *responses_body = Value::Object(chat_obj);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_returns_chat_completions() {
        // path() does not depend on T, so we can use ReqwestTransport which implements HttpTransport
        assert_eq!(ChatCompletionsClient::<codex_client::ReqwestTransport>::path(), "chat/completions");
    }

    #[test]
    fn converts_request_body_basic() {
        let mut body = json!({
            "model": "doubao-seed-2.0-pro",
            "input": [{"role": "user", "content": "Hello"}],
            "max_output_tokens": 100,
            "stream": true,
            "store": true,
            "include": ["reasoning.encrypted_content"]
        });

        convert_request_body(&mut body);

        assert_eq!(body["messages"], json!([{"role": "user", "content": "Hello"}]));
        assert_eq!(body["max_tokens"], 100);
        assert!(body.get("input").is_none());
        assert!(body.get("max_output_tokens").is_none());
        assert!(body.get("store").is_none());
        assert!(body.get("include").is_none());
    }

    #[test]
    fn converts_request_with_instructions() {
        let mut body = json!({
            "model": "test",
            "instructions": "You are helpful.",
            "input": [{"role": "user", "content": "Hi"}],
        });

        convert_request_body(&mut body);

        let messages = body["messages"].as_array().unwrap();
        assert_eq!(messages[0], json!({"role": "system", "content": "You are helpful."}));
        assert_eq!(messages[1], json!({"role": "user", "content": "Hi"}));
    }

    #[test]
    fn converts_request_with_tools() {
        let mut body = json!({
            "model": "test",
            "input": [{"role": "user", "content": "What's the weather?"}],
            "tools": [{"type": "function", "name": "get_weather", "description": "Get weather", "parameters": {"type": "object"}, "strict": false}],
            "tool_choice": "auto",
            "temperature": 0.7,
        });

        convert_request_body(&mut body);

        let tools = body["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["type"], "function");
        // Should be nested in "function" object for Chat Completions API
        assert_eq!(tools[0]["function"]["name"], "get_weather");
        assert_eq!(tools[0]["function"]["description"], "Get weather");
        assert_eq!(tools[0]["function"]["parameters"]["type"], "object");
        // Top-level name/description/parameters should be gone
        assert!(tools[0].get("name").is_none());
        assert!(tools[0].get("description").is_none());
        assert_eq!(body["tool_choice"], "auto");
        assert_eq!(body["temperature"], 0.7);
    }

    #[test]
    fn converts_request_with_text_format() {
        let mut body = json!({
            "model": "test",
            "input": [{"role": "user", "content": "Hi"}],
            "text": {"format": {"type": "json_object"}},
        });

        convert_request_body(&mut body);

        assert_eq!(body["response_format"], json!({"type": "json_object"}));
        assert!(body.get("text").is_none());
    }

    #[test]
    fn converts_content_item_types() {
        let mut body = json!({
            "model": "test",
            "input": [
                {"role": "user", "content": [
                    {"type": "input_text", "text": "Hello"},
                    {"type": "input_image", "image_url": "http://example.com/img.png"}
                ]},
                {"role": "assistant", "content": [
                    {"type": "output_text", "text": "Hi there"}
                ]}
            ],
        });

        convert_request_body(&mut body);

        let messages = body["messages"].as_array().unwrap();
        let user_content = messages[0]["content"].as_array().unwrap();
        assert_eq!(user_content[0]["type"], "text");
        assert_eq!(user_content[1]["type"], "image_url");
        let assistant_content = messages[1]["content"].as_array().unwrap();
        assert_eq!(assistant_content[0]["type"], "text");
    }

    #[test]
    fn converts_developer_role_to_system() {
        let mut body = json!({
            "model": "test",
            "input": [
                {"role": "developer", "content": "You are helpful."},
                {"role": "user", "content": "Hi"}
            ],
        });

        convert_request_body(&mut body);

        let messages = body["messages"].as_array().unwrap();
        assert_eq!(messages[0]["role"], "system");
        assert_eq!(messages[0]["content"], "You are helpful.");
        assert_eq!(messages[1]["role"], "user");
    }

    #[test]
    fn handles_empty_instructions() {
        let mut body = json!({
            "model": "test",
            "instructions": "",
            "input": [{"role": "user", "content": "Hi"}],
        });

        convert_request_body(&mut body);

        let messages = body["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], json!({"role": "user", "content": "Hi"}));
    }

    #[test]
    fn handles_missing_input() {
        let mut body = json!({
            "model": "test",
            "max_output_tokens": 50,
        });

        convert_request_body(&mut body);

        assert!(body.get("messages").is_none());
        assert_eq!(body["max_tokens"], 50);
    }
}
