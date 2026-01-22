use crate::models::TraceEvent;
use uuid::Uuid;

pub fn parse_frida_output(line: &str) -> Option<TraceEvent> {
    // Frida 输出格式: message: {'type': 'send', 'payload': 'tracer::...'} data: None
    if !line.contains("tracer::") || !line.contains("'payload': '") {
        return None;
    }

    // 提取 payload
    let start = line.find("'payload': '")?;
    let after_payload = &line[start + 12..];
    let end = after_payload.find("'}")?;
    let payload_content = &after_payload[..end];
    
    // 移除 tracer:: 前缀
    let json_str = payload_content.trim_start_matches("tracer::");
    
    // 解析 JSON
    let packet: serde_json::Value = serde_json::from_str(json_str).ok()?;
    
    Some(parse_frida_packet(packet))
}

fn parse_frida_packet(packet: serde_json::Value) -> TraceEvent {
    let cmd = packet["cmd"].as_str().unwrap_or("log");
    let data = &packet["data"];
    
    match cmd {
        "log" => TraceEvent {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().format("%H:%M:%S%.3f").to_string(),
            event_type: "log".to_string(),
            thread_id: None,
            thread_name: None,
            class_name: None,
            method_name: None,
            args: None,
            return_value: None,
            message: data.as_str().unwrap_or("").to_string(),
        },
        "enter" => {
            let empty_array = vec![];
            let data_array = data.as_array().unwrap_or(&empty_array);
            TraceEvent {
                id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now().format("%H:%M:%S%.3f").to_string(),
                event_type: "enter".to_string(),
                thread_id: data_array.get(0).and_then(|v| v.as_u64()).map(|v| v as u32),
                thread_name: data_array.get(1).and_then(|v| v.as_str()).map(|s| s.to_string()),
                class_name: data_array.get(2).and_then(|v| v.as_str()).map(|s| s.to_string()),
                method_name: data_array.get(3).and_then(|v| v.as_str()).map(|s| s.to_string()),
                args: data_array.get(4).and_then(|v| v.as_array()).map(|arr| {
                    arr.iter().map(|v| v.as_str().unwrap_or("").to_string()).collect()
                }),
                return_value: None,
                message: format!("Method enter: {}", 
                    data_array.get(3).and_then(|v| v.as_str()).unwrap_or("unknown")),
            }
        },
        "exit" => {
            let empty_array = vec![];
            let data_array = data.as_array().unwrap_or(&empty_array);
            TraceEvent {
                id: Uuid::new_v4().to_string(),
                timestamp: chrono::Utc::now().format("%H:%M:%S%.3f").to_string(),
                event_type: "exit".to_string(),
                thread_id: data_array.get(0).and_then(|v| v.as_u64()).map(|v| v as u32),
                thread_name: None,
                class_name: None,
                method_name: None,
                args: None,
                return_value: data_array.get(1).and_then(|v| v.as_str()).map(|s| s.to_string()),
                message: format!("Method exit: {}", 
                    data_array.get(1).and_then(|v| v.as_str()).unwrap_or("void")),
            }
        },
        _ => TraceEvent {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().format("%H:%M:%S%.3f").to_string(),
            event_type: "log".to_string(),
            thread_id: None,
            thread_name: None,
            class_name: None,
            method_name: None,
            args: None,
            return_value: None,
            message: format!("Unknown command: {}", cmd),
        }
    }
}
