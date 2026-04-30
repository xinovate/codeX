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
        let mut message_item_added = false;
        let mut _last_model: Option<String> = None;

        // Accumulate tool call state from streaming deltas.
        // Key: tool call index, Value: (id, name, arguments, item_added_sent)
        let mut tool_calls: std::collections::HashMap<i64, (String, String, String, bool)> =
            std::collections::HashMap::new();

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
                        .send(Err(ApiError::Stream("idle timeout waiting for SSE".into())))
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
                        end_turn: None,
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
                    // Content → output_text.delta
                    if let Some(content) = delta.get("content").and_then(|v| v.as_str()) {
                        if !content.is_empty() {
                            // Emit message item added on first text content
                            if !message_item_added {
                                let _ = tx_event
                                    .send(Ok(ResponseEvent::OutputItemAdded(
                                        codex_protocol::models::ResponseItem::Message {
                                            id: None,
                                            role: "assistant".to_string(),
                                            content: vec![],
                                            phase: None,
                                        },
                                    )))
                                    .await;
                                message_item_added = true;
                            }
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
                            // Ensure message item is added before reasoning delta
                            if !message_item_added {
                                let _ = tx_event
                                    .send(Ok(ResponseEvent::OutputItemAdded(
                                        codex_protocol::models::ResponseItem::Message {
                                            id: None,
                                            role: "assistant".to_string(),
                                            content: vec![],
                                            phase: None,
                                        },
                                    )))
                                    .await;
                                message_item_added = true;
                            }
                            let _ = tx_event
                                .send(Ok(ResponseEvent::ReasoningContentDelta {
                                    delta: reasoning.to_string(),
                                    content_index: 0,
                                }))
                                .await;
                        }
                    }

                    // Accumulate tool call deltas and emit proper events
                    if let Some(tc_deltas) = delta.get("tool_calls") {
                        if let Some(arr) = tc_deltas.as_array() {
                            for tc in arr {
                                let index = match tc.get("index").and_then(|v| v.as_i64()) {
                                    Some(i) => i,
                                    None => continue,
                                };

                                let entry = tool_calls.entry(index).or_insert_with(|| {
                                    (String::new(), String::new(), String::new(), false)
                                });

                                // Update id if present (first chunk for this tool call)
                                if let Some(id) = tc.get("id").and_then(|v| v.as_str()) {
                                    entry.0 = id.to_string();
                                }

                                // Update name if present (first chunk for this tool call)
                                if let Some(name) = tc
                                    .get("function")
                                    .and_then(|f| f.get("name"))
                                    .and_then(|v| v.as_str())
                                {
                                    entry.1 = name.to_string();
                                }

                                // Accumulate arguments
                                if let Some(args) = tc
                                    .get("function")
                                    .and_then(|f| f.get("arguments"))
                                    .and_then(|v| v.as_str())
                                {
                                    entry.2.push_str(args);
                                }

                                // Emit OutputItemAdded(FunctionCall) on first delta for this tool call
                                if !entry.3 {
                                    entry.3 = true;
                                    let _ = tx_event
                                        .send(Ok(ResponseEvent::OutputItemAdded(
                                            codex_protocol::models::ResponseItem::FunctionCall {
                                                id: None,
                                                name: entry.1.clone(),
                                                namespace: None,
                                                arguments: String::new(),
                                                call_id: entry.0.clone(),
                                            },
                                        )))
                                        .await;
                                }

                                // Emit argument delta
                                let delta_args = tc
                                    .get("function")
                                    .and_then(|f| f.get("arguments"))
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                if !delta_args.is_empty() {
                                    let _ = tx_event
                                        .send(Ok(ResponseEvent::ToolCallInputDelta {
                                            item_id: format!("tool_call_{index}"),
                                            call_id: Some(entry.0.clone()),
                                            delta: delta_args.to_string(),
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

                    if finish_reason == "tool_calls" {
                        // For tool call responses: close any open message first,
                        // then emit OutputItemDone(FunctionCall) for each tool call.

                        // Try to get tool calls from message.tool_calls (non-streaming)
                        // or fall back to accumulated streaming state.
                        let final_tool_calls: Vec<(String, String, String)> =
                            if let Some(message) = choices.get("message") {
                                if let Some(tcs) = message.get("tool_calls") {
                                    if let Some(arr) = tcs.as_array() {
                                        arr.iter()
                                            .map(|tc| {
                                                let id = tc
                                                    .get("id")
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("")
                                                    .to_string();
                                                let name = tc
                                                    .get("function")
                                                    .and_then(|f| f.get("name"))
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("")
                                                    .to_string();
                                                let arguments = tc
                                                    .get("function")
                                                    .and_then(|f| f.get("arguments"))
                                                    .and_then(|v| v.as_str())
                                                    .unwrap_or("")
                                                    .to_string();
                                                (id, name, arguments)
                                            })
                                            .collect()
                                    } else {
                                        Vec::new()
                                    }
                                } else {
                                    Vec::new()
                                }
                            } else {
                                Vec::new()
                            };

                        // Use message.tool_calls if available, otherwise fall back
                        // to accumulated streaming state
                        if !final_tool_calls.is_empty() {
                            for (id, name, arguments) in final_tool_calls {
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
                        } else {
                            // Fall back to accumulated state from streaming deltas
                            let mut sorted: Vec<_> = tool_calls.iter().collect();
                            sorted.sort_by_key(|(k, _)| **k);
                            for (_, (id, name, arguments, _)) in sorted {
                                let _ = tx_event
                                    .send(Ok(ResponseEvent::OutputItemDone(
                                        codex_protocol::models::ResponseItem::FunctionCall {
                                            id: None,
                                            name: name.clone(),
                                            namespace: None,
                                            arguments: arguments.clone(),
                                            call_id: id.clone(),
                                        },
                                    )))
                                    .await;
                            }
                        }
                    } else {
                        // Emit output_item.done for regular message
                        if !message_item_added {
                            // If we never sent OutputItemAdded(Message), send it now
                            let _ = tx_event
                                .send(Ok(ResponseEvent::OutputItemAdded(
                                    codex_protocol::models::ResponseItem::Message {
                                        id: None,
                                        role: "assistant".to_string(),
                                        content: vec![],
                                        phase: None,
                                    },
                                )))
                                .await;
                        }
                        let _ = tx_event
                            .send(Ok(ResponseEvent::OutputItemDone(
                                codex_protocol::models::ResponseItem::Message {
                                    id: None,
                                    role: "assistant".to_string(),
                                    content: vec![codex_protocol::models::ContentItem::OutputText {
                                        text: accumulated_text.clone(),
                                    }],
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
                            end_turn: None,
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

        // 2. Convert Responses API item types to Chat Completions messages.
        //    Responses API uses "function_call" and "function_call_output" types
        //    that have no "role" field — convert them to standard message format.
        //    Also handle role conversions and content type conversions.
        //    Reasoning items are collected and attached to the next assistant message
        //    as `reasoning_content` (required by DeepSeek and similar providers).
        let mut pending_reasoning_content: Option<String> = None;
        for msg in &mut messages {
            if let Some(obj) = msg.as_object_mut() {
                // Handle Responses API function_call items (no role field)
                let msg_type = obj.get("type").and_then(|v| v.as_str());
                if msg_type == Some("function_call") {
                    // Convert to assistant message with tool_calls
                    let call_id = obj.get("call_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let name = obj.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let arguments = obj.get("arguments").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    *obj = Map::new();
                    obj.insert("role".to_string(), Value::String("assistant".to_string()));
                    obj.insert("content".to_string(), Value::Null);
                    obj.insert("tool_calls".to_string(), json!([{
                        "id": call_id,
                        "type": "function",
                        "function": {
                            "name": name,
                            "arguments": arguments
                        }
                    }]));
                    continue;
                }
                if msg_type == Some("function_call_output") {
                    // Responses API → Chat Completions: convert to role:"tool" with
                    // tool_call_id so the provider can link it to the tool call.
                    // DeepSeek, Volcengine, Kimi all require this format.
                    // Note: Xiaomi Mimo doesn't support function calling, so this
                    // path is never hit for Mimo. The `continue` skips the
                    // role:"tool" → role:"user" fallback below.
                    let call_id = obj.get("call_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    // `output` can be a plain string or an array of content items
                    // (e.g. [{"type":"output_text","text":"..."}]).
                    let output = match obj.get("output") {
                        Some(Value::String(s)) => s.clone(),
                        Some(Value::Array(arr)) => arr
                            .iter()
                            .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                            .collect::<Vec<_>>()
                            .join(""),
                        _ => String::new(),
                    };
                    *obj = Map::new();
                    obj.insert("role".to_string(), Value::String("tool".to_string()));
                    obj.insert("tool_call_id".to_string(), Value::String(call_id));
                    obj.insert("content".to_string(), Value::String(output));
                    continue;
                }
                if msg_type == Some("reasoning") {
                    // Extract reasoning content to pass back in the next assistant
                    // message. DeepSeek API requires reasoning_content to be included
                    // in subsequent requests (despite docs saying otherwise).
                    let reasoning_text: String = obj
                        .get("content")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|item| {
                                    item.get("text").and_then(|t| t.as_str())
                                })
                                .collect::<Vec<_>>()
                                .join("")
                        })
                        .unwrap_or_default();
                    if !reasoning_text.is_empty() {
                        let existing = pending_reasoning_content.get_or_insert_with(String::new);
                        existing.push_str(&reasoning_text);
                    }
                    *obj = Map::new();
                    obj.insert("__remove__".to_string(), Value::Bool(true));
                    continue;
                }

                if let Some(role) = obj.get_mut("role") {
                    if role.as_str() == Some("developer") {
                        *role = Value::String("system".to_string());
                    }
                    // Fallback for providers that don't support "tool" role (e.g.
                    // Xiaomi Mimo). In normal Responses API flow, tool results arrive
                    // as "function_call_output" (handled above), so this only triggers
                    // if the input already contains role:"tool" messages.
                    if role.as_str() == Some("tool") {
                        *role = Value::String("user".to_string());
                        let call_id = obj
                            .get("tool_call_id")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        if let Some(content) = obj.get_mut("content") {
                            let prefix = call_id
                                .map(|id| format!("[Tool result for {id}]\n"))
                                .unwrap_or_default();
                            if let Some(s) = content.as_str() {
                                *content = Value::String(format!("{prefix}{s}"));
                            } else if let Some(arr) = content.as_array() {
                                let text: String = arr
                                    .iter()
                                    .filter_map(|item| item.get("text").and_then(|t| t.as_str()))
                                    .collect();
                                *content = Value::String(format!("{prefix}{text}"));
                            }
                        }
                    }
                }
                let is_assistant = obj.get("role").and_then(|r| r.as_str()) == Some("assistant");

                // Attach accumulated reasoning content to this assistant message.
                // DeepSeek API requires reasoning_content to be passed back.
                if is_assistant {
                    if let Some(reasoning) = pending_reasoning_content.take() {
                        obj.insert("reasoning_content".to_string(), Value::String(reasoning));
                    }
                }

                // Convert content item types within arrays
                if let Some(content) = obj.get_mut("content") {
                    convert_content_types(content);

                    // For assistant messages, flatten content array to plain string.
                    // Chat Completions API expects assistant content to be a string,
                    // not an array of content items.
                    if is_assistant {
                        if let Some(arr) = content.as_array() {
                            let text: String = arr
                                .iter()
                                .filter_map(|item| {
                                    if item.get("type").and_then(|t| t.as_str()) == Some("text") {
                                        item.get("text").and_then(|t| t.as_str())
                                    } else {
                                        None
                                    }
                                })
                                .collect();
                            if !text.is_empty() {
                                *content = Value::String(text);
                            }
                        }
                    }
                }
            }
        }

        // Remove items marked for removal (e.g. Reasoning items already extracted)
        messages.retain(|msg| {
            !msg.as_object()
                .map(|o| o.get("__remove__").is_some())
                .unwrap_or(false)
        });

        // Merge consecutive assistant messages with tool_calls into a single
        // assistant message. Responses API may emit one assistant message per
        // tool call, but Chat Completions API requires all tool_calls to be in
        // a single assistant message, immediately followed by tool responses.
        // Input:  [assistant(tc_A), assistant(tc_B), tool(A), tool(B)]
        // Output: [assistant(tc_A+tc_B), tool(A), tool(B)]
        let mut merged: Vec<Value> = Vec::with_capacity(messages.len());
        for msg in messages {
            let is_assistant_with_tools = msg
                .as_object()
                .map(|o| {
                    o.get("role").and_then(|r| r.as_str()) == Some("assistant")
                        && o.get("tool_calls").is_some()
                })
                .unwrap_or(false);

            if is_assistant_with_tools {
                // Check if the last merged message is also an assistant with tool_calls
                if let Some(last) = merged.last_mut() {
                    let last_is_assistant_with_tools = last
                        .as_object()
                        .map(|o| {
                            o.get("role").and_then(|r| r.as_str()) == Some("assistant")
                                && o.get("tool_calls").is_some()
                        })
                        .unwrap_or(false);
                    if last_is_assistant_with_tools {
                        // Merge tool_calls from current into last
                        if let (Some(last_obj), Some(cur_obj)) =
                            (last.as_object_mut(), msg.as_object())
                        {
                            if let (Some(Value::Array(last_tc)), Some(Value::Array(cur_tc))) = (
                                last_obj.get_mut("tool_calls"),
                                cur_obj.get("tool_calls"),
                            ) {
                                last_tc.extend(cur_tc.iter().cloned());
                            }
                            // Keep the last assistant's content if non-null, otherwise use current
                            if last_obj
                                .get("content")
                                .map_or(true, |c| c.is_null())
                            {
                                if let Some(cur_content) = cur_obj.get("content") {
                                    last_obj.insert("content".to_string(), cur_content.clone());
                                }
                            }
                        }
                        continue;
                    }
                }
            }
            merged.push(msg);
        }
        messages = merged;

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

    // 6. Disable thinking mode for DeepSeek and similar providers.
    //    DeepSeek's deepseek-v4-flash has thinking enabled by default, which
    //    returns reasoning_content that must be round-tripped. Since Codex
    //    doesn't support this for Chat Completions providers, disable it.
    chat_obj.insert(
        "thinking".to_string(),
        json!({"type": "disabled"}),
    );

    // 7. Convert tools from Responses API format to Chat Completions format
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

    // 8. Pass through remaining fields (model, temperature, tool_choice, etc.)
    let remaining = std::mem::take(obj);
    for (key, value) in remaining {
        chat_obj.insert(key, value);
    }

    *responses_body = Value::Object(chat_obj);

    // Debug: log the converted request body for troubleshooting
    tracing::debug!("Chat Completions request body: {}", serde_json::to_string(responses_body).unwrap_or_default());
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
        // Assistant content should be flattened to a plain string
        assert_eq!(messages[1]["content"], "Hi there");
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

    #[test]
    fn flattens_assistant_content_to_string() {
        let mut body = json!({
            "model": "test",
            "input": [
                {"role": "user", "content": "Hello"},
                {"role": "assistant", "content": [
                    {"type": "output_text", "text": "Part 1. "},
                    {"type": "output_text", "text": "Part 2."}
                ]},
                {"role": "user", "content": "Follow up"}
            ],
        });

        convert_request_body(&mut body);

        let messages = body["messages"].as_array().unwrap();
        // User messages stay as-is
        assert_eq!(messages[0]["content"], "Hello");
        // Assistant content flattened to single string
        assert_eq!(messages[1]["content"], "Part 1. Part 2.");
        assert_eq!(messages[2]["content"], "Follow up");
    }

    #[test]
    fn converts_tool_role_to_user() {
        let mut body = json!({
            "model": "test",
            "input": [
                {"role": "user", "content": "list files"},
                {"role": "assistant", "content": [
                    {"type": "output_text", "text": "I'll list the files."}
                ], "tool_calls": [{"id": "call_123", "type": "function", "function": {"name": "exec", "arguments": "{\"cmd\":\"ls\"}"}}]},
                {"role": "tool", "tool_call_id": "call_123", "content": "file1.txt\nfile2.txt"}
            ],
        });

        convert_request_body(&mut body);

        let messages = body["messages"].as_array().unwrap();
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[1]["role"], "assistant");
        // Tool role converted to user for Xiaomi Mimo compatibility
        assert_eq!(messages[2]["role"], "user");
        assert!(messages[2]["content"].as_str().unwrap().contains("call_123"));
        assert!(messages[2]["content"].as_str().unwrap().contains("file1.txt"));
    }

    #[test]
    fn converts_function_call_output_to_tool_role() {
        let mut body = json!({
            "model": "test",
            "input": [
                {"role": "user", "content": "list files"},
                {"type": "function_call", "id": "fc_1", "call_id": "call_123", "name": "exec", "arguments": "{\"cmd\":\"ls\"}"},
                {"type": "function_call_output", "call_id": "call_123", "output": "file1.txt\nfile2.txt"}
            ],
        });

        convert_request_body(&mut body);

        let messages = body["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0]["role"], "user");
        // function_call → assistant with tool_calls
        assert_eq!(messages[1]["role"], "assistant");
        assert!(messages[1]["tool_calls"].is_array());
        assert_eq!(messages[1]["tool_calls"][0]["id"], "call_123");
        // function_call_output → tool with tool_call_id
        assert_eq!(messages[2]["role"], "tool");
        assert_eq!(messages[2]["tool_call_id"], "call_123");
        assert_eq!(messages[2]["content"], "file1.txt\nfile2.txt");
    }

    #[test]
    fn merges_consecutive_assistant_tool_calls() {
        // Responses API may emit one assistant message per tool call.
        // Chat Completions requires all tool_calls in a single assistant message.
        let mut body = json!({
            "model": "test",
            "input": [
                {"role": "user", "content": "do two things"},
                {"type": "function_call", "id": "fc_1", "call_id": "call_A", "name": "exec", "arguments": "{\"cmd\":\"ls\"}"},
                {"type": "function_call", "id": "fc_2", "call_id": "call_B", "name": "exec", "arguments": "{\"cmd\":\"pwd\"}"},
                {"type": "function_call_output", "call_id": "call_A", "output": "file1.txt"},
                {"type": "function_call_output", "call_id": "call_B", "output": "/tmp"},
            ],
        });

        convert_request_body(&mut body);

        let messages = body["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 4); // user, assistant(merged), tool(A), tool(B)
        assert_eq!(messages[0]["role"], "user");
        // Two function_calls merged into one assistant message
        assert_eq!(messages[1]["role"], "assistant");
        let tool_calls = messages[1]["tool_calls"].as_array().unwrap();
        assert_eq!(tool_calls.len(), 2);
        assert_eq!(tool_calls[0]["id"], "call_A");
        assert_eq!(tool_calls[1]["id"], "call_B");
        // Tool responses
        assert_eq!(messages[2]["role"], "tool");
        assert_eq!(messages[2]["tool_call_id"], "call_A");
        assert_eq!(messages[3]["role"], "tool");
        assert_eq!(messages[3]["tool_call_id"], "call_B");
    }

    #[test]
    fn attaches_reasoning_content_to_next_assistant_message() {
        let mut body = json!({
            "model": "test",
            "input": [
                {"role": "user", "content": "Think about this"},
                {"type": "reasoning", "id": "rs-1", "summary": [],
                 "content": [{"type": "reasoning_text", "text": "Let me think..."}]},
                {"role": "assistant", "content": [
                    {"type": "output_text", "text": "Here is my answer."}
                ]},
                {"role": "user", "content": "Follow up"}
            ],
        });

        convert_request_body(&mut body);

        let messages = body["messages"].as_array().unwrap();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0]["role"], "user");
        assert_eq!(messages[1]["role"], "assistant");
        assert_eq!(messages[1]["content"], "Here is my answer.");
        assert_eq!(messages[1]["reasoning_content"], "Let me think...");
        assert_eq!(messages[2]["role"], "user");
    }

    #[test]
    fn reasoning_content_without_following_assistant_is_ignored() {
        let mut body = json!({
            "model": "test",
            "input": [
                {"type": "reasoning", "id": "rs-1", "summary": [],
                 "content": [{"type": "reasoning_text", "text": "Orphan reasoning"}]},
                {"role": "user", "content": "Hello"}
            ],
        });

        convert_request_body(&mut body);

        let messages = body["messages"].as_array().unwrap();
        // Reasoning item removed, only user message remains
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0]["role"], "user");
    }
}
