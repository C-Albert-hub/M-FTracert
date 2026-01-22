import { useRef, useEffect, useState } from "react";

// 右键菜单组件
function ContextMenu({ x, y, node, onClose, onShowDetail, onShowCallStack }) {
  const menuRef = useRef(null);
  
  useEffect(() => {
    const handleClickOutside = (e) => {
      if (menuRef.current && !menuRef.current.contains(e.target)) {
        onClose();
      }
    };
    
    document.addEventListener('mousedown', handleClickOutside);
    return () => document.removeEventListener('mousedown', handleClickOutside);
  }, [onClose]);
  
  const handleShowDetail = () => {
    onShowDetail(node);
    onClose();
  };
  
  const handleShowCallStack = () => {
    onShowCallStack(node);
    onClose();
  };
  
  const handleCopyMethod = () => {
    const text = node.event_type === "enter" 
      ? `${node.class_name}.${node.method_name}`
      : node.message;
    navigator.clipboard.writeText(text);
    onClose();
  };
  
  const handleCopyArgs = () => {
    if (node.args && node.args.length > 0) {
      navigator.clipboard.writeText(JSON.stringify(node.args, null, 2));
    }
    onClose();
  };
  
  const handleCopyRetval = () => {
    if (node.return_value) {
      navigator.clipboard.writeText(node.return_value);
    }
    onClose();
  };
  
  const handleCopyAll = () => {
    const text = JSON.stringify({
      thread: `${node.thread_id} - ${node.thread_name}`,
      class: node.class_name,
      method: node.method_name,
      args: node.args,
      return_value: node.return_value,
      timestamp: node.timestamp
    }, null, 2);
    navigator.clipboard.writeText(text);
    onClose();
  };
  
  return (
    <div 
      ref={menuRef}
      className="context-menu" 
      style={{ left: `${x}px`, top: `${y}px` }}
    >
      <div className="context-menu-item" onClick={handleShowDetail}>
        View Details
      </div>
      <div className="context-menu-item" onClick={handleShowCallStack}>
        View Call Stack
      </div>
      <div className="context-menu-divider"></div>
      <div className="context-menu-item" onClick={handleCopyMethod}>
        Copy Method
      </div>
      {node.args && node.args.length > 0 && (
        <div className="context-menu-item" onClick={handleCopyArgs}>
          Copy Arguments
        </div>
      )}
      {node.return_value && (
        <div className="context-menu-item" onClick={handleCopyRetval}>
          Copy Return Value
        </div>
      )}
      <div className="context-menu-divider"></div>
      <div className="context-menu-item" onClick={handleCopyAll}>
        Copy All
      </div>
    </div>
  );
}

