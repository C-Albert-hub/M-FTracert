use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Process {
    pub pid: u32,
    pub name: String,
    pub package: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TraceEvent {
    pub id: String,
    pub timestamp: String,
    pub event_type: String, // "log", "enter", "exit", "hook"
    pub thread_id: Option<u32>,
    pub thread_name: Option<String>,
    pub class_name: Option<String>,
    pub method_name: Option<String>,
    pub args: Option<Vec<String>>,
    pub return_value: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TracerStatus {
    pub status: String, // "DETACHED", "ATTACHED", "CONNECTING", "ERROR"
    pub target_process: Option<Process>,
    pub events_count: usize,
    pub frida_available: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MatchRule {
    pub pattern: String,
    pub rule_type: String, // "regex", "exact", "match"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlacklistRule {
    pub pattern: String,
    pub rule_type: String, // "regex", "exact", "match"
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FridaConfig {
    pub frida_path: Option<String>,
    pub device_id: Option<String>,
    pub spawn_mode: bool,
}
