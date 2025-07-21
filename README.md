# Kovi Minecraft MOTD 插件 🎮

[![协议: MPL-2.0](https://img.shields.io/badge/协议-MPL%202.0-brightgreen.svg)](https://www.mozilla.org/en-US/MPL/2.0/)
![Rust 版本](https://img.shields.io/badge/rust-1.70%2B-blue)

## 重要协议声明 ⚖️

### 代码来源
- `motd_be.rs` 和 `motd_je.rs` 通过AI工具从以下项目转换：
    - 原项目：[MCBE-Server-Motd](https://github.com/BlackBEDevelopment/MCBE-Server-Motd/)
    - 原语言：Go → 转换语言：Rust
    - 原协议：Mozilla Public License Version 2.0 (MPL-2.0)


为 [Kovi Bot](https://github.com/ThriceCola/Kovi) 设计的 Minecraft 服务器状态查询插件，支持 Java版 和 基岩版 双协议。

## 功能特性 ✨

- ✅ 双协议支持（JE/BE）
- 🖼️ Java版服务器图标显示
- 📊 基岩版完整服务器信息
- 🌐 自动端口检测（25565/19132）
- 🚀 基于 Rust 的异步实现

## 使用说明 💡

### 指令格式
```text
motd <IP或域名>[:端口]
```
### 查询示例
```text
motd mc.hypixel.net       # 查询Java版默认端口(25565)
motd 127.0.0.1:19132      # 强制查询基岩版(19132)
motd geyser.example.com   # 自动识别双端协议
```

## 安装指南 📦

### 环境要求
- 已安装 [Kovi Bot](https://github.com/ThriceCola/Kovi) 框架
- Rust 1.70+ 工具链

```bash
git clone https://github.com/Chlna6666/kovi-plugin-mc-motd.git
```

## 致谢 🙏

本项目基于以下开源项目构建，特此感谢原作者的贡献：

### 核心框架
- [Kovi Bot](https://github.com/ThriceCola/Kovi)  
  提供机器人核心功能支持  
  协议：MPL-2.0 license

### MOTD 协议实现
- [MCBE-Server-Motd](https://github.com/BlackBEDevelopment/MCBE-Server-Motd/)  
  原始 Go 语言实现的 Minecraft 服务器状态查询  
  协议：Mozilla Public License 2.0 (MPL-2.0)  
  转换说明：通过 AI 辅助工具将原 Go 代码转换为 Rust 实现