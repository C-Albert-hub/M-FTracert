use crate::models::*;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use which::which;

pub struct FridaManager {
    pub config: FridaConfig,
}

impl FridaManager {
    pub fn new(config: FridaConfig) -> Self {
        Self { config }
    }

    pub fn update_config(&mut self, config: FridaConfig) {
        self.config = config;
    }

    pub fn get_frida_path(&self) -> Result<PathBuf, String> {
        if let Some(ref custom_path) = self.config.frida_path {
            if !custom_path.is_empty() {
                return Ok(PathBuf::from(custom_path));
            }
        }
        which("frida").map_err(|_| "Frida not found in PATH".to_string())
    }

    pub fn get_frida_ps_path(&self) -> Result<PathBuf, String> {
        let frida_path = self.get_frida_path()?;
        let frida_ps_path = if let Some(parent) = frida_path.parent() {
            parent.join(if cfg!(windows) { "frida-ps.exe" } else { "frida-ps" })
        } else {
            return Err("Invalid frida path".to_string());
        };

        if !frida_ps_path.exists() {
            return Err(format!(
                "frida-ps not found at: {:?}. Please make sure frida-tools is installed.",
                frida_ps_path
            ));
        }

        Ok(frida_ps_path)
    }

    pub fn create_frida_command(&self, pid: u32, script_path: &str) -> Result<Command, String> {
        let frida_path = self.get_frida_path()?;
        let device_id = self.config.device_id.as_ref().unwrap_or(&"usb".to_string()).clone();

        let mut cmd = Command::new(frida_path);

        // 添加设备参数
        if device_id == "usb" || device_id == "U" {
            cmd.arg("-U");
        } else {
            cmd.arg("-D").arg(&device_id);
        }

        // 添加其他参数
        cmd.args(&[
            "-p", &pid.to_string(),
            "-l", script_path,
        ]);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        
        // 设置环境变量以支持 UTF-8
        #[cfg(windows)]
        {
            cmd.env("PYTHONIOENCODING", "utf-8");
            cmd.env("PYTHONUTF8", "1");
        }
        
        // Windows: 隐藏控制台窗口
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            cmd.creation_flags(CREATE_NO_WINDOW);
        }

        Ok(cmd)
    }

    pub fn is_available(&self) -> bool {
        self.get_frida_path().is_ok()
    }
}
