use crate::models::*;
use crate::frida_manager::FridaManager;
use crate::script_generator::generate_frida_script;
use crate::event_parser::parse_frida_output;
use crate::config_storage::{save_config, AppConfig};
use std::sync::{Arc, Mutex};
use std::io::{BufRead, BufReader};
use tauri::{Manager, State};
use uuid::Uuid;

pub type ProcessList = Arc<Mutex<Vec<Process>>>;
pub type TraceEvents = Arc<Mutex<Vec<TraceEvent>>>;
pub type TracerState = Arc<Mutex<TracerStatus>>;
pub type MatchRulesList = Arc<Mutex<Vec<MatchRule>>>;
pub type BlacklistRulesList = Arc<Mutex<Vec<BlacklistRule>>>;
pub type FridaManagerState = Arc<Mutex<FridaManager>>;
pub type ActiveFridaProcess = Arc<Mutex<Option<std::process::Child>>>;

#[tauri::command]
pub async fn get_adb_devices() -> Result<Vec<String>, String> {
    let mut cmd = std::process::Command::new("adb");
    cmd.arg("devices");
    
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    
    let output = tokio::task::spawn_blocking(move || cmd.output())
        .await
        .map_err(|e| format!("Task error: {}", e))?
        .map_err(|e| format!("Failed to execute adb: {}", e))?;
    
    if !output.status.success() {
        return Err("Failed to get device list".to_string());
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let mut devices = Vec::new();
    
    for line in stdout.lines().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 && parts[1] == "device" {
            devices.push(parts[0].to_string());
        }
    }
    
    Ok(devices)
}


#[tauri::command]
pub fn check_frida_installation(frida_manager: State<FridaManagerState>) -> Result<FridaConfig, String> {
    let manager = frida_manager.lock().unwrap();
    
    // 检查 adb 是否可用
    let adb_available = std::process::Command::new("adb")
        .arg("version")
        .output()
        .is_ok();
    
    if !adb_available {
        return Err("ADB not found. Please install Android SDK Platform Tools and add adb to PATH.".to_string());
    }
    
    Ok(FridaConfig {
        frida_path: manager.config.frida_path.clone(),
        device_id: manager.config.device_id.clone(),
        spawn_mode: manager.config.spawn_mode,
    })
}

#[tauri::command]
pub fn set_frida_config(
    frida_path: Option<String>,
    device_id: String,
    frida_manager: State<FridaManagerState>,
    match_rules: State<MatchRulesList>,
    blacklist_rules: State<BlacklistRulesList>,
) -> Result<(), String> {
    // 验证路径
    if let Some(ref path) = frida_path {
        if !path.is_empty() && !std::path::Path::new(path).exists() {
            return Err(format!("Frida path does not exist: {}", path));
        }
    }
    
    let mut manager = frida_manager.lock().unwrap();
    manager.update_config(FridaConfig {
        frida_path: frida_path.clone(),
        device_id: Some(device_id.clone()),
        spawn_mode: false,
    });
    drop(manager);
    
    // 保存配置到文件
    let config = AppConfig {
        frida_config: FridaConfig {
            frida_path,
            device_id: Some(device_id),
            spawn_mode: false,
        },
        match_rules: match_rules.lock().unwrap().clone(),
        blacklist_rules: blacklist_rules.lock().unwrap().clone(),
    };
    
    save_config(&config)?;
    
    Ok(())
}

