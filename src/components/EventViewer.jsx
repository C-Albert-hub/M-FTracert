import { useRef, useEffect } from "react";

export function EventViewer({ events, onClear }) {
  const eventsEndRef = useRef(null);

  console.log("🎨 EventViewer render, events count:", events.length);

  const scrollToBottom = () => {
    eventsEndRef.current?.scrollIntoView({ behavior: "smooth" });
  };

  useEffect(() => {
    console.log("📜 Events updated, scrolling to bottom. Count:", events.length);
    scrollToBottom();
  }, [events]);

  const formatEventMessage = (event) => {
    if (event.event_type === "enter") {
      return `→ ${event.class_name}.${event.method_name}(${event.args ? event.args.join(", ") : ""})`;
    } else if (event.event_type === "exit") {
      return `← return: ${event.return_value || "void"}`;
    } else {
      return event.message || "";
    }
  };

  const getEventType = (event) => {
    if (event.message && event.message.includes("hooking:")) {
      return "hook";
    }
    return event.event_type || "log";
  };

  return (
    <div className="right-panel">
      <div className="events-header">
        <div>
          <h3>Tracer Events ({events.length})</h3>
          <p>Real-time method calls and system logs</p>
        </div>
        <button className="clear-btn" onClick={onClear}>
          Clear
        </button>
      </div>
      <div className="events-container">
        {events.length === 0 ? (
          <div className="no-events">
            No events yet. Attach to a process to start tracing.
          </div>
        ) : (
          <div className="events-list">
            {events.slice(-100).map((event, index) => (
              <div key={`${event.id}-${index}`} className={`event-item ${getEventType(event)}`}>
                <span className="event-time">{event.timestamp}</span>
                <span className="event-message">{formatEventMessage(event)}</span>
              </div>
            ))}
            <div ref={eventsEndRef} />
          </div>
        )}
      </div>
    </div>
  );
}
