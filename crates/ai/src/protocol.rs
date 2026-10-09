//! Provider wire formats. Complete tool arguments are data for the host, never authority.
use crate::{AiError, Event, Message, Profile, Protocol, Usage, sse::Frame};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub schema: Value,
}

#[derive(Debug, Clone, Default)]
pub struct Request {
    pub system: String,
    pub messages: Vec<Message>,
    /// Live-only complete native exchanges, including signed provider output when required.
    pub wire_history: Vec<Value>,
    pub tools: Vec<ToolDefinition>,
}

pub fn encode_message(protocol: Protocol, message: &Message) -> Value {
    match protocol {
        Protocol::Anthropic => {
            json!({"role":message.role,"content":[{"type":"text","text":message.content}]})
        }
        _ => json!({"role":message.role,"content":message.content}),
    }
}

pub fn tool_result(protocol: Protocol, id: &str, result: &Value) -> Value {
    let content = result.to_string();
    match protocol {
        Protocol::Responses => json!({"type":"function_call_output","call_id":id,"output":content}),
        Protocol::Chat => json!({"role":"tool","tool_call_id":id,"content":content}),
        Protocol::Anthropic => {
            json!({"role":"user","content":[{"type":"tool_result","tool_use_id":id,"content":content,"is_error":result.get("error").is_some()}]})
        }
    }
}

pub fn request_body(profile: &Profile, request: &Request, stream: bool) -> Result<Value, AiError> {
    profile.validate()?;
    let protocol = Protocol::parse(&profile.protocol)?;
    let messages: Vec<Value> = if request.wire_history.is_empty() {
        request
            .messages
            .iter()
            .map(|m| encode_message(protocol, m))
            .collect()
    } else {
        request.wire_history.clone()
    };
    if serde_json::to_vec(&messages)
        .map_err(|_| AiError::Protocol)?
        .len()
        + request.system.len()
        > profile.limits.input_bytes
    {
        return Err(AiError::Limit);
    }
    let tools: Vec<Value> = request.tools.iter().map(|t| match protocol {
        Protocol::Responses => json!({"type":"function","name":t.name,"description":t.description,"parameters":t.schema}),
        Protocol::Chat => json!({"type":"function","function":{"name":t.name,"description":t.description,"parameters":t.schema}}),
        Protocol::Anthropic => json!({"name":t.name,"description":t.description,"input_schema":t.schema}),
    }).collect();
    let mut body = match protocol {
        Protocol::Responses => {
            json!({"model":profile.model,"input":messages,"instructions":request.system,"max_output_tokens":profile.limits.output_tokens,"store":false,"stream":stream})
        }
        Protocol::Chat => {
            let mut messages = messages;
            if !request.system.is_empty() {
                messages.insert(0, json!({"role":"system","content":request.system}));
            }
            json!({"model":profile.model,"messages":messages,"max_tokens":profile.limits.output_tokens,"stream":stream})
        }
        Protocol::Anthropic => {
            json!({"model":profile.model,"messages":messages,"system":request.system,"max_tokens":profile.limits.output_tokens,"stream":stream})
        }
    };
    if !tools.is_empty() {
        body["tools"] = Value::Array(tools);
    }
    Ok(body)
}

#[derive(Default)]
struct PendingCall {
    id: String,
    name: String,
    arguments: String,
    dispatched: bool,
}

