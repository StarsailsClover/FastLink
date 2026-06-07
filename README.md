# FastLink

**High-performance P2P networking protocol suite**

---

## 🚀 Project Introduction

FastLink is a high-performance P2P networking protocol suite providing:

- **Mother Protocol**: Unified abstraction layer, time synchronization, NAT parameter library
- **P2P Protocol**: Enhanced NAT traversal with BirthdayPunch algorithm, connection state machine
- **Server Protocol**: Five-dimensional weighted intelligent routing
- **Swift Tunnel**: Anti-DPI obfuscated transport
- **Games Protocol**: Low-latency game networking
- **Aztec Protocol**: Enterprise-grade distributed Mesh networking
- **Chat Protocol**: Zero-server end-to-end encrypted messaging

---

## 🏃 Quick Start

```bash
# Clone repository
git clone https://github.com/StarsailsClover/FastLink.git
cd FastLink

# Build project
cargo build --release

# Run tests
cargo test

# Run CLI
cargo run --bin fastlink-cli
```

---

## 🏗️ Project Structure

```
FastLink/
├── apps/fastlink-cli/          # CLI application
├── core/                        # Core libraries
│   ├── mother-protocol/         # Mother protocol
│   ├── libfastcrypto/           # Crypto library
│   ├── libcommon/               # Common components
│   ├── libomnilink/             # P2P omnilink (NAT traversal)
│   ├── libfasttransport/        # Transport layer
│   ├── libfastdht/              # DHT
│   ├── libantidpi/              # Anti-DPI
│   └── libnetworktest/          # Network testing
├── protocols/                   # Protocol implementations
│   ├── fastlink-p2p/            # P2P protocol
│   ├── fastlink-server/         # Server protocol
│   ├── fastlink-swift/          # Swift tunnel
│   ├── fastlink-games/          # Game networking
│   ├── fastlink-aztec/          # Enterprise Mesh
│   └── fastlink-chat/           # Encrypted messaging
└── tools/                       # Testing and debugging tools
```

---

## 🔥 Core Features

### 🎂 BirthdayPunch NAT Traversal (NEW)

**Multi-strategy NAT traversal with intelligent fallback:**

| Strategy | NAT Type | Success Rate | Description |
|----------|----------|--------------|-------------|
| Direct | Open | 100% | Direct connection |
| STUN | Full/Restricted Cone | >90% | Standard hole punching |
| **BirthdayPunch** | **Symmetric** | **~60%** | **Port prediction via birthday paradox** |
| TURN | All types | 99% | Relay fallback |

**Key improvements:**
- ✅ **BirthdayPunch Algorithm**: Based on birthday paradox, predicts NAT port allocation
- ✅ **Adaptive Strategy Selection**: Automatically chooses best method based on detected NAT type
- ✅ **Symmetric NAT Support**: Improved from <50% to ~60% success rate
- ✅ **TURN Fallback**: Guaranteed connectivity when direct methods fail

### How BirthdayPunch Works

```
1. Detect NAT type via STUN servers
2. If Symmetric NAT detected:
   - Analyze port allocation pattern (sequential/random/delta)
   - Generate predicted port candidates using birthday paradox
   - Attempt connection to predicted ports
3. If all methods fail, fall back to TURN relay
```

### 📊 NAT Traversal Success Rates

| NAT Type | Best Strategy | Expected Success |
|----------|---------------|------------------|
| Open Internet | Direct | 100% |
| Full Cone | STUN | >95% |
| Restricted Cone | STUN | >90% |
| Port Restricted | STUN/BirthdayPunch | >85% |
| **Symmetric** | **BirthdayPunch** | **~60%** |

*Note: Actual success depends on ISP and router configuration*

### Five-Dimensional Weighted Routing

```
W = 0.4·Latency + 0.3·Loss + 0.1·Hops + 0.15·ISP + 0.05·Geo
```

### 6-State Connection State Machine

```
Idle → Detect → PreMap → Punch → Connected/Disconnected
         ↓        ↓        ↓
      Role Swap  Failure Report
```

---

## 🛠️ Technical Stack

| Component | Technology | Purpose |
|-----------|------------|---------|
| Core | Rust + Tokio | Async runtime |
| Crypto | x25519-dalek, ed25519-dalek | Key exchange & signatures |
| Encryption | ChaCha20-Poly1305 | AEAD symmetric encryption |
| Hash | Blake3 | Fast cryptographic hashing |
| NAT | STUN/TURN + BirthdayPunch | NAT traversal |
| DHT | Kademlia variant | Peer discovery |

---

## 📦 Installation

### Requirements
- Rust 1.75+
- Cargo

### Build
```bash
git clone https://github.com/StarsailsClover/FastLink.git
cd FastLink
cargo build --release
```

### Run
```bash
# CLI help
cargo run --bin fastlink-cli -- --help

# Start P2P node
cargo run --bin fastlink-cli -- node start

# Run tests
cargo test --workspace
```

---

## 🤝 Contributing

Issues and Pull Requests are welcome!

---

## 📜 License

MIT OR Apache-2.0

---

## 📧 Contact

**Email**: [sailshuang@gmail.com](mailto:sailshuang@gmail.com)  
**GitHub**: [@StarsailsClover](https://github.com/StarsailsClover)

---

## 🙏 Acknowledgments

- BirthdayPunch algorithm inspired by academic research on NAT traversal
- STUN/TURN protocols from RFC 5389/5766
- ICE framework from RFC 5245
