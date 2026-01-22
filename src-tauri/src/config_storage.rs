use crate::models::{FridaConfig, MatchRule, BlacklistRule};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub frida_config: FridaConfig,
    pub match_rules: Vec<MatchRule>,
    pub blacklist_rules: Vec<BlacklistRule>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            frida_config: FridaConfig {
                frida_path: None,
                device_id: Some("usb".to_string()),
                spawn_mode: false,
            },
            match_rules: vec![],
            blacklist_rules: vec![
                BlacklistRule {
                    pattern: "android.view.*".to_string(),
                    rule_type: "match".to_string(),
                },
                BlacklistRule {
                    pattern: "android.widget.*".to_string(),
                    rule_type: "match".to_string(),
                },
                BlacklistRule {
                    pattern: "android.graphics.*".to_string(),
                    rule_type: "match".to_string(),
                },
                BlacklistRule {
                    pattern: "java.lang.String".to_string(),
                    rule_type: "match".to_string(),
                },
            ],
        }
    }
}

pub fn get_config_path() -> Result<PathBuf, String> {
    let config_dir = dirs::config_dir()
        .ok_or("Failed to get config directory")?;
    
    let app_config_dir = config_dir.join("m-ftracert");
    
    // 确保目录存在
    if !app_config_dir.exists() {
        fs::create_dir_all(&app_config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;
    }
    
    Ok(app_config_dir.join("config.json"))
}

pub fn load_config() -> AppConfig {
    match get_config_path() {
        Ok(path) => {
            if path.exists() {
                match fs::read_to_string(&path) {
                    Ok(content) => {
                        match serde_json::from_str::<AppConfig>(&content) {
                            Ok(mut config) => {
                                println!("[+] Loaded config from: {:?}", path);
                                
                                // 迁移旧的 E: 前缀规则
                                for rule in &mut config.match_rules {
                                    if rule.pattern.starts_with("E:") {
                                        rule.pattern = rule.pattern[2..].to_string();
                                        rule.rule_type = "match".to_string();
                                    }
                                }
                                for rule in &mut config.blacklist_rules {
                                    if rule.pattern.starts_with("E:") {
                                        rule.pattern = rule.pattern[2..].to_string();
                                        rule.rule_type = "match".to_string();
                                    }
                                }
                                
                                return config;
                            }
                            Err(e) => {
                                println!("[!] Failed to parse config: {}, using default", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("[!] Failed to read config: {}, using default", e);
                    }
                }
            } else {
                println!("[+] Config file not found, using default");
            }
        }
        Err(e) => {
            println!("[!] Failed to get config path: {}, using default", e);
        }
    }
    
    AppConfig::default()
}

pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let path = get_config_path()?;
    
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    
    fs::write(&path, json)
        .map_err(|e| format!("Failed to write config: {}", e))?;
    
    println!("[+] Saved config to: {:?}", path);
    Ok(())
}