pub struct Decoder {
    protocol: Protocol,
    calls: BTreeMap<String, PendingCall>,
    pub output: Vec<Value>,
    text: String,
    blocks: BTreeMap<u64, Value>,
    chat_extra: BTreeMap<String, String>,
    complete: bool,
    output_limit: usize,
    argument_limit: usize,
    total_bytes: usize,
}
impl Decoder {
    pub fn new(profile: &Profile) -> Result<Self, AiError> {
        Ok(Self {
            protocol: Protocol::parse(&profile.protocol)?,
            calls: BTreeMap::new(),
            output: Vec::new(),
            text: String::new(),
            blocks: BTreeMap::new(),
            chat_extra: BTreeMap::new(),
            complete: false,
            output_limit: profile.limits.output_bytes,
            argument_limit: profile.limits.tool_bytes,
            total_bytes: 0,
        })
    }
    pub fn is_complete(&self) -> bool {
        self.complete
    }
    fn text_event(&mut self, text: &str, events: &mut Vec<Event>) -> Result<(), AiError> {
        if self.text.len() + text.len() > self.output_limit {
            return Err(AiError::Limit);
        }
        self.text.push_str(text);
        if !text.is_empty() {
            events.push(Event::Text(text.into()));
        }
        Ok(())
    }
    fn dispatch_call(&mut self, key: &str, events: &mut Vec<Event>) -> Result<(), AiError> {
        let call = self.calls.get_mut(key).ok_or(AiError::Protocol)?;
        if call.dispatched {
            return Ok(());
        }
        if call.arguments.len() > self.argument_limit {
            return Err(AiError::Limit);
        }
        if call.name.is_empty() || call.id.is_empty() {
            return Err(AiError::Protocol);
        }
        let arguments: Value =
            serde_json::from_str(&call.arguments).map_err(|_| AiError::Protocol)?;
        if !arguments.is_object() {
            return Err(AiError::Protocol);
        }
        call.dispatched = true;
        events.push(Event::ToolCall {
            id: call.id.clone(),
            name: call.name.clone(),
            arguments,
        });
        Ok(())
    }
    fn done(&mut self, events: &mut Vec<Event>) {
        if !self.complete {
            self.complete = true;
            events.push(Event::Complete);
        }
    }
    fn account(&mut self, bytes: usize) -> Result<(), AiError> {
        self.total_bytes = self.total_bytes.checked_add(bytes).ok_or(AiError::Limit)?;
        // Bound opaque replay state, not only visible text.
        if self.total_bytes > self.output_limit.saturating_mul(8) {
            Err(AiError::Limit)
        } else {
            Ok(())
        }
    }
    pub fn push(&mut self, frame: &Frame) -> Result<Vec<Event>, AiError> {
        self.account(frame.data.len())?;
        let mut events = Vec::new();
        if frame.data == "[DONE]" {
            if self.protocol != Protocol::Chat || !self.complete {
                return Err(AiError::Protocol);
            }
            return Ok(events);
        }
        let value: Value = serde_json::from_str(&frame.data).map_err(|_| AiError::Protocol)?;
        if value.get("error").is_some() || value["type"] == "error" {
            return Err(AiError::Protocol);
        }
        match self.protocol {
            Protocol::Responses => match value["type"].as_str().unwrap_or(&frame.event) {
                "response.output_text.delta" => self.text_event(
                    value["delta"].as_str().ok_or(AiError::Protocol)?,
                    &mut events,
                )?,
                "response.output_item.added" if value["item"]["type"] == "function_call" => {
                    let item = &value["item"];
                    let key = item["id"].as_str().ok_or(AiError::Protocol)?.to_string();
                    self.calls.insert(
                        key,
                        PendingCall {
                            id: item["call_id"].as_str().unwrap_or("").into(),
                            name: item["name"].as_str().unwrap_or("").into(),
                            arguments: item["arguments"].as_str().unwrap_or("").into(),
                            dispatched: false,
                        },
                    );
                }
                "response.function_call_arguments.delta" => {
                    let call = self
                        .calls
                        .get_mut(value["item_id"].as_str().ok_or(AiError::Protocol)?)
                        .ok_or(AiError::Protocol)?;
                    call.arguments
                        .push_str(value["delta"].as_str().ok_or(AiError::Protocol)?);
                    if call.arguments.len() > self.argument_limit {
                        return Err(AiError::Limit);
                    }
                }
                "response.function_call_arguments.done" => {
                    let key = value["item_id"].as_str().ok_or(AiError::Protocol)?;
                    if let Some(arguments) = value["arguments"].as_str() {
                        self.calls.get_mut(key).ok_or(AiError::Protocol)?.arguments =
                            arguments.into();
                    }
                    self.dispatch_call(key, &mut events)?;
                }
                "response.output_item.done" => {
                    let item = value["item"].clone();
                    if item["type"] == "function_call" {
                        let key = item["id"].as_str().ok_or(AiError::Protocol)?.to_string();
                        let call = self.calls.entry(key.clone()).or_default();
                        call.id = item["call_id"].as_str().unwrap_or("").into();
                        call.name = item["name"].as_str().unwrap_or("").into();
                        call.arguments = item["arguments"].as_str().unwrap_or("").into();
                        self.dispatch_call(&key, &mut events)?;
                    }
                    self.output.push(item);
                }
                "response.completed" => {
                    if value["response"]["status"]
                        .as_str()
                        .is_some_and(|s| s != "completed")
                    {
                        return Err(AiError::Protocol);
                    }
                    if let Some(output) = value["response"]["output"].as_array() {
                        self.output = output.clone();
                    }
                    emit_usage(&value["response"]["usage"], &mut events);
                    self.done(&mut events);
                }
                "response.failed" | "response.incomplete" => return Err(AiError::Protocol),
                _ => {}
            },
            Protocol::Chat => {
                emit_usage(&value["usage"], &mut events);
                if let Some(choices) = value["choices"].as_array() {
                    for choice in choices.iter().take(1) {
                        let delta = &choice["delta"];
                        if let Some(text) = delta["content"].as_str() {
                            self.text_event(text, &mut events)?;
                        }
                        for extra in ["reasoning_content", "reasoning"] {
                            if let Some(text) = delta[extra].as_str() {
                                self.chat_extra
                                    .entry(extra.into())
                                    .or_default()
                                    .push_str(text);
                            }
                        }
                        if let Some(calls) = delta["tool_calls"].as_array() {
                            for update in calls {
                                let key = update["index"]
                                    .as_u64()
                                    .ok_or(AiError::Protocol)?
                                    .to_string();
                                let call = self.calls.entry(key).or_default();
                                if let Some(id) = update["id"].as_str() {
                                    call.id = id.into();
                                }
                                if let Some(name) = update["function"]["name"].as_str() {
                                    call.name.push_str(name);
                                }
                                if let Some(arguments) = update["function"]["arguments"].as_str() {
                                    call.arguments.push_str(arguments);
                                }
                                if call.arguments.len() > self.argument_limit {
                                    return Err(AiError::Limit);
                                }
                            }
                        }
                        match choice["finish_reason"].as_str() {
                            Some("tool_calls") => {
                                for key in self.calls.keys().cloned().collect::<Vec<_>>() {
                                    self.dispatch_call(&key, &mut events)?;
                                }
                                self.finish_chat();
                                self.done(&mut events);
                            }
                            Some("stop") if self.calls.is_empty() => {
                                self.finish_chat();
                                self.done(&mut events);
                            }
                            Some("length") => return Err(AiError::Limit),
                            Some(_) => return Err(AiError::Protocol),
                            None => {}
                        }
                    }
                }
            }
            Protocol::Anthropic => match value["type"].as_str().unwrap_or(&frame.event) {
                "message_start" => emit_usage(&value["message"]["usage"], &mut events),
                "content_block_start" => {
                    let index = value["index"].as_u64().ok_or(AiError::Protocol)?;
                    let block = value["content_block"].clone();
                    if block["type"] == "tool_use" {
                        self.calls.insert(
                            index.to_string(),
                            PendingCall {
                                id: block["id"].as_str().unwrap_or("").into(),
                                name: block["name"].as_str().unwrap_or("").into(),
                                ..Default::default()
                            },
                        );
                    }
                    self.blocks.insert(index, block);
                }
                "content_block_delta" => {
                    let index = value["index"].as_u64().ok_or(AiError::Protocol)?;
                    let delta = &value["delta"];
                    match delta["type"].as_str() {
                        Some("text_delta") => {
                            let text = delta["text"].as_str().ok_or(AiError::Protocol)?;
                            self.text_event(text, &mut events)?;
                            let block = self.blocks.get_mut(&index).ok_or(AiError::Protocol)?;
                            block["text"] = Value::String(format!(
                                "{}{text}",
                                block["text"].as_str().unwrap_or("")
                            ));
                        }
                        Some("input_json_delta") => {
                            let call = self
                                .calls
                                .get_mut(&index.to_string())
                                .ok_or(AiError::Protocol)?;
                            call.arguments
                                .push_str(delta["partial_json"].as_str().ok_or(AiError::Protocol)?);
                            if call.arguments.len() > self.argument_limit {
                                return Err(AiError::Limit);
                            }
                        }
                        Some("thinking_delta") | Some("signature_delta") => {
                            let field = if delta["type"] == "thinking_delta" {
                                "thinking"
                            } else {
                                "signature"
                            };
                            let block = self.blocks.get_mut(&index).ok_or(AiError::Protocol)?;
                            block[field] = Value::String(format!(
                                "{}{}",
                                block[field].as_str().unwrap_or(""),
                                delta[field].as_str().unwrap_or("")
                            ));
                        }
                        _ => {}
                    }
                }
                "content_block_stop" => {
                    let index = value["index"].as_u64().ok_or(AiError::Protocol)?;
                    let key = index.to_string();
                    if let Some(call) = self.calls.get_mut(&key) {
                        if call.arguments.is_empty() {
                            call.arguments =
                                self.blocks.get(&index).ok_or(AiError::Protocol)?["input"]
                                    .to_string();
                        }
                        self.dispatch_call(&key, &mut events)?;
                        self.blocks.get_mut(&index).ok_or(AiError::Protocol)?["input"] =
                            serde_json::from_str(&self.calls[&key].arguments)
                                .map_err(|_| AiError::Protocol)?;
                    }
                }
                "message_delta" => {
                    if matches!(
                        value["delta"]["stop_reason"].as_str(),
                        Some("max_tokens" | "refusal")
                    ) {
                        return Err(AiError::Limit);
                    }
                    emit_usage(&value["usage"], &mut events);
                }
                "message_stop" => {
                    if self.calls.values().any(|c| !c.dispatched) {
                        return Err(AiError::Protocol);
                    }
                    self.output = vec![
                        json!({"role":"assistant","content":self.blocks.values().collect::<Vec<_>>()}),
                    ];
                    self.done(&mut events);
                }
                _ => {}
            },
        }
        Ok(events)
    }
    fn finish_chat(&mut self) {
        let mut message = json!({"role":"assistant","content":self.text});
        if !self.calls.is_empty() {
            message["tool_calls"] = json!(self.calls.values().map(|c| json!({"id":c.id,"type":"function","function":{"name":c.name,"arguments":c.arguments}})).collect::<Vec<_>>());
        }
        for (field, text) in &self.chat_extra {
            message[field] = json!(text);
        }
        self.output = vec![message];
    }
    pub fn completed_response(&mut self, value: Value) -> Result<Vec<Event>, AiError> {
        self.account(value.to_string().len())?;
        let mut events = Vec::new();
        match self.protocol {
            Protocol::Responses => {
                if value["status"] != "completed" {
                    return Err(AiError::Protocol);
                }
                self.output = value["output"].as_array().ok_or(AiError::Protocol)?.clone();
                for item in self.output.clone() {
                    if item["type"] == "function_call" {
                        let key = item["id"].as_str().ok_or(AiError::Protocol)?.to_string();
                        self.calls.insert(
                            key.clone(),
                            PendingCall {
                                id: item["call_id"].as_str().unwrap_or("").into(),
                                name: item["name"].as_str().unwrap_or("").into(),
                                arguments: item["arguments"].as_str().unwrap_or("").into(),
                                dispatched: false,
                            },
                        );
                        self.dispatch_call(&key, &mut events)?;
                    } else if let Some(parts) = item["content"].as_array() {
                        for part in parts {
                            if let Some(text) = part["text"].as_str() {
                                self.text_event(text, &mut events)?;
                            }
                        }
                    }
                }
                emit_usage(&value["usage"], &mut events);
            }
            Protocol::Chat => {
                let choice = &value["choices"][0];
                if !matches!(
                    choice["finish_reason"].as_str(),
                    Some("stop" | "tool_calls")
                ) {
                    return Err(AiError::Protocol);
                }
                let message = choice["message"].clone();
                if let Some(text) = message["content"].as_str() {
                    self.text_event(text, &mut events)?;
                }
                if let Some(calls) = message["tool_calls"].as_array() {
                    for (i, c) in calls.iter().enumerate() {
                        let key = i.to_string();
                        self.calls.insert(
                            key.clone(),
                            PendingCall {
                                id: c["id"].as_str().unwrap_or("").into(),
                                name: c["function"]["name"].as_str().unwrap_or("").into(),
                                arguments: c["function"]["arguments"].as_str().unwrap_or("").into(),
                                dispatched: false,
                            },
                        );
                        self.dispatch_call(&key, &mut events)?;
                    }
                }
                self.output = vec![message];
                emit_usage(&value["usage"], &mut events);
            }
            Protocol::Anthropic => {
                if !matches!(
                    value["stop_reason"].as_str(),
                    Some("end_turn" | "tool_use" | "stop_sequence")
                ) {
                    return Err(AiError::Protocol);
                }
                let content = value["content"].as_array().ok_or(AiError::Protocol)?;
                for (i, part) in content.iter().enumerate() {
                    if let Some(text) = part["text"].as_str() {
                        self.text_event(text, &mut events)?;
                    }
                    if part["type"] == "tool_use" {
                        let key = i.to_string();
                        self.calls.insert(
                            key.clone(),
                            PendingCall {
                                id: part["id"].as_str().unwrap_or("").into(),
                                name: part["name"].as_str().unwrap_or("").into(),
                                arguments: part["input"].to_string(),
                                dispatched: false,
                            },
                        );
                        self.dispatch_call(&key, &mut events)?;
                    }
                }
                self.output = vec![json!({"role":"assistant","content":content})];
                emit_usage(&value["usage"], &mut events);
            }
        }
        self.done(&mut events);
        Ok(events)
    }
}
fn emit_usage(value: &Value, events: &mut Vec<Event>) {
    if value.is_object() {
        events.push(Event::Usage(Usage {
            input_tokens: value["input_tokens"]
                .as_u64()
                .or(value["prompt_tokens"].as_u64()),
            output_tokens: value["output_tokens"]
                .as_u64()
                .or(value["completion_tokens"].as_u64()),
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn profile(provider: &str) -> Profile {
        let mut p = Profile::preset(provider, "test");
        p.model = "fixture-model".into();
        p
    }
    fn frame(value: Value) -> Frame {
        Frame {
            event: String::new(),
            data: value.to_string(),
        }
    }
    #[test]
    fn responses_store_false_and_complete_arguments_only_once() {
        let p = profile("openai");
        let body = request_body(
            &p,
            &Request {
                messages: vec![Message {
                    role: "user".into(),
                    content: "Hi".into(),
                }],
                ..Default::default()
            },
            true,
        )
        .unwrap();
        assert_eq!(body["store"], false);
        let mut d = Decoder::new(&p).unwrap();
        assert!(d.push(&frame(json!({"type":"response.output_item.added","item":{"type":"function_call","id":"fc1","call_id":"c1","name":"read_text","arguments":""}}))).unwrap().is_empty());
        assert!(d.push(&frame(json!({"type":"response.function_call_arguments.delta","item_id":"fc1","delta":"{\"path\":"}))).unwrap().is_empty());
        let call=d.push(&frame(json!({"type":"response.function_call_arguments.done","item_id":"fc1","arguments":"{\"path\":\"note.md\"}"}))).unwrap();
        assert!(matches!(&call[0],Event::ToolCall{id,..} if id=="c1"));
        assert!(d.push(&frame(json!({"type":"response.output_item.done","item":{"type":"function_call","id":"fc1","call_id":"c1","name":"read_text","arguments":"{\"path\":\"note.md\"}"}}))).unwrap().is_empty());
        assert!(!d.is_complete());
        d.push(&frame(json!({"type":"response.completed","response":{"status":"completed","usage":{"input_tokens":1,"output_tokens":2}}}))).unwrap();
        assert!(d.is_complete());
        assert_eq!(
            tool_result(Protocol::Responses, "c1", &json!({"text":"note"}))["call_id"],
            "c1"
        );
    }
    #[test]
    fn compatible_chat_assembles_multiple_calls_and_preserves_reasoning() {
        let mut d = Decoder::new(&profile("deepseek")).unwrap();
        d.push(&frame(json!({"choices":[{"delta":{"reasoning_content":"opaque","tool_calls":[{"index":0,"id":"a","function":{"name":"read_text","arguments":"{"}},{"index":1,"id":"b","function":{"name":"list_files","arguments":"{}"}}]},"finish_reason":null}]}))).unwrap();
        d.push(&frame(json!({"choices":[{"delta":{"tool_calls":[{"index":0,"function":{"arguments":"\"path\":\"a.md\"}"}}]},"finish_reason":null}]}))).unwrap();
        let events = d
            .push(&frame(
                json!({"choices":[{"delta":{},"finish_reason":"tool_calls"}]}),
            ))
            .unwrap();
        assert_eq!(
            events
                .iter()
                .filter(|e| matches!(e, Event::ToolCall { .. }))
                .count(),
            2
        );
        assert_eq!(d.output[0]["reasoning_content"], "opaque");
        assert_eq!(d.output[0]["tool_calls"][1]["id"], "b");
        d.push(&Frame {
            event: String::new(),
            data: "[DONE]".into(),
        })
        .unwrap();
    }
    #[test]
    fn anthropic_keeps_signed_blocks_and_pairs_tool_results() {
        let mut d = Decoder::new(&profile("anthropic")).unwrap();
        for value in [
            json!({"type":"content_block_start","index":0,"content_block":{"type":"thinking","thinking":"","signature":""}}),
            json!({"type":"content_block_delta","index":0,"delta":{"type":"thinking_delta","thinking":"opaque"}}),
            json!({"type":"content_block_delta","index":0,"delta":{"type":"signature_delta","signature":"signed"}}),
            json!({"type":"content_block_start","index":1,"content_block":{"type":"tool_use","id":"tool1","name":"read_text","input":{}}}),
            json!({"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":"{\"path\":\"a.md\"}"}}),
        ] {
            assert!(d.push(&frame(value)).unwrap().is_empty());
        }
        assert!(matches!(
            &d.push(&frame(json!({"type":"content_block_stop","index":1})))
                .unwrap()[0],
            Event::ToolCall { .. }
        ));
        d.push(&frame(json!({"type":"message_stop"}))).unwrap();
        assert_eq!(d.output[0]["content"][0]["signature"], "signed");
        assert_eq!(d.output[0]["content"][1]["input"]["path"], "a.md");
        assert_eq!(
            tool_result(Protocol::Anthropic, "tool1", &json!({"text":"a"}))["content"][0]["tool_use_id"],
            "tool1"
        );
    }
    #[test]
    fn truncated_arguments_errors_and_unfinished_streams_are_not_success() {
        let mut d = Decoder::new(&profile("local")).unwrap();
        d.push(&frame(json!({"choices":[{"delta":{"tool_calls":[{"index":0,"id":"a","function":{"name":"read_text","arguments":"{"}}]}}]}))).unwrap();
        assert!(!d.is_complete());
        assert_eq!(
            d.push(&frame(
                json!({"choices":[{"delta":{},"finish_reason":"tool_calls"}]})
            )),
            Err(AiError::Protocol)
        );
        assert_eq!(
            Decoder::new(&profile("openai"))
                .unwrap()
                .push(&frame(json!({"type":"response.incomplete"}))),
            Err(AiError::Protocol)
        );
    }
    #[test]
    fn completed_text_only_local_response_and_budget() {
        let mut p = profile("local");
        let mut d = Decoder::new(&p).unwrap();
        let events=d.completed_response(json!({"choices":[{"message":{"role":"assistant","content":"中文"},"finish_reason":"stop"}]})).unwrap();
        assert!(events.contains(&Event::Text("中文".into())));
        assert!(d.is_complete());
        p.limits.input_bytes = 1024;
        assert_eq!(
            request_body(
                &p,
                &Request {
                    messages: vec![Message {
                        role: "user".into(),
                        content: "x".repeat(1025)
                    }],
                    ..Default::default()
                },
                true
            ),
            Err(AiError::Limit)
        );
    }
}
