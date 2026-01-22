# 发布指南

## 构建产物说明

运行 `npm run tauri build` 后，会在 `src-tauri/target/release/bundle/` 目录下生成多种格式的安装包：

### Windows 平台

```
src-tauri/target/release/
├── M-FTracert.exe                    # 可执行文件（绿色版）
└── bundle/
    ├── msi/
    │   └── M-FTracert_0.1.0_x64_en-US.msi    # MSI 安装包（推荐）
    └── nsis/
        └── M-FTracert_0.1.0_x64-setup.exe    # NSIS 安装包
```

## 推荐的发布方式

### 方案 1：发布 MSI 安装包（推荐）

**优点：**
- ✅ 标准的 Windows 安装程序
- ✅ 自动创建开始菜单快捷方式
- ✅ 支持卸载
- ✅ 用户体验好

**发布文件：**
```
M-FTracert_0.1.0_x64_en-US.msi
```

**用户使用：**
1. 双击 .msi 文件
2. 按照向导安装
3. 从开始菜单启动

### 方案 2：发布绿色版（便携版）

**优点：**
- ✅ 无需安装，解压即用
- ✅ 适合临时使用
- ✅ 文件小

**发布文件：**
```
M-FTracert-v0.1.0-portable.zip
```

**打包步骤：**
```bash
# 创建发布目录
mkdir M-FTracert-v0.1.0-portable
cd M-FTracert-v0.1.0-portable

# 复制必要文件
copy ..\src-tauri\target\release\M-FTracert.exe .
copy ..\README.md .
copy ..\LICENSE .

# 创建使用说明
echo "双击 M-FTracert.exe 运行程序" > 使用说明.txt

# 压缩成 zip
# 使用 7-Zip 或 WinRAR 压缩
```

### 方案 3：同时发布两种版本（最佳）

在 GitHub Release 中同时提供：

1. **M-FTracert_0.1.0_x64_en-US.msi** - 安装版
2. **M-FTracert-v0.1.0-portable.zip** - 绿色版

## GitHub Release 发布流程

### 1. 创建 Release

```bash
# 1. 确保代码已提交
git add .
git commit -m "Release v0.1.0"
git push

# 2. 创建标签
git tag v0.1.0
git push origin v0.1.0
```

### 2. 在 GitHub 上创建 Release

1. 访问你的仓库页面
2. 点击 "Releases" → "Create a new release"
3. 填写信息：
   - **Tag version**: `v0.1.0`
   - **Release title**: `M-FTracert v0.1.0`
   - **Description**: 参考下面的模板

### 3. Release 描述模板

```markdown
## M-FTracert v0.1.0

### 🎉 首次发布

基于 Frida 的 Android 应用方法追踪工具，提供直观的树形视图和详细的调用栈分析。

### ✨ 主要功能

- 🎯 实时方法追踪
- 🌲 树形视图展示调用关系
- 🔍 智能搜索功能
- 📊 调用栈分析
- 🎨 语法高亮显示

### 📦 下载

#### Windows 用户

**安装版（推荐）：**
- [M-FTracert_0.1.0_x64_en-US.msi](链接) - 标准安装程序

**绿色版（便携版）：**
- [M-FTracert-v0.1.0-portable.zip](链接) - 解压即用

### 📋 系统要求

- Windows 10/11 (64-bit)
- [WebView2 运行时](https://go.microsoft.com/fwlink/p/?LinkId=2124703)
- [Visual C++ Redistributable](https://aka.ms/vs/17/release/vc_redist.x64.exe)
- Frida 和 frida-server

### 🚀 快速开始

1. 安装 Frida：`pip install frida-tools`
2. 在 Android 设备上运行 frida-server
3. 运行 M-FTracert
4. 配置 Frida 路径
5. 选择进程并开始追踪

详细使用说明请查看 [README.md](链接)

### 🐛 已知问题

- 首次 attach 可能无法 hook 到某些延迟加载的类（需要 detach 后重新 attach）

### 📝 更新日志

- 初始版本发布
- 实现核心追踪功能
- 添加树形视图
- 添加调用栈分析
- 优化界面和配色

---

**完整文档：** [README.md](链接)
**问题反馈：** [Issues](链接)
```

### 4. 上传文件

在 Release 页面的 "Attach binaries" 区域，拖拽上传：

1. `M-FTracert_0.1.0_x64_en-US.msi`
2. `M-FTracert-v0.1.0-portable.zip`
3. （可选）`checksums.txt` - 文件校验和

### 5. 生成校验和（可选但推荐）

```bash
# Windows PowerShell
Get-FileHash M-FTracert_0.1.0_x64_en-US.msi -Algorithm SHA256
Get-FileHash M-FTracert-v0.1.0-portable.zip -Algorithm SHA256

# 保存到 checksums.txt
```

## 自动化发布（GitHub Actions）

创建 `.github/workflows/release.yml`：

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: 18
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Install dependencies
        run: npm install
      
      - name: Build
        run: npm run tauri build
      
      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            src-tauri/target/release/bundle/msi/*.msi
            src-tauri/target/release/bundle/nsis/*.exe
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

## 版本号管理

### 更新版本号

需要同时更新以下文件：

1. **package.json**
```json
{
  "version": "0.1.0"
}
```

2. **src-tauri/tauri.conf.json**
```json
{
  "package": {
    "version": "0.1.0"
  }
}
```

3. **src-tauri/Cargo.toml**
```toml
[package]
version = "0.1.0"
```

### 版本号规范

遵循语义化版本（Semantic Versioning）：

- **主版本号**：不兼容的 API 修改
- **次版本号**：向下兼容的功能性新增
- **修订号**：向下兼容的问题修正

示例：
- `v0.1.0` - 初始版本
- `v0.1.1` - Bug 修复
- `v0.2.0` - 新功能
- `v1.0.0` - 正式版本

## 发布检查清单

发布前确认：

- [ ] 所有功能正常工作
- [ ] 没有已知的严重 bug
- [ ] README.md 已更新
- [ ] 版本号已更新（3 个文件）
- [ ] 代码已提交并推送
- [ ] 创建了 git tag
- [ ] 构建成功且测试通过
- [ ] 准备好 Release 说明
- [ ] 生成了校验和

## 用户安装指南

在 README.md 中添加：

### 安装方式 1：MSI 安装包

1. 下载 `M-FTracert_0.1.0_x64_en-US.msi`
2. 双击运行安装程序
3. 按照向导完成安装
4. 从开始菜单启动 M-FTracert

### 安装方式 2：绿色版

1. 下载 `M-FTracert-v0.1.0-portable.zip`
2. 解压到任意目录
3. 双击 `M-FTracert.exe` 运行

### 依赖安装

如果程序无法启动，请安装以下依赖：

1. **WebView2 运行时**（必需）
   - 下载：https://go.microsoft.com/fwlink/p/?LinkId=2124703
   
2. **Visual C++ Redistributable**（必需）
   - 下载：https://aka.ms/vs/17/release/vc_redist.x64.exe

3. **Frida**（必需）
   ```bash
   pip install frida-tools
   ```

## 总结

**推荐发布方式：**
1. 构建项目：`npm run tauri build`
2. 打包绿色版：将 .exe 和文档打包成 .zip
3. 在 GitHub 创建 Release
4. 上传 .msi 和 .zip 文件
5. 填写详细的 Release 说明

这样用户可以根据自己的需求选择安装方式！
