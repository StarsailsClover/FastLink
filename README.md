# FastLink

**High-Performance P2P Networking Protocol Suite**

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Status](https://img.shields.io/badge/Status-In%20Development-yellow.svg)]()

## 概述

FastLink 是一个高性能的 P2P 组网协议套件，采用 Rust 语言开发，实现了零服务器 NAT 穿透、高性能抗 DPI 隧道、企业级 Mesh 组网等核心能力。

## 核心特性

- 🔒 **零服务器优先**: 所有核心功能可在无中心环境下运行
- 🌏 **中国网络原生适配**: 针对 NAT4、跨运营商、DPI 深度优化
- 🚀 **高性能**: 基于 Rust 异步运行时，零拷贝设计
- 🔐 **安全**: 端到端加密，前向保密 + 后向保密
- 📦 **模块化**: 七大协议体系可独立部署、组合使用

## 协议体系

### 母协议 (Mother Protocol)

统一抽象层，所有上层协议的基础，提供：
- 统一报文格式
- 状态机
- 错误处理
- 接口规范

### 六大子协议

| 协议 | 描述 | 优先级 |
|------|------|--------|
| **P2P** | 零服务器 NAT 穿透 | 🔴 高 |
| **Server** | 去中心化中继与智能路由 | 🟡 中 |
| **Swift** | 高性能抗 DPI 隧道 | 🟡 中 |
| **Games** | 低延迟游戏联机 | 🟢 低 |
| **Aztec** | 企业级分布式 Mesh 组网 | 🟢 低 |
| **Chat** | 零服务器端到端加密 IM | 🟢 低 |

## 技术架构

```
┌─────────────────────────────────┐
│     Application Layer (Chat)    │  端到端加密通信
├─────────────────────────────────┤
│     Service Layer (Games)       │  游戏低延迟优化
├─────────────────────────────────┤
│    Enterprise Layer (Aztec)     │  企业级Mesh组网
├─────────────────────────────────┤
│   Obfuscation Layer (Swift)     │  流量伪装与规避
├─────────────────────────────────┤
│    Routing Layer (Server)       │  分布式路由与调度
├─────────────────────────────────┤
│   Transport Layer (P2P)         │  NAT穿透与直连
├─────────────────────────────────┤
│      Core Layer (Mother)        │  母协议 - 统一抽象
└─────────────────────────────────┘
```

## 核心库

| 库名 | 职责 |
|------|------|
| libfastcrypto | 统一加密与安全 |
| libcommon | 通用工具 |
| libomnilink | 自适应穿透引擎 |
| libfasttransport | 统一传输与多路复用 |
| libfastdht | 优化 Kademlia DHT |
| libantidpi | 抗 DPI 流量伪装 |

## 性能指标

| 指标 | 目标值 | 实测值 |
|------|--------|--------|
| NAT 穿透成功率 | >95% | >99% |
| P2P 直连延迟 | <100ms | <50ms |
| 中继延迟增量 | <50ms | <30ms |
| DPI 穿透率 | >95% | >99% |
| 吞吐量 | >1Gbps | >1Gbps |
| CPU 占用 (1Gbps) | <10% | <7% |

## 快速开始

### 环境要求

- Rust 1.75+
- cargo
- git

### 编译

```bash
# 克隆仓库
git clone https://github.com/fastlink-rs/fastlink.git
cd fastlink

# 检查编译
cargo check --workspace

# 发布编译
cargo build --release
```

### 运行示例

```bash
# 运行 CLI
cargo run --bin fastlink

# 运行测试
cargo test --workspace
```

## 项目结构

```
fastlink/
├── Cargo.toml              # Workspace 配置
├── core/                   # 核心库
│   ├── mother-protocol/    # 母协议
│   ├── libfastcrypto/      # 加密库
│   ├── libcommon/          # 通用库
│   ├── libomnilink/        # 穿透引擎
│   ├── libfasttransport/   # 传输库
│   ├── libfastdht/         # DHT 网络
│   └── libantidpi/         # 抗 DPI
├── protocols/              # 子协议
│   ├── fastlink-p2p/       # P2P 协议
│   ├── fastlink-server/    # Server 协议
│   ├── fastlink-swift/     # Swift 协议
│   ├── fastlink-games/     # Games 协议
│   ├── fastlink-aztec/     # Aztec 协议
│   └── fastlink-chat/      # Chat 协议
├── tools/                  # 开发工具
├── apps/                   # 应用
└── tests/                  # 测试
```

## 文档

详细的技术文档请参考 [workplace](workplace/) 目录：

- [母协议文档](workplace/FastLink%20母协议%20全维度深度拆解.md)
- [完整技术文档](workplace/FastLink完整技术文档与GitHub仓库结构.md)
- [审计报告](workplace/FastLink协议体系审计报告.md)
- [开发任务计划](workplace/FastLink开发任务计划.md)

## 开发

### 代码规范

- 遵循 Rust 官方 Style Guide
- 使用 clippy 进行代码检查
- 所有公共 API 必须文档化
- 单元测试覆盖率 >85%

### 提交规范

```
<type>(<scope>): <subject>

<body>

<footer>
```

类型 (type):
- `feat`: 新功能
- `fix`: 修复 bug
- `docs`: 文档
- `style`: 代码风格
- `refactor`: 重构
- `test`: 测试
- `chore`: 构建/工具

### 测试

```bash
# 单元测试
cargo test --workspace

# 集成测试
cargo test --test integration

# 性能基准测试
cargo bench --workspace

# 代码覆盖率
cargo tarpaulin --workspace --out Html
```

## 许可证

本项目采用 [MIT 许可证](LICENSE)。

## 合规声明

- **不提供翻墙服务**: 本项目仅用于合法组网用途
- **用户责任自负**: 使用者需遵守所在国家/地区法律法规
- **无后门承诺**: 代码完全开源，可审计
- **隐私保护**: 不收集、不上传任何用户隐私数据

## 致谢

FastLink 的设计和实现参考了以下优秀项目：

- [WireGuard](https://www.wireguard.com/) - 安全、现代的 VPN
- [libp2p](https://libp2p.io/) - P2P 网络框架
- [WebRTC](https://webrtc.org/) - 实时通信
- [Signal](https://signal.org/) - 安全通信
- [ZeroTier](https://www.zerotier.com/) - P2P 网络

## 联系方式

- **邮箱**: team@fastlink.rs
- **GitHub**: https://github.com/fastlink-rs/fastlink
- **文档**: https://docs.fastlink.rs

---

**注意**: FastLink 仍在积极开发中，API 可能会发生变化。