#[tauri::command]
pub async fn refresh_processes(
    process_list: State<'_, ProcessList>,
    frida_manager: State<'_, FridaManagerState>,
) -> Result<Vec<Process>, String> {
    // 使用 adb 获取进程列表
    let device_id = {
        let manager = frida_manager.lock().unwrap();
        manager.config.device_id.clone().unwrap_or_else(|| "".to_string())
    };
    
    // 构建 adb 命令
    let mut cmd = std::process::Command::new("adb");
    
    // 如果指定了设备 ID 且不是 "usb" 或 "U"，则添加 -s 参数
    if !device_id.is_empty() && device_id != "usb" && device_id != "U" {
        cmd.arg("-s").arg(&device_id);
    }
    
    // 执行 ps 命令获取进程列表
    cmd.args(&["shell", "ps"]);
    
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    
    let output = tokio::task::spawn_blocking(move || cmd.output())
        .await
        .map_err(|e| format!("Task error: {}", e))?
        .map_err(|e| format!("Failed to execute adb: {}. Make sure adb is installed and in PATH.", e))?;
    
    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("adb command failed: {}", error));
    }
    
    // 解析输出
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    
    let mut processes = Vec::new();
    let lines: Vec<&str> = stdout.lines().collect();
    
    if lines.is_empty() {
        return Err("No output from adb ps command".to_string());
    }
    
    // 跳过标题行，解析进程信息
    // ps 输出格式通常是: USER PID PPID VSZ RSS WCHAN PC NAME
    for line in lines.iter().skip(1) {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = line.split_whitespace().collect();
        // 至少需要 PID 和 NAME 字段
        if parts.len() >= 9 {
            // PID 在第二列（索引1）
            if let Ok(pid) = parts[1].parse::<u32>() {
                // NAME 在最后一列（索引8）
                let name = parts[8].to_string();
                
                // 只显示应用进程（包名格式）
                if name.contains('.') {
                    processes.push(Process {
                        pid,
                        name: name.clone(),
                        package: Some(name),
                    });
                }
            }
        }
    }
    
    if processes.is_empty() {
        return Err("No application processes found. Make sure the device is connected.".to_string());
    }
    
    // 按包名排序
    processes.sort_by(|a, b| a.name.cmp(&b.name));
    
    let mut list = process_list.lock().unwrap();
    *list = processes.clone();
    Ok(processes)
}

#[tauri::command]
pub fn get_process_list(state: State<ProcessList>) -> Vec<Process> {
    state.lock().unwrap().clone()
}

