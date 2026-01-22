# M-FTracert 更新日志

## [当前版本] - 2025-01-21

### ✨ 新增功能

#### 1. 进程监控和自动分离
- ✅ 当附加的进程关闭时，自动检测并提示用户
- ✅ 在事件日志中显示 "⚠️ Target process closed, please detach manually"
- ✅ 前端定时检查状态，自动更新 UI

#### 2. 增强的调试日志
- ✅ 后端添加详细的 Frida 输出日志（`📥 Frida output`）
- ✅ 后端添加事件解析日志（`✅ Parsed event`）
- ✅ 前端添加事件接收日志（`📨 Received trace_event`）
- ✅ 前端添加事件统计日志（`📊 Total events`）
- ✅ 组件渲染日志（`🎨 EventViewer render`）

#### 3. 协议优化
- ✅ 消息前缀从 `ZenTracer:::` 改为 `tracer::`
- ✅ 更简洁的协议格式
- ✅ 保持完全向后兼容

### 🐛 Bug 修复

#### 1. UI 渲染问题
- ✅ 修复事件接收但不显示的问题
- ✅ 添加 `getEventType()` 函数正确识别 hooking 事件
- ✅ 修复 React key 属性，使用 `${event.id}-${index}` 确保唯一性
- ✅ 添加自动滚动到底部功能

#### 2. Tauri API 兼容性
- ✅ 修复 `window.__TAURI_IPC__ is not a function` 错误
- ✅ 确保使用正确的 Tauri v1 API
- ✅ 优化事件监听器初始化流程

#### 3. 异步编程问题
- ✅ 修复 Rust MutexGuard 跨 await 的生命周期问题
- ✅ 使用 `tokio::task::spawn_blocking` 避免阻塞
- ✅ 正确处理进程输出流的异步读取

### 🏗️ 架构改进

#### 1. 后端模块化
创建清晰的模块结构：
- `models.rs` - 所有数据模型定义
- `frida_manager.rs` - Frida 进程管理
- `script_generator.rs` - Frida 脚本生成
- `event_parser.rs` - 事件解析逻辑
- `commands.rs` - Tauri 命令处理
- `main.rs` - 简洁的主入口（仅 70 行）

#### 2. 前端模块化
创建可复用的组件：
- `ProcessList.jsx` - 进程列表组件
- `RuleManager.jsx` - 规则管理组件（可复用）
- `EventViewer.jsx` - 事件查看器组件
- `Settings.jsx` - 设置面板组件
- `useFridaTracer.js` - 自定义 Hook 封装业务逻辑

#### 3. 代码质量
- ✅ 所有 TypeScript/JavaScript 诊断检查通过
- ✅ 所有 Rust 编译检查通过（仅有 1 个可忽略的警告）
- ✅ 组件间通过 props 通信，保持单向数据流
- ✅ 使用 React Hooks 管理状态和副作用

### 📝 文档完善

#### 新增文档
1. `MODULAR_STRUCTURE.md` - 模块化结构说明
2. `PROTOCOL_CHANGE.md` - 协议变更说明
3. `TESTING_GUIDE.md` - 详细的测试指南
4. `CHANGELOG.md` - 更新日志（本文件）

#### 更新文档
- `README.md` - 项目说明
- `FRIDA_SETUP.md` - Frida 配置指南
- `QUICK_START.md` - 快速开始指南
- `TROUBLESHOOTING.md` - 故障排除指南

### 🔧 技术栈

#### 前端
- React 18 + Vite
- Tauri API v1
- 自定义 Hooks
- 模块化组件设计

#### 后端
- Rust + Tauri 1.x
- Tokio 异步运行时
- Frida 集成
- GBK 编码支持（Windows 中文环境）

### 📊 性能优化

- ✅ 事件列表只显示最近 100 条（避免内存溢出）
- ✅ 使用 `tokio::spawn_blocking` 避免阻塞主线程
- ✅ 优化事件解析性能
- ✅ 减少不必要的状态更新

### 🎯 已知问题

#### 1. 自动分离功能
- ⚠️ 当前只能检测进程退出并提示，需要手动点击 Detach
- 📝 原因：Tauri State 的生命周期限制，无法在 tokio::spawn 中直接修改
- 💡 解决方案：用户看到提示后手动 Detach，或者使用事件通知机制

#### 2. 未使用的方法警告
- ⚠️ `frida_manager.rs` 中的 `get_processes` 方法未使用
- 📝 原因：为了避免 MutexGuard 跨 await 问题，在 commands.rs 中直接实现
- 💡 解决方案：可以删除该方法，或者保留作为备用

### 🚀 下一步计划

#### 短期目标
- [ ] 实现真正的自动 Detach（使用事件机制）
- [ ] 添加事件过滤功能
- [ ] 添加事件导出功能（JSON/CSV）
- [ ] 添加搜索功能

#### 中期目标
- [ ] 支持多进程同时追踪
- [ ] 添加调用栈可视化
- [ ] 添加性能分析功能
- [ ] 支持自定义脚本

#### 长期目标
- [ ] 支持 iOS 设备
- [ ] 添加插件系统
- [ ] 云端协作功能
- [ ] AI 辅助分析

### 🙏 致谢

本项目参考了 [ZenTracer](https://github.com/hluwa/ZenTracer) 的设计思路，特此感谢原作者。

### 📄 许可证

本项目采用 MIT 许可证。

---

## 版本历史

### v0.1.0 - 初始版本
- 基础的 Frida 集成
- 进程列表和附加功能
- 事件追踪和显示
- 规则管理
- 设置面板
