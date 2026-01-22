export function Settings({ 
  show, 
  fridaPath, 
  deviceId, 
  onFridaPathChange, 
  onDeviceIdChange, 
  onSave, 
  onCancel 
}) {
  if (!show) return null;

  return (
    <div className="settings-modal">
      <div className="settings-content">
        <h2>Frida Configuration</h2>
        
        <div className="setting-group">
          <label>Frida Path (Optional)</label>
          <input
            type="text"
            value={fridaPath}
            onChange={(e) => onFridaPathChange(e.target.value)}
            placeholder=""
          />
          <small>If Frida is in a virtual environment, enter the full path</small>
        </div>

        <div className="setting-group">
          <label>Device ID</label>
          <input
            type="text"
            value={deviceId}
            onChange={(e) => onDeviceIdChange(e.target.value)}
            placeholder=""
          />
          <small>Usually "usb" for USB devices, or enter specific device ID like "emulator-5554"</small>
        </div>

        <div className="settings-help">
          <h4>How to find Frida path?</h4>
          <p><strong>Windows (Virtual Environment):</strong></p>
          <code>C:\path\to\venv\Scripts\frida.exe</code>
          <p><strong>Linux/Mac (Virtual Environment):</strong></p>
          <code>/path/to/venv/bin/frida</code>
          <p><strong>Find command:</strong></p>
          <code>where frida (Windows) or which frida (Linux/Mac)</code>
        </div>

        <div className="settings-actions">
          <button onClick={onSave}>Save</button>
          <button onClick={onCancel}>Cancel</button>
        </div>
      </div>
    </div>
  );
}
