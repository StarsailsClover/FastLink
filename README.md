# FastLink

**High-performance P2P networking protocol suite**  
[中文文档](RELEASE_NOTES_v26.5-20260531_zh.md) | [English Docs](RELEASE_NOTES_v26.5-20260531.md)

---

## 🚀 Project Introduction

FastLink is a high-performance P2P networking protocol suite providing:

- **Mother Protocol**: Unified abstraction layer, time synchronization, NAT parameter library
- **P2P Protocol**: BirthdayPunch NAT traversal, connection state machine
- **Server Protocol**: Five-dimensional weighted intelligent routing
- **Swift Tunnel**: Anti-DPI obfuscated transport
- **Games Protocol**: Low-latency game networking
- **Aztec Protocol**: Enterprise-grade distributed Mesh networking
- **Chat Protocol**: Zero-server end-to-end encrypted messaging

---

## 📦 Quick Start

```bash
# Clone repository
git clone https://github.com/StarsailsClover/FastLink.git
cd FastLink

# Build project
cargo build

# Run tests
cargo test

# Run CLI
cargo run --bin fastlink-cli
```

---

## 📁 Project Structure

```
FastLink/
├── apps/fastlink-cli/          # CLI application
├── core/                        # Core libraries
│   ├── mother-protocol/         # Mother protocol
│   ├── libfastcrypto/           # Crypto library
│   ├── libcommon/               # Common components
│   ├── libomnilink/             # P2P omnilink
│   ├── libfasttransport/        # Transport layer
│   ├── libfastdht/              # DHT
│   ├── libantidpi/              # Anti-DPI
│   └── libnetworktest/          # Network testing
└── protocols/                   # Protocol implementations
    ├── fastlink-p2p/            # P2P protocol
    ├── fastlink-server/         # Server protocol
    ├── fastlink-swift/          # Swift tunnel
    ├── fastlink-games/          # Game networking
    ├── fastlink-aztec/          # Enterprise Mesh
    └── fastlink-chat/           # Encrypted messaging
```

---

## 🔧 Core Features

### BirthdayPunch NAT Traversal
Deterministic port generation algorithm based on the birthday paradox, with adaptive support for China Mobile/Telecom/Unicom ISPs.

### Five-Dimensional Weighted Routing
```
W = 0.4·Latency + 0.3·Loss + 0.1·Hops + 0.15·ISP + 0.05·Geo
```

### 6-State Connection State Machine
```
Idle → Detect → PreMap → Punch → Connected/Disconnected
         ↕ Role Swap
        Failure Report
```

---

## 📝 Documentation

- [Mother Protocol Deep Dive](workplace/FastLink%20母协议%20全维度深度拆解.md)
- [P2P Deep Discussion](workplace/FastLink%20七大子协议全维度深度研讨%EF%BC%9AFastLink-P2P%20篇.md)
- [Server Sub-protocol](workplace/第二子协议%EF%BC%9AFastLink-Server%20去中心化中继与智能路由子协议.md)
- [Complete Technical Docs](workplace/FastLink完整技术文档与GitHub仓库结构.md)

---

## 🤝 Contributing

Issues and Pull Requests are welcome!

---

## 📄 License

MIT OR Apache-2.0

---

## 📬 Contact

**Email**: [sailshuang@gmail.com](mailto:sailshuang@gmail.com)  
**GitHub**: [@StarsailsClover](https://github.com/StarsailsClover)
