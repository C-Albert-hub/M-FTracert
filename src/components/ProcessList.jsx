import { useState } from "react";

export function ProcessList({ 
  processes, 
  selectedProcess, 
  onSelectProcess, 
  onAttach, 
  onDetach, 
  onRefresh,
  isLoading,
  tracerStatus 
}) {
  return (
    <div className="section">
      <h3>Process Selection</h3>
      <button 
        className="refresh-btn" 
        onClick={onRefresh}
        disabled={isLoading}
      >
        {isLoading ? "Loading..." : "Refresh"}
      </button>
      <div className="process-list">
        {isLoading ? (
          <div className="loading-message">Loading processes...</div>
        ) : processes.length === 0 ? (
          <div className="no-processes">
            <p>No processes found</p>
            <small>Click Refresh to load process list</small>
          </div>
        ) : (
          processes.map((process) => {
            // frida-ps 返回格式可能是 "应用名 包名" 或只有 "包名"
            // 尝试分离应用名和包名
            let appName = process.name;
            let packageName = null;
            
            // 如果 name 中包含空格，尝试分离
            const spaceIndex = process.name.indexOf(' ');
            if (spaceIndex > 0) {
              appName = process.name.substring(0, spaceIndex);
              packageName = process.name.substring(spaceIndex + 1).trim();
            }
            
            return (
              <div 
                key={process.pid} 
                className={`process-item ${selectedProcess?.pid === process.pid ? 'selected' : ''}`}
                onClick={() => onSelectProcess(process)}
              >
                <div className="process-app-name">{appName}</div>
                {packageName && (
                  <div className="process-package-name">{packageName}</div>
                )}
                <div className="process-pid">
                  <span className="pid-label">PID:</span>
                  <span className="pid-number">{process.pid}</span>
                </div>
              </div>
            );
          })
        )}
      </div>
      <div className="attach-controls">
        <button 
          className="attach-btn"
          onClick={() => selectedProcess && onAttach(selectedProcess)}
          disabled={!selectedProcess || tracerStatus.status === "ATTACHED"}
        >
          ▶ Attach
        </button>
        <button 
          className="detach-btn"
          onClick={onDetach}
          disabled={tracerStatus.status !== "ATTACHED"}
        >
          ⏹ Detach
        </button>
      </div>
    </div>
  );
}
