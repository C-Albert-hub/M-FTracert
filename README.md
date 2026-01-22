# M-FTracert

<div align="center">

**一个强大的 Android 应用方法追踪工具**

基于 Frida 的实时方法调用追踪器，提供直观的树形视图和详细的调用栈分析

[功能特性](#功能特性) • [快速开始](#快速开始) • [使用指南](#使用指南) • [技术栈](#技术栈)

</div>

---

## � 界面预览

- **进程选择**：清晰的三行布局显示应用名、包名和 PID
- **方法追踪**：实时显示方法调用的树形结构
- **调用栈分析**：右键查看完整的方法调用链路
- **智能搜索**：支持多关键词模糊搜索，快速定位目标方法

## ✨ 功能特性

### 核心功能

- 🎯 **实时方法追踪**：基于 Frida 动态 Hook，实时捕获方法调用
- 🌲 **树形视图**：直观展示方法调用的层级关系
- 🔍 **智能搜索**：支持类名、方法名、参数、返回值的模糊搜索
- 📊 **调用栈分析**：右键查看从根节点到当前方法的完整调用链
- 🎨 **语法高亮**：方法、参数、返回值使用不同颜色区分

### 规则管理

- ✅ **匹配规则**：使用正则表达式指定要追踪的类
- ❌ **黑名单规则**：排除不需要追踪的系统类，提高性能
- 🔄 **动态更新**：规则修改后自动重新 Hook

### 界面特性

- 🎭 **双标签页**：Method Calls（方法调用）和 System Hooks（系统日志）分离显示
- 🧹 **分类清除**：可以单独清除方法调用或系统日志
- 📱 **响应式布局**：窗口大小自适应，支持拖动调整
- 🌈 **配色优化**：精心设计的配色方案，长时间使用不疲劳

## 🚀 快速开始

### 环境要求

- **操作系统**：Windows / macOS / Linux
- **Node.js**：v16 或更高版本
- **Rust**：最新稳定版
- **Frida**：需要在系统中安装 frida-tools
- **Android 设备**：已 root 并运行 frida-server

### 安装 Frida

```bash
# 使用 pip 安装 frida-tools
pip install frida-tools

# 验证安装
frida --version
```

### 启动 frida-server

1. 下载对应架构的 frida-server：https://github.com/frida/frida/releases
2. 推送到 Android 设备：
   ```bash
   adb push frida-server /data/local/tmp/
   adb shell "chmod 755 /data/local/tmp/frida-server"
   adb shell "/data/local/tmp/frida-server &"
   ```

### 运行项目

```bash
# 克隆项目
git clone <repository-url>
cd m-ftracert

# 安装依赖
npm install

# 开发模式运行
npm run tauri dev

# 构建生产版本
npm run tauri build
```

## 📖 使用指南

### 1. 配置 Frida 路径

首次使用需要配置 Frida 路径：

1. 点击右上角的 **Settings** 按钮
2. 设置 Frida 可执行文件路径（如：`C:\Python\Scripts\frida.exe`）
3. 设置设备 ID（默认 `usb` 表示 USB 连接的设备）
4. 点击 **Save** 保存

### 2. 选择目标进程

1. 点击 **Refresh** 按钮加载进程列表
2. 选择要追踪的应用进程
3. 点击 **▶ Attach** 附加到进程

### 3. 配置追踪规则

**Match Rules（匹配规则）**：
```
com.example.myapp.*          # 追踪整个包
com.example.MainActivity     # 追踪特定类
com.example.utils.Crypto.*  # 追踪子包
```

**Blacklist Rules（黑名单规则）**：
```
android.view.*               # 排除 Android 视图类
android.widget.*             # 排除 Android 控件类
java.lang.*                  # 排除 Java 基础类
```

### 4. 查看追踪结果

- **Method Calls 标签页**：查看方法调用的树形结构
  - 展开/折叠节点查看调用层级
  - 右键点击节点查看详细信息或调用栈
  - 使用搜索框快速定位目标方法

- **System Hooks 标签页**：查看 Hook 日志和系统消息

### 5. 搜索功能

支持多种搜索方式：
- 类名搜索：`MainActivity`
- 方法名搜索：`onClick`
- 完整路径：`com.example.MainActivity.onCreate`
- 多关键词：`main click`（同时包含 main 和 click）

### 6. 调用栈分析

1. 在 Method Calls 标签页中右键点击任意方法
2. 选择 **View Call Stack**
3. 查看从根节点到当前方法的完整调用链路

## 🛠️ 技术栈

### 前端
- **React 18**：现代化的 UI 框架
- **Vite**：快速的构建工具
- **CSS3**：精心设计的样式系统

### 后端
- **Tauri**：轻量级桌面应用框架
- **Rust**：高性能的系统编程语言
- **Frida**：动态插桩框架

### 核心技术
- **Frida JavaScript API**：动态 Hook 和方法追踪
- **进程间通信**：Tauri 的事件系统
- **实时数据流**：基于事件的异步架构

## 📁 项目结构

```
m-ftracert/
├── src/                      # 前端源码
│   ├── components/          # React 组件
│   │   ├── ProcessList.jsx  # 进程列表
│   │   ├── RuleManager.jsx  # 规则管理
│   │   ├── TreeEventViewer.jsx  # 树形视图
│   │   └── Settings.jsx     # 设置面板
│   ├── hooks/               # React Hooks
│   │   └── useFridaTracer.js  # Frida 追踪逻辑
│   ├── App.jsx              # 主应用组件
│   └── App.css              # 样式文件
├── src-tauri/               # Tauri 后端
│   ├── src/
│   │   ├── commands.rs      # Tauri 命令
│   │   ├── frida_manager.rs # Frida 管理器
│   │   ├── script_generator.rs  # 脚本生成器
│   │   ├── models.rs        # 数据模型
│   │   └── main.rs          # 入口文件
│   └── Cargo.toml           # Rust 依赖
├── package.json             # Node.js 依赖
└── README.md                # 项目文档
```

## ⚙️ 配置说明

### Frida 配置

配置文件位置：`~/.m-ftracert/config.json`

```json
{
  "frida_path": "C:\\Python\\Scripts\\frida.exe",
  "device_id": "usb",
  "match_rules": [
    "com.example.myapp.*"
  ],
  "blacklist_rules": [
    "android.view.*",
    "android.widget.*"
  ]
}
```

### 窗口配置

- 初始大小：1200x800
- 最小大小：900x600
- 支持调整大小和最大化

## 🎯 使用场景

- **安全分析**：分析 Android 应用的加密算法和数据处理流程
- **逆向工程**：理解应用的内部逻辑和方法调用关系
- **性能优化**：识别性能瓶颈和频繁调用的方法
- **漏洞挖掘**：追踪敏感数据的流向和处理过程
- **学习研究**：学习 Android 应用的实现原理

## ⚠️ 注意事项

1. **合法使用**：仅用于学习研究和授权的安全测试
2. **性能影响**：避免使用 `.*` 匹配所有类，会导致大量事件
3. **设备要求**：需要 root 权限和 frida-server
4. **规则优化**：合理配置黑名单规则以提高性能

## 🤝 贡献指南

欢迎提交 Issue 和 Pull Request！

## 📄 许可证

本项目仅供学习研究使用。

## 🙏 致谢

- [Frida](https://frida.re/) - 强大的动态插桩框架
- [Tauri](https://tauri.app/) - 现代化的桌面应用框架
- [React](https://react.dev/) - 优秀的 UI 框架
- [ZenTracer](https://github.com/hluwa/ZenTracer) - an android method tracer gui tool base-on frida
---

<div align="center">

**如果这个项目对你有帮助，请给个 ⭐️ Star 支持一下！**

</div>
