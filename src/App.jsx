import { useState } from "react";
import { ProcessList } from "./components/ProcessList";
import { RuleManager } from "./components/RuleManager";
import { TreeEventViewer } from "./components/TreeEventViewer";
import { Settings } from "./components/Settings";
import { useFridaTracer } from "./hooks/useFridaTracer";
import "./App.css";

function App() {
  const [showSettings, setShowSettings] = useState(false);
  
  const {
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
  } = useFridaTracer();

  const getStatusColor = (status) => {
    switch (status) {
      case "ATTACHED": return "#28a745";
      case "CONNECTING": return "#ffc107";
      case "ERROR": return "#dc3545";
      case "DETACHED": 
      default: return "#dc3545";
    }
  };

  const handleSaveSettings = async () => {
    await saveSettings();
    setShowSettings(false);
  };

  return (
    <div className="app">
      <header className="app-header">
        <div className="header-content">
          <div>
            <h1>M-FTracert</h1>
            <p>Real-time method tracing for Android applications</p>
          </div>
          <button className="settings-btn" onClick={() => setShowSettings(!showSettings)}>
            Settings
          </button>
        </div>
      </header>

      <div className="status-bar">
        <div className="status-indicator">
          <span 
            className="status-dot" 
            style={{ backgroundColor: getStatusColor(tracerStatus.status) }}
          ></span>
          <span>Status: {tracerStatus.status}</span>
        </div>
        <div className="frida-status">
          <span 
            className="status-dot" 
            style={{ backgroundColor: tracerStatus.frida_available ? "#28a745" : "#dc3545" }}
          ></span>
          <span>Frida: {tracerStatus.frida_available ? "Available" : "Not Found"}</span>
        </div>
      </div>

      <div className="main-content">
        <Settings
          show={showSettings}
          fridaPath={fridaPath}
          deviceId={deviceId}
          onFridaPathChange={setFridaPath}
          onDeviceIdChange={setDeviceId}
          onSave={handleSaveSettings}
          onCancel={() => setShowSettings(false)}
        />

        <div className="left-panel">
          <ProcessList
            processes={processes}
            selectedProcess={selectedProcess}
            onSelectProcess={setSelectedProcess}
            onAttach={attachToProcess}
            onDetach={detachFromProcess}
            onRefresh={refreshProcesses}
            isLoading={isLoadingProcesses}
            tracerStatus={tracerStatus}
          />

          <RuleManager
            title="Match Rules"
            description="Classes to trace (regex match)"
            rules={matchRules}
            onAddRule={addMatchRule}
            onRemoveRule={removeMatchRule}
          />

          <RuleManager
            title="Blacklist Rules"
            description="Classes to exclude (regex match)"
            rules={blacklistRules}
            onAddRule={addBlacklistRule}
            onRemoveRule={removeBlacklistRule}
          />
        </div>

        <TreeEventViewer
          events={traceEvents}
          onClear={clearEvents}
        />
      </div>
    </div>
  );
}

export default App;