// 调用栈弹窗组件
function CallStackModal({ callStack, onClose }) {
  if (!callStack) return null;
  
  return (
    <div className="detail-modal-overlay" onClick={onClose}>
      <div className="detail-modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="detail-modal-header">
          <h3>Call Stack</h3>
          <button className="detail-modal-close" onClick={onClose}>✕</button>
        </div>
        <div className="detail-modal-body">
          <div className="call-stack-list">
            {callStack.map((frame, index) => (
              <div key={index} className="call-stack-frame">
                <div className="call-stack-index">#{index + 1}</div>
                <div className="call-stack-info">
                  <div className="call-stack-method">
                    {frame.class_name}.{frame.method_name}
                  </div>
                  {frame.args && frame.args.length > 0 && (
                    <div className="call-stack-args">
                      Args: [{frame.args.join(", ")}]
                    </div>
                  )}
                  {frame.return_value && (
                    <div className="call-stack-retval">
                      Return: {frame.return_value}
                    </div>
                  )}
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}

// 详情弹窗组件
function DetailModal({ node, onClose }) {
  if (!node) return null;
  
  return (
    <div className="detail-modal-overlay" onClick={onClose}>
      <div className="detail-modal-content" onClick={(e) => e.stopPropagation()}>
        <div className="detail-modal-header">
          <h3>Method Details</h3>
          <button className="detail-modal-close" onClick={onClose}>✕</button>
        </div>
        <div className="detail-modal-body">
          <div className="detail-row">
            <span className="detail-label">Thread:</span>
            <span className="detail-value">{node.thread_id} - {node.thread_name}</span>
          </div>
          <div className="detail-row">
            <span className="detail-label">Class:</span>
            <span className="detail-value">{node.class_name || "N/A"}</span>
          </div>
          <div className="detail-row">
            <span className="detail-label">Method:</span>
            <span className="detail-value">{node.method_name || node.message}</span>
          </div>
          <div className="detail-row">
            <span className="detail-label">Arguments:</span>
            <pre className="detail-value-pre">
              {node.args && node.args.length > 0 
                ? JSON.stringify(node.args, null, 2)
                : "No arguments"}
            </pre>
          </div>
          <div className="detail-row">
            <span className="detail-label">Return Value:</span>
            <pre className="detail-value-pre">
              {node.return_value !== null && node.return_value !== undefined
                ? node.return_value
                : "N/A"}
            </pre>
          </div>
          <div className="detail-row">
            <span className="detail-label">Timestamp:</span>
            <span className="detail-value">{node.timestamp}</span>
          </div>
        </div>
      </div>
    </div>
  );
}

// 简单日志节点组件（用于 System Hooks）
function LogNode({ node }) {
  return (
    <div className="log-node">
      <span className="log-time">{node.timestamp}</span>
      <span className="log-message">{node.message}</span>
    </div>
  );
}

// 树节点组件（用于 Method Calls）
function TreeNode({ node, level = 0, onShowDetail, onShowContextMenu }) {
  const [isExpanded, setIsExpanded] = useState(true);
  
  const hasChildren = node.children && node.children.length > 0;
  const indent = level * 20;
  
  const getNodeClass = () => {
    if (node.event_type === "enter") return "tree-node-enter";
    if (node.event_type === "exit") return "tree-node-exit";
    if (node.event_type === "log" && node.message?.includes("hooking:")) return "tree-node-hook";
    return "tree-node-log";
  };
  
  const formatNodeText = () => {
    if (node.event_type === "enter") {
      return `${node.class_name}.${node.method_name}`;
    } else if (node.event_type === "log") {
      return node.message;
    }
    return node.message;
  };
  
  const formatArgs = () => {
    if (node.args && node.args.length > 0) {
      return `[${node.args.join(", ")}]`;
    }
    return "";
  };
  
  const formatRetval = () => {
    if (node.return_value !== null && node.return_value !== undefined) {
      return node.return_value;
    }
    return "";
  };
  
  const handleContextMenu = (e) => {
    e.preventDefault();
    onShowContextMenu(e.clientX, e.clientY, node);
  };
  
  return (
    <div className="tree-node">
      <div 
        className={`tree-node-row ${getNodeClass()}`}
        onContextMenu={handleContextMenu}
      >
        {/* Method 列 */}
        <div className="tree-col-method" style={{ paddingLeft: `${indent}px` }}>
          {hasChildren && (
            <span 
              className="tree-toggle"
              onClick={() => setIsExpanded(!isExpanded)}
            >
              {isExpanded ? "▼" : "▶"}
            </span>
          )}
          {!hasChildren && <span className="tree-spacer">　</span>}
          <span className="tree-method-text">{formatNodeText()}</span>
        </div>
        
        {/* 分割线 */}
        <div className="tree-col-divider">|</div>
        
        {/* Args 列 */}
        <div className="tree-col-args">
          {formatArgs()}
        </div>
        
        {/* 分割线 */}
        <div className="tree-col-divider">|</div>
        
        {/* Retval 列 */}
        <div className="tree-col-retval">
          {formatRetval()}
        </div>
      </div>
      
      {hasChildren && isExpanded && (
        <div className="tree-children">
          {node.children.map((child, index) => (
            <TreeNode 
              key={`${child.id}-${index}`} 
              node={child} 
              level={level + 1}
              onShowDetail={onShowDetail}
              onShowContextMenu={onShowContextMenu}
            />
          ))}
        </div>
      )}
    </div>
  );
}

// 线程树组件
function ThreadTree({ threadId, threadName, rootNodes, onShowDetail, onShowContextMenu, isLogView = false }) {
  const [isExpanded, setIsExpanded] = useState(true);
  
  return (
    <div className="thread-tree">
      <div 
        className="thread-header"
        onClick={() => setIsExpanded(!isExpanded)}
      >
        <span className="tree-toggle">{isExpanded ? "▼" : "▶"}</span>
        <span className="thread-name">{threadId} - {threadName}</span>
      </div>
      
      {isExpanded && (
        <div className="thread-content">
          {isLogView ? (
            // System Hooks 使用简单列表
            rootNodes.map((node, index) => (
              <LogNode key={`${node.id}-${index}`} node={node} />
            ))
          ) : (
            // Method Calls 使用树形结构
            rootNodes.map((node, index) => (
              <TreeNode 
                key={`${node.id}-${index}`} 
                node={node} 
                level={0}
                onShowDetail={onShowDetail}
                onShowContextMenu={onShowContextMenu}
              />
            ))
          )}
        </div>
      )}
    </div>
  );
}

export function TreeEventViewer({ events, onClear }) {
  const [threadTrees, setThreadTrees] = useState({});
  const [selectedNode, setSelectedNode] = useState(null);
  const [callStack, setCallStack] = useState(null);
  const [contextMenu, setContextMenu] = useState(null);
  const [activeTab, setActiveTab] = useState("threads"); // "threads" 或 "hooks"
  const [searchText, setSearchText] = useState(""); // 搜索文本
  const containerRef = useRef(null);

  // 处理清除按钮点击
  const handleClear = () => {
    if (activeTab === "threads") {
      onClear("method"); // 清除 Method Calls
    } else if (activeTab === "hooks") {
      onClear("log"); // 清除 System Hooks
    }
  };

  // 统计不同类型的事件数量
  const methodCallsCount = events.filter(e => e.event_type === "enter" || e.event_type === "exit").length;
  const systemHooksCount = events.filter(e => e.event_type === "log").length;
  
  // 构建调用栈
  const buildCallStack = (node, trees) => {
    const stack = [];
    
    // 递归查找节点的父节点路径
    const findPath = (currentNode, targetNode, path = []) => {
      if (currentNode === targetNode) {
        return [...path, currentNode];
      }
      
      if (currentNode.children && currentNode.children.length > 0) {
        for (const child of currentNode.children) {
          const result = findPath(child, targetNode, [...path, currentNode]);
          if (result) return result;
        }
      }
      
      return null;
    };
    
    // 在所有线程树中查找
    for (const tree of Object.values(trees)) {
      for (const rootNode of tree.rootNodes) {
        const path = findPath(rootNode, node);
        if (path) {
          return path.filter(n => n.event_type === "enter");
        }
      }
    }
    
    return [node];
  };
  
  const handleShowCallStack = (node) => {
    const stack = buildCallStack(node, threadTrees);
    setCallStack(stack);
  };

  // 构建树形结构
  useEffect(() => {
    const trees = {};
    const threadStacks = {}; // 每个线程的调用栈
    const seenEventIds = new Set(); // 防止重复事件
    
    events.forEach(event => {
      // 跳过重复事件
      if (seenEventIds.has(event.id)) {
        return;
      }
      seenEventIds.add(event.id);
      
      if (event.event_type === "log") {
        // 日志事件添加到 logs 分组（用于显示 hooking 信息）
        const threadKey = "logs";
        if (!trees[threadKey]) {
          trees[threadKey] = {
            threadId: "System",
            threadName: "Hooks",
            rootNodes: []
          };
        }
        trees[threadKey].rootNodes.push({
          ...event,
          children: []
        });
      } else if (event.event_type === "enter") {
        // 方法进入
        const threadKey = `${event.thread_id}`;
        if (!trees[threadKey]) {
          trees[threadKey] = {
            threadId: event.thread_id,
            threadName: event.thread_name || "Unknown",
            rootNodes: []
          };
          threadStacks[threadKey] = [];
        }
        
        const node = {
          ...event,
          children: [],
          return_value: null
        };
        
        // 如果栈为空，添加到根节点
        if (threadStacks[threadKey].length === 0) {
          trees[threadKey].rootNodes.push(node);
        } else {
          // 否则添加到栈顶节点的子节点
          const parent = threadStacks[threadKey][threadStacks[threadKey].length - 1];
          parent.children.push(node);
        }
        
        // 压入栈
        threadStacks[threadKey].push(node);
      } else if (event.event_type === "exit") {
        // 方法退出
        const threadKey = `${event.thread_id}`;
        if (threadStacks[threadKey] && threadStacks[threadKey].length > 0) {
          // 弹出栈顶，设置返回值
          const node = threadStacks[threadKey].pop();
          node.return_value = event.return_value;
        }
      }
    });
    
    setThreadTrees(trees);
  }, [events]);

  const handleShowContextMenu = (x, y, node) => {
    setContextMenu({ x, y, node });
  };

  const handleCloseContextMenu = () => {
    setContextMenu(null);
  };

  // 过滤节点函数 - 支持模糊匹配
  const filterNode = (node, searchLower) => {
    if (!searchLower) return true;
    
    // 构建搜索文本：包含所有可能的匹配字段
    const searchableText = [
      node.method_name,
      node.class_name,
      node.message,
      node.class_name && node.method_name ? `${node.class_name}.${node.method_name}` : '',
      node.args ? node.args.join(' ') : '',
      node.return_value ? String(node.return_value) : ''
    ]
      .filter(Boolean)
      .join(' ')
      .toLowerCase();
    
    // 支持多个关键词搜索（空格分隔）
    const keywords = searchLower.split(/\s+/).filter(k => k.length > 0);
    
    // 所有关键词都要匹配（AND 逻辑）
    return keywords.every(keyword => searchableText.includes(keyword));
  };

  // 过滤树结构 - 只返回匹配的节点，不包含父节点和子节点
  const filterTree = (nodes, searchLower) => {
    if (!searchLower) return nodes;
    
    const results = [];
    
    const collectMatches = (node) => {
      // 检查当前节点是否匹配
      if (filterNode(node, searchLower)) {
        // 创建一个新节点，不包含子节点
        results.push({
          ...node,
          children: []
        });
      }
      
      // 递归检查子节点
      if (node.children && node.children.length > 0) {
        node.children.forEach(child => collectMatches(child));
      }
    };
    
    nodes.forEach(node => collectMatches(node));
    return results;
  };

  // 获取过滤后的树
  const getFilteredTrees = () => {
    // 如果没有搜索文本，返回所有树
    if (!searchText.trim()) return threadTrees;
    
    // 如果在 System Hooks 标签页，不应用搜索过滤
    if (activeTab === "hooks") return threadTrees;
    
    // 只对 Method Calls 标签页应用搜索
    const searchLower = searchText.toLowerCase();
    const filtered = {};
    
    Object.entries(threadTrees).forEach(([key, tree]) => {
      // 跳过 logs（System Hooks）
      if (key === "logs") {
        filtered[key] = tree;
        return;
      }
      
      const filteredNodes = filterTree(tree.rootNodes, searchLower);
      if (filteredNodes.length > 0) {
        filtered[key] = {
          ...tree,
          rootNodes: filteredNodes
        };
      }
    });
    
    return filtered;
  };

  const filteredTrees = getFilteredTrees();

  return (
    <div className="right-panel">
      {/* 标签页切换 */}
      <div className="tab-bar">
        <div className="tab-buttons">
          <button 
            className={`tab-button ${activeTab === "threads" ? "active" : ""}`}
            onClick={() => setActiveTab("threads")}
          >
            Method Calls ({methodCallsCount})
          </button>
          <button 
            className={`tab-button ${activeTab === "hooks" ? "active" : ""}`}
            onClick={() => setActiveTab("hooks")}
          >
            System Hooks ({systemHooksCount})
          </button>
        </div>
        
        <div className="tab-actions">
          {/* 搜索框 - 只在 Method Calls 标签页显示 */}
          {activeTab === "threads" && (
            <div className="search-box">
              <input
                type="text"
                placeholder="Search by class or method..."
                value={searchText}
                onChange={(e) => setSearchText(e.target.value)}
                className="search-input"
              />
              {searchText && (
                <button 
                  className="search-clear"
                  onClick={() => setSearchText("")}
                >
                  ✕
                </button>
              )}
            </div>
          )}
          
          <button className="clear-btn" onClick={handleClear}>
            Clear {activeTab === "threads" ? "Methods" : "Hooks"}
          </button>
        </div>
      </div>
      
      {/* 表格头 - 只在 Method Calls 标签页显示 */}
      {activeTab === "threads" && (
        <div className="tree-table-header">
          <div className="tree-table-col-method">method</div>
          <div className="tree-table-col-divider">|</div>
          <div className="tree-table-col-args">args</div>
          <div className="tree-table-col-divider">|</div>
          <div className="tree-table-col-retval">retval</div>
        </div>
      )}
      
      <div className="tree-container" ref={containerRef}>
        {Object.keys(filteredTrees).length === 0 ? (
          <div className="no-events">
            {searchText ? `No results found for "${searchText}"` : "No events yet. Attach to a process to start tracing."}
          </div>
        ) : (
          <div className="tree-list">
            {/* Method Calls 标签页 - 显示线程调用树 */}
            {activeTab === "threads" && (
              <>
                {Object.entries(filteredTrees)
                  .filter(([key]) => key !== "logs")
                  .map(([key, tree]) => (
                    <ThreadTree
                      key={key}
                      threadId={tree.threadId}
                      threadName={tree.threadName}
                      rootNodes={tree.rootNodes}
                      onShowDetail={setSelectedNode}
                      onShowContextMenu={handleShowContextMenu}
                    />
                  ))}
                {Object.keys(filteredTrees).filter(k => k !== "logs").length === 0 && (
                  <div className="no-events">
                    {searchText ? `No method calls found for "${searchText}"` : "No method calls yet. Trigger actions in the app to see traces."}
                  </div>
                )}
              </>
            )}
            
            {/* System Hooks 标签页 - 显示 Hook 日志 */}
            {activeTab === "hooks" && (
              <>
                {filteredTrees["logs"] ? (
                  <ThreadTree
                    key="logs"
                    threadId={filteredTrees["logs"].threadId}
                    threadName={filteredTrees["logs"].threadName}
                    rootNodes={filteredTrees["logs"].rootNodes}
                    onShowDetail={setSelectedNode}
                    onShowContextMenu={handleShowContextMenu}
                    isLogView={true}
                  />
                ) : (
                  <div className="no-events">
                    {searchText ? `No hooks found for "${searchText}"` : "No hooks yet. Attach to a process to see hooked methods."}
                  </div>
                )}
              </>
            )}
          </div>
        )}
      </div>
      
      {/* 右键菜单 */}
      {contextMenu && (
        <ContextMenu
          x={contextMenu.x}
          y={contextMenu.y}
          node={contextMenu.node}
          onClose={handleCloseContextMenu}
          onShowDetail={setSelectedNode}
          onShowCallStack={handleShowCallStack}
        />
      )}
      
      {/* 调用栈弹窗 */}
      {callStack && (
        <CallStackModal 
          callStack={callStack} 
          onClose={() => setCallStack(null)} 
        />
      )}
      
      {/* 详情弹窗 */}
      {selectedNode && (
        <DetailModal 
          node={selectedNode} 
          onClose={() => setSelectedNode(null)} 
        />
      )}
    </div>
  );
}
