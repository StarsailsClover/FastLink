# FastLink

**High-performance P2P networking protocol suite**  
[中文文档](RELEASE_NOTES_v26.5-20260531_zh.md) | [English Docs](RELEASE_NOTES_v26.5-20260531.md)

---

## 🚀 项目简介

FastLink 是一套高性能 P2P 网络协议套件，提供：

- **母协议**: 统一抽象层、时间同步、NAT参数库
- **P2P 协议**: BirthdayPunch NAT穿透、建连状态机
- **Server 协议**: 五维加权智能路由
- **Swift 隧道**: 抗 DPI 混淆传输
- **Games 协议**: 低延迟游戏联机
- **Aztec 协议**: 企业级分布式 Mesh 组网
- **Chat 协议**: 零服务器端到端加密通讯

---

## 📦 快速开始

```bash
# 克隆仓库
git clone https://github.com/StarsailsClover/FastLink.git
cd FastLink

# 编译项目
cargo build

# 运行测试
cargo test

# 运行 CLI
cargo run --bin fastlink-cli
```

---

## 📁 项目结构

```
FastLink/
├── apps/fastlink-cli/          # CLI 应用
├── core/                        # 核心库
│   ├── mother-protocol/         # 母协议
│   ├── libfastcrypto/           # 加密库
│   ├── libcommon/               # 公共组件
│   ├── libomnilink/             # P2P 全链接
│   ├── libfasttransport/        # 传输层
│   ├── libfastdht/              # DHT
│   ├── libantidpi/              # 反 DPI
│   └── libnetworktest/          # 网络测试
└── protocols/                   # 协议实现
    ├── fastlink-p2p/            # P2P 协议
    ├── fastlink-server/         # Server 协议
    ├── fastlink-swift/          # Swift 隧道
    ├── fastlink-games/          # 游戏联机
    ├── fastlink-aztec/          # 企业 Mesh
    └── fastlink-chat/           # 加密通讯
```

---

## 🔧 核心特性

### BirthdayPunch NAT 穿透
基于生日悖论的确定性端口生成算法，支持中国移动/电信/联通三网自适应。

### 五维加权路由
```text
W = 0.4·延迟 + 0.3·丢包 + 0.1·跳数 + 0.15·运营商 + 0.05·地理
```

### 6 状态建连自动机
```
空闲 → 探测 → 预映射 → 打洞 → 已连通/断开
         ↕ 角色互换
        失败上报
```

---

## 📝 文档

- [母协议深度拆解](workplace/FastLink%20母协议%20全维度深度拆解.md)
- [P2P 篇深度研讨](workplace/FastLink%20七大子协议全维度深度研讨%EF%BC%9AFastLink-P2P%20篇.md)
- [Server 子协议](workplace/第二子协议%EF%BC%9AFastLink-Server%20去中心化中继与智能路由子协议.md)
- [完整技术文档](workplace/FastLink完整技术文档与GitHub仓库结构.md)

---

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

---

## 📄 许可证

MIT OR Apache-2.0

---

## 📬 联系方式

**Email**: [sailshuang@gmail.com](mailto:sailshuang@gmail.com)  
**GitHub**: [@StarsailsClover](https://github.com/StarsailsClover)
