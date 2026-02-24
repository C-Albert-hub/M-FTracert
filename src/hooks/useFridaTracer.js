import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { listen } from "@tauri-apps/api/event";

export function useFridaTracer() {
  const [processes, setProcesses] = useState([]);
  const [selectedProcess, setSelectedProcess] = useState(null);
  const [tracerStatus, setTracerStatus] = useState({ status: "DETACHED", events_count: 0, frida_available: false });
  const [traceEvents, setTraceEvents] = useState([]);
  const [matchRules, setMatchRules] = useState([]);
  const [blacklistRules, setBlacklistRules] = useState([]);
  const [fridaPath, setFridaPath] = useState("");
  const [deviceId, setDeviceId] = useState("usb");
  const [isLoadingProcesses, setIsLoadingProcesses] = useState(false);

  useEffect(() => {
    let interval;
    let unlisten;
    
    const initialize = async () => {
      try {
        await checkFridaInstallation();
        await loadProcesses();
        await loadRules();
        
        // 设置事件监听器
        unlisten = await listen('trace_event', (event) => {
          console.log("[+] Received trace_event:", {
            type: event.payload?.event_type,
            id: event.payload?.id,
            message: event.payload?.message?.substring(0, 50)
          });
          
          const traceEvent = event.payload;
          
          // 检测进程关闭消息
          if (traceEvent.message && traceEvent.message.includes("Target process closed")) {
            console.log("[!] Target process closed detected, auto-detaching...");
            // 自动 Detach 和刷新
            setTimeout(async () => {
              try {
                await invoke("detach_from_process");
                setSelectedProcess(null);
                console.log("[+] Auto-detached successfully");
                
                // 刷新进程列表
                console.log("[+] Refreshing process list...");
                setIsLoadingProcesses(true);
                try {
                  const processList = await invoke("refresh_processes");
                  setProcesses(processList);
                  console.log("[+] Process list refreshed");
                } catch (error) {
                  console.error("[-] Failed to refresh processes:", error);
                } finally {
                  setIsLoadingProcesses(false);
                }
              } catch (error) {
                console.error("[-] Auto-detach failed:", error);
              }
            }, 500);
            
            // 不将这个内部消息添加到事件列表
            return;
          }
          
          setTraceEvents(prev => {
            // 防止重复添加相同 ID 的事件
            if (prev.some(e => e.id === traceEvent.id)) {
              console.log("[!] Duplicate event detected:", traceEvent.id);
              return prev;
            }
            const newEvents = [...prev, traceEvent];
            console.log(`[+] Total events: ${newEvents.length}`);
            return newEvents;
          });
        });
        
        console.log("[+] trace_event listener setup complete");
        
        // 延迟刷新进程列表，避免初始化时的错误提示
        setTimeout(() => {
          refreshProcesses();
        }, 500);
        
        interval = setInterval(() => {
          updateTracerStatus();
        }, 1000);
      } catch (error) {
        console.error("Initialization error:", error);
      }
    };
    
    initialize();

    return () => {
      if (interval) clearInterval(interval);
      if (unlisten) unlisten();
    };
  }, []);

  const checkFridaInstallation = async () => {
    try {
      const config = await invoke("check_frida_installation");
      if (config.frida_path) {
        setFridaPath(config.frida_path);
      }
      if (config.device_id) {
        setDeviceId(config.device_id);
      }
    } catch (error) {
      console.error("Frida check failed:", error);
    }
  };

  const loadProcesses = async () => {
    try {
      const processList = await invoke("get_process_list");
      setProcesses(processList);
    } catch (error) {
      console.error("Failed to load processes:", error);
    }
  };

  const refreshProcesses = async () => {
    setIsLoadingProcesses(true);
    try {
      const processList = await invoke("refresh_processes");
      setProcesses(processList);
      console.log("[OK] Loaded processes:", processList.length);
    } catch (error) {
      console.error("Failed to refresh processes:", error);
      // 只在第一次失败且没有进程时显示警告
      if (processes.length === 0) {
        const errorMsg = `进程列表获取失败: ${error}\n\n请确保:\n1. ADB 已安装且在 PATH 中\n2. Android 设备/模拟器已连接 (adb devices)\n3. USB 调试已开启\n4. 设备 ID 配置正确 (当前: ${deviceId})`;
        console.warn(errorMsg);
      }
    } finally {
      setIsLoadingProcesses(false);
    }
  };

  const loadRules = async () => {
    try {
      const matchRulesList = await invoke("get_match_rules");
      const blacklistRulesList = await invoke("get_blacklist_rules");
      setMatchRules(matchRulesList);
      setBlacklistRules(blacklistRulesList);
    } catch (error) {
      console.error("Failed to load rules:", error);
    }
  };

  const updateTracerStatus = async () => {
    try {
      const status = await invoke("get_tracer_status");
      
      // 检测到自动 Detach
      if (tracerStatus.status === "ATTACHED" && status.status === "DETACHED") {
        console.log("[WARN] Detected auto-detach, clearing selected process");
        setSelectedProcess(null);
      }
      
      setTracerStatus(status);
    } catch (error) {
      console.error("[ERROR] Failed to get tracer status:", error);
    }
  };

  const attachToProcess = async (process) => {
    try {
      console.log("[ATTACH] Attaching to process:", process.pid, process.name);
      await invoke("attach_to_process", { pid: process.pid });
      setSelectedProcess(process);
      updateTracerStatus();
      console.log("[OK] Attach command completed");
    } catch (error) {
      console.error("[ERROR] Failed to attach to process:", error);
      alert("Failed to attach: " + error);
    }
  };

  const detachFromProcess = async () => {
    try {
      await invoke("detach_from_process");
      setSelectedProcess(null);
      updateTracerStatus();
    } catch (error) {
      console.error("Failed to detach from process:", error);
    }
  };

  const clearEvents = async (eventType = null) => {
    try {
      if (eventType) {
        // 只清除特定类型的事件
        setTraceEvents(prev => {
          if (eventType === "log") {
            // 清除 System Hooks (log 类型)
            return prev.filter(e => e.event_type !== "log");
          } else if (eventType === "method") {
            // 清除 Method Calls (enter/exit 类型)
            return prev.filter(e => e.event_type === "log");
          }
          return prev;
        });
      } else {
        // 清除所有事件
        await invoke("clear_trace_events");
        setTraceEvents([]);
      }
      updateTracerStatus();
    } catch (error) {
      console.error("Failed to clear events:", error);
    }
  };

  const addMatchRule = async (pattern) => {
    try {
      // 移除可能的 M: 前缀
      let cleanPattern = pattern;
      if (pattern.startsWith("M:")) {
        cleanPattern = pattern.substring(2);
      }
      
      // 警告：.* 会导致性能问题
      if (cleanPattern === ".*") {
        alert(
          "警告：匹配所有类 (.*) 会导致大量事件，可能造成应用卡死！\n\n" +
          "建议使用更精确的规则，例如：\n" +
          "• com.example.myapp.*  (只匹配目标应用)\n" +
          "• com.example.MainActivity  (只匹配特定类)\n\n" +
          "如果确定要使用，请再次添加。"
        );
        return;
      }
      
      const ruleType = "match";
      await invoke("add_match_rule", { pattern: cleanPattern, ruleType });
      loadRules();
    } catch (error) {
      console.error("Failed to add match rule:", error);
    }
  };

  const removeMatchRule = async (index) => {
    try {
      await invoke("remove_match_rule", { index });
      loadRules();
    } catch (error) {
      console.error("Failed to remove match rule:", error);
    }
  };

  const addBlacklistRule = async (pattern) => {
    try {
      // 移除可能的 M: 或 E: 前缀
      let cleanPattern = pattern;
      if (pattern.startsWith("M:") || pattern.startsWith("E:")) {
        cleanPattern = pattern.substring(2);
      }
      
      const ruleType = "match";
      await invoke("add_blacklist_rule", { pattern: cleanPattern, ruleType });
      loadRules();
    } catch (error) {
      console.error("Failed to add blacklist rule:", error);
    }
  };

  const removeBlacklistRule = async (index) => {
    try {
      await invoke("remove_blacklist_rule", { index });
      loadRules();
    } catch (error) {
      console.error("Failed to remove blacklist rule:", error);
    }
  };

  const saveSettings = async () => {
    try {
      await invoke("set_frida_config", { 
        fridaPath: fridaPath || null,
        deviceId: deviceId || "usb"
      });
      await checkFridaInstallation();
      updateTracerStatus();
    } catch (error) {
      console.error("Failed to save settings:", error);
      alert("Failed to save settings: " + error);
    }
  };

  return {
    processes,
    selectedProcess,
    setSelectedProcess,
    tracerStatus,
    traceEvents,
    matchRules,
    blacklistRules,
    fridaPath,
    setFridaPath,
    deviceId,
    setDeviceId,
    isLoadingProcesses,
    refreshProcesses,
    attachToProcess,
    detachFromProcess,
    clearEvents,
    addMatchRule,
    removeMatchRule,
    addBlacklistRule,
    removeBlacklistRule,
    saveSettings,
  };
}
