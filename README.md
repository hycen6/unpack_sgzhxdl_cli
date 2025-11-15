# 三幻Spine动态立绘还原CLI工具

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

> 🦀 **本项目是 [unpack_sgzhxdl](https://github.com/hycen6/unpack_sgzhxdl) 的 Rust重构 CLI 版本**，提供更高性能、更友好的命令行界面和更强的并发处理能力。

## ✨ 特性

- **高性能**: 基于 Rust 实现，支持多线程并行处理，大幅提升处理速度
- **智能识别**: 自动识别文件类型（PNG、JSON、XML、Atlas、Skel）
- **自动归档**: 智能整理文件结构，自动创建目录并分类文件
- **强大搜索**: 支持Atlas和Skel文件内容搜索，支持模糊匹配
- **进度显示**: 实时显示处理进度和状态信息

### 重要提醒
- 请先按照 [三幻美术资源提取指南](./AboutArtsResource.md) 的指引提取游戏美术资源

## 🚀 快速开始

### 方式一：下载预编译可执行文件（推荐）

从 [GitHub Releases](https://github.com/hycen6/unpack_sgzhxdl_cli/releases) 下载对应平台的可执行文件：

#### Windows 用户
1. 下载 `unpack_sgzhxdl_cli-windows-x86_64.exe`
2. 直接双击运行或在命令行执行：
   ```cmd
   unpack_sgzhxdl_cli-windows-x86_64.exe
   ```
3. **首次使用可能遇到的安全提示**：
   - 如果遇到 Windows SmartScreen 阻止，点击"更多信息" → "仍要运行"
   - 如果遇到杀毒软件拦截，需要添加到白名单或信任列表
   - 这是正常的，因为可执行文件没有数字签名

#### macOS 用户
1. 下载 `unpack_sgzhxdl_cli-macos-aarch64（Apple Silicon）` 或 `unpack_sgzhxdl_cli-macos-x86_64（Intel）`（根据芯片选择）
2. **首次使用需要设置权限**：
   ```bash
   # 赋予可执行权限
   <!-- 苹果芯片 -->
   chmod +x unpack_sgzhxdl_cli-macos-aarch64
   <!-- 英特尔 -->
   chmod +x unpack_sgzhxdl_cli-macos-x86_64

   # 运行程序
   <!-- 苹果芯片 -->
   ./unpack_sgzhxdl_cli-macos-aarch64
   <!-- 英特尔 -->
   ./unpack_sgzhxdl_cli-macos-x86_64
   ```
3. 如果出现"无法打开，因为无法验证开发者"的提示：
   - 打开 **系统设置 → 隐私与安全性**
   - 找到"安全性"部分，点击"仍要打开"
   - 或者在 Finder 中右键点击应用 → "打开" → "打开"

#### Linux 用户
1. 下载 `unpack_sgzhxdl_cli-linux-aarch64`或`unpack_sgzhxdl_cli-linux-x86_64`（根据芯片选择）
2. 赋予可执行权限并运行：
   ```bash
   chmod +x unpack_sgzhxdl_cli-linux-aarch64
   ./unpack_sgzhxdl_cli-linux-aarch64
   ```

---

### 方式二：手动编译（适合开发者）

#### 安装前提
- Rust 1.70+ （推荐使用 [rustup](https://rustup.rs/) 安装）

#### 依赖说明
- `clap`: CLI参数解析
- `anyhow`: 错误处理
- `rayon`: 并行处理
- `walkdir`: 文件遍历
- `indicatif`: 进度条显示
- `console`: 终端样式
- `dialoguer`: 交互式界面

#### 编译安装
```bash
# 克隆项目
git clone https://github.com/hycen6/unpack_sgzhxdl_cli
cd unpack_sgzhxdl_cli

# 发布模式编译（性能优化，推荐使用）
cargo build --release
```

#### 运行程序
```bash
# 运行程序（推荐）
./target/release/unpack_sgzhxdl_cli

# 或直接指定工作目录
./target/release/unpack_sgzhxdl_cli --work-dir 美术资源目录（例如./miniRes或./udp）
# 指定工作目录简写
./target/release/unpack_sgzhxdl_cli -w 美术资源目录（例如./miniRes或./udp）
```

---

### 使用说明

无论使用哪种方式启动，程序都支持以下命令选项：

```bash
# 显示帮助信息
unpack_sgzhxdl_cli --help

# 直接指定工作目录
unpack_sgzhxdl_cli --work-dir 美术资源目录（例如./miniRes或./udp）
# 指定工作目录简写
unpack_sgzhxdl_cli -w 美术资源目录（例如./miniRes或./udp）
```



## 📋 使用流程

### 工作流程

```
选择工作目录 → 恢复扩展名 → 文件归档 → PNG重命名
```

### CLI交互式菜单

启动程序后，会显示中文交互菜单：

#### 选择工作目录
- 支持从当前目录的文件夹中选择
- 支持手动输入完整路径
- 自动验证目录存在性

### 处理资源说明

#### 1. 恢复文件扩展名
- 自动识别无扩展名的文件类型
- 支持格式：PNG、JSON、XML、Atlas、Skel
- 重名文件自动添加数字后缀

#### 2. 文件归类整理
- 自动创建atlas、skels文件夹并归类文件

#### 3. PNG文件重命名
- 按图片尺寸重命名：`size_宽度x高度.png`
- 重名文件自动添加数字后缀

## Spine动态立绘还原指南

### 还原步骤

#### 步骤1：处理资源文件
按[处理资源](#处理资源说明)完成资源的自动化处理

#### 步骤2：定位目标立绘
1. 在重命名后的图片中找到需要的角色立绘碎片
2. 记录图片尺寸（如：size_2017x1937.png）
3. 将图片移动到存储文件夹
   - 例如：`三幻立绘/SP孙策/size_2017x1937.png`

#### 3. 资源内容搜索
**Atlas搜索**:
- 按图片尺寸搜索相关Atlas文件
- 输入格式：`2017,1937`
- 将最终确定好的Atlas文件移动到存储文件夹
   - 例如：`三幻立绘/SP孙策/6c3caaaaad29cff6e2f06e92950ee759.atlas`

**Skel搜索**:
- 支持多个关键词同时搜索（空格分隔）
- 例如：`jianjia_shengzi_l_01 jianjia_shengzi_l_02 jiao_r lang_houtui_l_01`
- 将最终确定好的Skel文件移动到存储文件夹
   - 例如：`三幻立绘/SP孙策/99b6fec08bcf93a65b7919cd9b33ef02.skel`

#### 步骤4：文件整理和导入
1. 将骨骼图片重命名为 `skeleton.png`
   - 如有多张，按atlas文件中的描述命名。例如：

    ```
    # atlas文件内容
    skeleton.png
    size: 2017,1937
    skeleton1.png
    size: 300,169
    ```
    > 尺寸为2017,1937的图片重命名为：skeleton.png
    
    > 尺寸为300,169的图片重命名为：skeleton1.png
2. `.skel` 和 `.atlas` 文件可根据个人喜好命名
3. 导入支持Spine合成的软件（如Live2DViewerEx）

___

通过以上方法，您就能成功找到并还原喜欢的角色动态立绘！

动态立绘可用作个人搜藏/桌面动态壁纸/桌宠。