#[tauri::command]
pub async fn attach_to_process(
    pid: u32,
    process_list: State<'_, ProcessList>,
    tracer_state: State<'_, TracerState>,
    match_rules: State<'_, MatchRulesList>,
    blacklist_rules: State<'_, BlacklistRulesList>,
    frida_manager: State<'_, FridaManagerState>,
    active_process: State<'_, ActiveFridaProcess>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    // 先 detach 旧进程（如果存在）
    {
        let mut active = active_process.lock().unwrap();
        if let Some(mut child) = active.take() {
            println!("[+] Detaching from previous process...");
            drop(child.stdin.take());
            std::thread::sleep(std::time::Duration::from_millis(100));
            match child.try_wait() {
                Ok(Some(_)) => {},
                _ => {
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        }
    }
    
    let processes = process_list.lock().unwrap();
    let target_process = processes.iter().find(|p| p.pid == pid).cloned();
    drop(processes);
    
    let process = target_process.ok_or("Process not found")?;
    
    // 检查匹配规则
    let match_rules_list = match_rules.lock().unwrap().clone();
    if match_rules_list.is_empty() {
        return Err("[!] No match rules configured!\n\nPlease add at least one match rule before attaching.\n\nExample:\n  com.zj.wuaipojie2023_1.*".to_string());
    }
    
    // 更新状态
    {
        let mut status = tracer_state.lock().unwrap();
        status.status = "CONNECTING".to_string();
        status.target_process = Some(process.clone());
    }
    
    // 生成脚本
    let match_rules_list = match_rules.lock().unwrap().clone();
    let blacklist_rules_list = blacklist_rules.lock().unwrap().clone();
    let script_content = generate_frida_script(&match_rules_list, &blacklist_rules_list);
    
    // 保存脚本
    let script_path = std::env::temp_dir().join("m_ftracert_script.js");
    std::fs::write(&script_path, script_content)
        .map_err(|e| format!("Failed to write script: {}", e))?;
    
    // 创建 Frida 命令
    let manager = frida_manager.lock().unwrap();
    let mut cmd = manager.create_frida_command(pid, script_path.to_str().unwrap())?;
    drop(manager);
    
    // 启动进程
    let mut child = cmd.spawn().map_err(|e| format!("Failed to start frida: {}", e))?;
    
    let stdout = child.stdout.take().ok_or("Failed to get stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to get stderr")?;
    
    // 保存进程句柄
    {
        let mut active = active_process.lock().unwrap();
        *active = Some(child);
    }
    
    // 更新状态
    {
        let mut status = tracer_state.lock().unwrap();
        status.status = "ATTACHED".to_string();
    }
    
    // 发送测试事件以验证事件系统
    let test_event = TraceEvent {
        id: Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().format("%H:%M:%S%.3f").to_string(),
        event_type: "log".to_string(),
        thread_id: None,
        thread_name: None,
        class_name: None,
        method_name: None,
        args: None,
        return_value: None,
        message: "[+] Frida attached successfully, waiting for events...".to_string(),
    };
    match app_handle.emit_all("trace_event", &test_event) {
        Ok(_) => {},
        Err(e) => println!("[-] Failed to send initial event: {:?}", e),
    }
    
    // 监听输出
    let app_handle_clone = app_handle.clone();
    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Some(event) = parse_frida_output(&line) {
                    let event_summary = format!("{} - {}", 
                        event.event_type, 
                        event.message.chars().take(50).collect::<String>()
                    );
                    
                    match app_handle_clone.emit_all("trace_event", &event) {
                        Ok(_) => {},
                        Err(e) => println!("[-] Failed to emit {}: {:?}", event_summary, e),
                    }
                }
            }
        }
        
        // stdout 关闭意味着 Frida 进程退出
        println!("[!] Frida process stdout closed");
        // 发送内部消息触发前端自动 Detach，但不显示给用户
        let event = TraceEvent {
            id: Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().format("%H:%M:%S%.3f").to_string(),
            event_type: "log".to_string(),
            thread_id: None,
            thread_name: None,
            class_name: None,
            method_name: None,
            args: None,
            return_value: None,
            message: "Target process closed".to_string(),
        };
        let _ = app_handle_clone.emit_all("trace_event", event);
    });
    
    // 监听错误
    let app_handle_clone2 = app_handle.clone();
    tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                if !line.trim().is_empty() {
                    println!("[-] Frida stderr: {}", line);
                    let event = TraceEvent {
                        id: Uuid::new_v4().to_string(),
                        timestamp: chrono::Utc::now().format("%H:%M:%S%.3f").to_string(),
                        event_type: "log".to_string(),
                        thread_id: None,
                        thread_name: None,
                        class_name: None,
                        method_name: None,
                        args: None,
                        return_value: None,
                        message: format!("[-] Error: {}", line),
                    };
                    let _ = app_handle_clone2.emit_all("trace_event", event);
                }
            }
        }
    });
    
    Ok("Attached successfully".to_string())
}

