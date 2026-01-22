// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod models;
mod frida_manager;
mod script_generator;
mod event_parser;
mod commands;
mod config_storage;

use models::*;
use frida_manager::FridaManager;
use commands::*;
use config_storage::load_config;
use std::sync::{Arc, Mutex};
use which::which;

fn main() {
    // 加载配置
    let app_config = load_config();
    
    let process_list: ProcessList = Arc::new(Mutex::new(Vec::new()));
    let trace_events: TraceEvents = Arc::new(Mutex::new(Vec::new()));
    let tracer_state: TracerState = Arc::new(Mutex::new(TracerStatus {
        status: "DETACHED".to_string(),
        target_process: None,
        events_count: 0,
        frida_available: which("frida").is_ok(),
    }));
    
    // 从配置文件加载规则
    let match_rules: MatchRulesList = Arc::new(Mutex::new(app_config.match_rules.clone()));
    let blacklist_rules: BlacklistRulesList = Arc::new(Mutex::new(app_config.blacklist_rules.clone()));
    
    // 从配置文件加载 Frida 配置
    let frida_manager = FridaManager::new(app_config.frida_config.clone());
    let frida_manager_state: FridaManagerState = Arc::new(Mutex::new(frida_manager));
    let active_process: ActiveFridaProcess = Arc::new(Mutex::new(None));

    tauri::Builder::default()
        .manage(process_list)
        .manage(trace_events)
        .manage(tracer_state)
        .manage(match_rules)
        .manage(blacklist_rules)
        .manage(frida_manager_state)
        .manage(active_process)
        .invoke_handler(tauri::generate_handler![
            check_frida_installation,
            set_frida_config,
            get_process_list,
            refresh_processes,
            attach_to_process,
            detach_from_process,
            get_tracer_status,
            clear_trace_events,
            add_match_rule,
            remove_match_rule,
            get_match_rules,
            add_blacklist_rule,
            remove_blacklist_rule,
            get_blacklist_rules
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