#[tauri::command]
pub fn detach_from_process(
    tracer_state: State<TracerState>,
    active_process: State<ActiveFridaProcess>,
) -> Result<String, String> {
    // 终止进程
    {
        let mut active = active_process.lock().unwrap();
        if let Some(mut child) = active.take() {
            // 尝试优雅关闭：先关闭 stdin，让 Frida 自然退出
            drop(child.stdin.take());
            
            // 等待一小段时间让进程自然退出
            std::thread::sleep(std::time::Duration::from_millis(100));
            
            // 如果还没退出，强制 kill
            match child.try_wait() {
                Ok(Some(_)) => {
                    // 进程已经退出
                }
                _ => {
                    // 进程还在运行，强制终止
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
        }
    }
    
    // 更新状态
    let mut status = tracer_state.lock().unwrap();
    status.status = "DETACHED".to_string();
    status.target_process = None;
    status.events_count = 0;
    
    Ok("Detached successfully".to_string())
}

#[tauri::command]
pub fn get_tracer_status(
    tracer_state: State<TracerState>,
    frida_manager: State<FridaManagerState>,
) -> TracerStatus {
    let mut status = tracer_state.lock().unwrap();
    let manager = frida_manager.lock().unwrap();
    status.frida_available = manager.is_available();
    status.clone()
}

#[tauri::command]
pub fn clear_trace_events(
    events: State<TraceEvents>,
    tracer_state: State<TracerState>,
) -> Result<(), String> {
    let mut event_list = events.lock().unwrap();
    event_list.clear();
    
    let mut status = tracer_state.lock().unwrap();
    status.events_count = 0;
    
    Ok(())
}

// 规则管理命令
#[tauri::command]
pub fn add_match_rule(
    pattern: String, 
    rule_type: String, 
    rules: State<MatchRulesList>,
    frida_manager: State<FridaManagerState>,
    blacklist_rules: State<BlacklistRulesList>,
) -> Result<(), String> {
    let mut rule_list = rules.lock().unwrap();
    rule_list.push(MatchRule { pattern, rule_type });
    
    // 保存配置
    let manager = frida_manager.lock().unwrap();
    let config = AppConfig {
        frida_config: manager.config.clone(),
        match_rules: rule_list.clone(),
        blacklist_rules: blacklist_rules.lock().unwrap().clone(),
    };
    drop(rule_list);
    drop(manager);
    
    save_config(&config)?;
    Ok(())
}

#[tauri::command]
pub fn remove_match_rule(
    index: usize, 
    rules: State<MatchRulesList>,
    frida_manager: State<FridaManagerState>,
    blacklist_rules: State<BlacklistRulesList>,
) -> Result<(), String> {
    let mut rule_list = rules.lock().unwrap();
    if index < rule_list.len() {
        rule_list.remove(index);
        
        // 保存配置
        let manager = frida_manager.lock().unwrap();
        let config = AppConfig {
            frida_config: manager.config.clone(),
            match_rules: rule_list.clone(),
            blacklist_rules: blacklist_rules.lock().unwrap().clone(),
        };
        drop(rule_list);
        drop(manager);
        
        save_config(&config)?;
        Ok(())
    } else {
        Err("Index out of bounds".to_string())
    }
}

#[tauri::command]
pub fn get_match_rules(rules: State<MatchRulesList>) -> Vec<MatchRule> {
    rules.lock().unwrap().clone()
}

#[tauri::command]
pub fn add_blacklist_rule(
    pattern: String, 
    rule_type: String, 
    rules: State<BlacklistRulesList>,
    frida_manager: State<FridaManagerState>,
    match_rules: State<MatchRulesList>,
) -> Result<(), String> {
    let mut rule_list = rules.lock().unwrap();
    rule_list.push(BlacklistRule { pattern, rule_type });
    
    // 保存配置
    let manager = frida_manager.lock().unwrap();
    let config = AppConfig {
        frida_config: manager.config.clone(),
        match_rules: match_rules.lock().unwrap().clone(),
        blacklist_rules: rule_list.clone(),
    };
    drop(rule_list);
    drop(manager);
    
    save_config(&config)?;
    Ok(())
}

#[tauri::command]
pub fn remove_blacklist_rule(
    index: usize, 
    rules: State<BlacklistRulesList>,
    frida_manager: State<FridaManagerState>,
    match_rules: State<MatchRulesList>,
) -> Result<(), String> {
    let mut rule_list = rules.lock().unwrap();
    if index < rule_list.len() {
        rule_list.remove(index);
        
        // 保存配置
        let manager = frida_manager.lock().unwrap();
        let config = AppConfig {
            frida_config: manager.config.clone(),
            match_rules: match_rules.lock().unwrap().clone(),
            blacklist_rules: rule_list.clone(),
        };
        drop(rule_list);
        drop(manager);
        
        save_config(&config)?;
        Ok(())
    } else {
        Err("Index out of bounds".to_string())
    }
}

#[tauri::command]
pub fn get_blacklist_rules(rules: State<BlacklistRulesList>) -> Vec<BlacklistRule> {
    rules.lock().unwrap().clone()
}
