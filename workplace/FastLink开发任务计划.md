# FastLink 协议体系开发任务计划

**版本**: v1.0
**制定时间**: 2026-05-30
**预计周期**: 6-12 个月

---

## 一、总体开发策略

### 1.1 开发原则

1. **模块化开发**: 遵循文档中的分层架构，逐步实现每个模块
2. **测试驱动**: 每个模块必须有完整的单元测试和集成测试
3. **文档同步**: 代码实现与文档保持同步更新
4. **性能优先**: 性能关键部分先实现并基准测试
5. **安全第一**: 加密和安全模块必须经过严格审计

### 1.2 开发阶段划分

| 阶段 | 时间 | 目标 | 里程碑 |
|------|------|------|--------|
| Phase 0 | 1-2 周 | 项目初始化 | Workspace 结构搭建完成 |
| Phase 1 | 4-8 周 | 核心基础设施 | 母协议 + 核心库完成 |
| Phase 2 | 6-10 周 | 传输层实现 | P2P 直连完成 |
| Phase 3 | 8-12 周 | 基础子协议 | P2P + Server + Swift 完成 |
| Phase 4 | 8-12 周 | 高级子协议 | Games + Aztec + Chat 完成 |
| Phase 5 | 4-8 周 | 测试与优化 | 完整测试体系 + 性能优化 |

---

## 二、详细任务分解

### Phase 0: 项目初始化（第 1-2 周）

#### 任务 0.1: 创建 Workspace 结构

```
fastlink/
├── Cargo.toml                    # Workspace 配置
├── README.md                      # 项目说明
├── LICENSE                       # MIT 许可证
├── src/
│   └── lib.rs                    # 根库入口
├── core/                         # 母协议核心
│   ├── mother-protocol/          # 母协议抽象层
│   ├── libfastcrypto/            # 加密库
│   ├── libcommon/                # 通用工具库
│   ├── libomnilink/              # 穿透引擎
│   ├── libfasttransport/         # 传输库
│   ├── libfastdht/               # DHT 网络
│   └── libantidpi/               # 抗 DPI
├── protocols/                    # 子协议
│   ├── fastlink-p2p/             # P2P 子协议
│   ├── fastlink-server/          # Server 子协议
│   ├── fastlink-swift/           # Swift 子协议
│   ├── fastlink-games/           # Games 子协议
│   ├── fastlink-aztec/           # Aztec 子协议
│   └── fastlink-chat/            # Chat 子协议
├── tools/                        # 开发工具（已存在）
├── tests/                        # 集成测试
├── docs/                         # 文档
└── .github/
    └── workflows/               # CI/CD 配置
```

**交付物**:
- [ ] Workspace Cargo.toml 配置
- [ ] 所有 crate 的 Cargo.toml 模板
- [ ] 项目 README.md
- [ ] 基础目录结构

#### 任务 0.2: CI/CD 流水线搭建

**任务清单**:
- [ ] GitHub Actions 工作流配置
- [ ] 自动构建流程
- [ ] 单元测试自动化
- [ ] 代码风格检查（clippy）
- [ ] 安全检查配置
- [ ] 文档自动生成

---

### Phase 1: 核心基础设施（第 3-6 周）

#### 任务 1.1: libcommon 通用工具库

**优先级**: 🔴 高

**模块划分**:

1. **错误处理模块** (`error.rs`)
   ```rust
   // 定义全局错误码（继承母协议规范）
   pub const ERR_OK: u32 = 0x00000000;
   pub const ERR_VERSION: u32 = 0x00000001;
   pub const ERR_HANDSHAKE: u32 = 0x00000002;
   // ... 完整的错误码定义
   ```

2. **日志模块** (`logging.rs`)
   ```rust
   // 基于 tracing 的结构化日志
   pub fn init_logging(level: LogLevel);
   pub fn set_log_format(format: LogFormat);
   ```

3. **配置管理模块** (`config.rs`)
   ```rust
   pub struct Config {
       pub mtu: u16,
       pub heartbeat_interval: Duration,
       pub idle_timeout: Duration,
       // ... 所有参数默认值
   }
   ```

4. **时间同步模块** (`time.rs`)
   ```rust
   pub async fn sync_time() -> Result<TimeOffset>;
   pub fn get_local_time() -> Timestamp;
   pub fn calculate_offset(remote: Timestamp) -> TimeOffset;
   ```

5. **序列化模块** (`serialization.rs`)
   ```rust
   pub fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>>;
   pub fn deserialize<T: Deserialize>(data: &[u8]) -> Result<T>;
   ```

6. **平台适配模块** (`platform.rs`)
   ```rust
   #[cfg(target_os = "windows")]
   mod windows;
   #[cfg(target_os = "linux")]
   mod linux;
   #[cfg(target_os = "macos")]
   mod macos;
   ```

**交付物**:
- [ ] 完整的 libcommon 库实现
- [ ] 所有模块的单元测试
- [ ] API 文档生成

#### 任务 1.2: libfastcrypto 加密库

**优先级**: 🔴 高

**模块划分**:

1. **数字签名模块** (`signature.rs`)
   ```rust
   // Ed25519 实现
   pub struct Ed25519;
   impl SignatureScheme for Ed25519 {
       fn sign(&self, private_key: &[u8], message: &[u8]) -> Vec<u8>;
       fn verify(&self, public_key: &[u8], message: &[u8], signature: &[u8]) -> bool;
   }
   ```

2. **密钥交换模块** (`key_exchange.rs`)
   ```rust
   // X25519 实现
   pub struct X25519;
   impl KeyExchange for X25519 {
       fn generate_keypair() -> (Vec<u8>, Vec<u8>);
       fn derive_shared_secret(private_key: &[u8], public_key: &[u8]) -> Vec<u8>;
   }
   ```

3. **对称加密模块** (`symmetric.rs`)
   ```rust
   // AES-256-GCM 实现
   pub struct Aes256Gcm;
   impl SymmetricCipher for Aes256Gcm {
       fn encrypt(&self, key: &[u8], nonce: &[u8], plaintext: &[u8]) -> Vec<u8>;
       fn decrypt(&self, key: &[u8], nonce: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>>;
   }
   ```

4. **哈希模块** (`hash.rs`)
   ```rust
   // BLAKE3 实现
   pub struct Blake3;
   impl HashFunction for Blake3 {
       fn hash(data: &[u8]) -> Vec<u8>;
       fn verify(data: &[u8], hash: &[u8]) -> bool;
   }
   ```

5. **密钥派生模块** (`kdf.rs`)
   ```rust
   // HKDF-SHA256 实现
   pub struct HkdfSha256;
   impl KeyDerivation for HkdfSha256 {
       fn derive(key: &[u8], salt: &[u8], info: &[u8], length: usize) -> Vec<u8>;
   }
   ```

6. **防重放模块** (`replay_protection.rs`)
   ```rust
   pub struct ReplayProtection {
       window_size: u64,
       seen_packets: HashMap<u64, bool>,
   }
   impl ReplayProtection {
       pub fn check_and_mark(&mut self, sequence: u64) -> bool;
   }
   ```

7. **统一密钥管理模块** (`key_manager.rs`)
   ```rust
   pub struct KeyManager {
       identity_keypair: KeyPair,
       session_keys: HashMap<SessionId, SessionKey>,
   }
   impl KeyManager {
       pub fn new() -> Self;
       pub fn get_session_key(&mut self, peer: &PeerId) -> &SessionKey;
       pub fn rotate_session_key(&mut self, peer: &PeerId);
   }
   ```

**交付物**:
- [ ] 完整的 libfastcrypto 库实现
- [ ] 所有加密算法的单元测试
- [ ] 安全审计文档

#### 任务 1.3: 母协议核心抽象层

**优先级**: 🔴 高

**模块划分**:

1. **报文格式定义** (`message.rs`)
   ```rust
   #[repr(C, packed)]
   pub struct MessageHeader {
       magic: u32,          // 0x464C4B48 "FLKH"
       version: u4,
       msg_type: u4,
       flags: u8,
       length: u16,
       session_id: u64,
       sequence: u32,
       checksum: u32,
   }
   
   pub enum MessageType {
       Handshake = 0x0,
       Data = 0x1,
       Heartbeat = 0x2,
       Error = 0x3,
       Control = 0x4,
   }
   ```

2. **统一接口定义** (`traits.rs`)
   ```rust
   pub trait Connection: Send + Sync {
       async fn connect(&mut self, addr: SocketAddr) -> Result<()>;
       async fn close(&mut self) -> Result<()>;
       async fn send(&mut self, data: &[u8]) -> Result<usize>;
       async fn recv(&mut self, buf: &mut [u8]) -> Result<usize>;
       fn state(&self) -> ConnectionState;
   }
   
   pub trait Listener: Send + Sync {
       async fn bind(addr: SocketAddr) -> Result<Self>
       where Self: Sized;
       async fn accept(&mut self) -> Result<Box<dyn Connection>>;
       fn local_addr(&self) -> SocketAddr;
   }
   
   pub trait Node: Send + Sync {
       fn node_id(&self) -> NodeId;
       fn public_key(&self) -> &PublicKey;
       async fn connect_to(&mut self, node_id: &NodeId) -> Result<Box<dyn Connection>>;
   }
   ```

3. **状态机实现** (`state_machine.rs`)
   ```rust
   pub enum ConnectionState {
       Closed,
       Connecting,
       Handshake,
       Established,
       Closing,
   }
   
   pub struct ConnectionStateMachine {
       state: ConnectionState,
       last_event: Instant,
   }
   impl ConnectionStateMachine {
       pub fn transition(&mut self, event: StateEvent) -> Result<()>;
       pub fn current_state(&self) -> ConnectionState;
   }
   ```

4. **握手流程实现** (`handshake.rs`)
   ```rust
   pub struct Handshake流程 {
       // 四步握手
       // Step 1: ClientHello
       // Step 2: ServerHello
       // Step 3: Finished (Client)
       // Step 4: Finished (Server)
   }
   ```

5. **统一调度器** (`scheduler.rs`)
   ```rust
   pub struct Scheduler {
       connection_pool: ConnectionPool,
       path_selector: PathSelector,
       failover_manager: FailoverManager,
   }
   impl Scheduler {
       pub async fn connect(&mut self, node_id: &NodeId) -> Result<Box<dyn Connection>>;
       pub fn get_best_path(&self, requirements: &PathRequirements) -> Path;
       pub fn switch_path(&mut self, conn: &mut dyn Connection, new_path: Path);
   }
   ```

**交付物**:
- [ ] 完整的母协议核心实现
- [ ] 所有接口的 Mock 实现（用于测试）
- [ ] 状态机单元测试

---

### Phase 2: 传输层实现（第 7-10 周）

#### 任务 2.1: libfasttransport 传输库

**优先级**: 🟡 中

**模块划分**:

1. **可靠 UDP 模块** (`reliable_udp.rs`)
   ```rust
   pub struct ReliableUdp {
       rto: Duration,
       send_window: SlidingWindow,
       recv_window: SlidingWindow,
       retransmit_queue: Vec<Packet>,
   }
   impl ReliableUdp {
       pub async fn send(&mut self, data: &[u8]) -> Result<()>;
       pub async fn recv(&mut self, buf: &mut [u8]) -> Result<usize>;
       pub fn handle_ack(&mut self, ack: Ack);
   }
   ```

2. **多路复用模块** (`multiplexing.rs`)
   ```rust
   pub struct Multiplexer {
       // 基于 Yamux 协议
       stream_id_counter: AtomicU64,
       streams: HashMap<StreamId, Stream>,
   }
   impl Multiplexer {
       pub fn open_stream(&mut self) -> StreamId;
       pub fn close_stream(&mut self, id: StreamId);
       pub fn write_to_stream(&mut self, id: StreamId, data: &[u8]) -> Result<()>;
   }
   ```

3. **拥塞控制模块** (`congestion.rs`)
   ```rust
   pub struct BbrCongestionControl {
       // BBRv2 中国版实现
       min_rtt: Duration,
       bandwidth: f64,
       pacing_rate: f64,
   }
   impl CongestionControl for BbrCongestionControl {
       fn on_ack(&mut self, ack: Ack);
       fn on_timeout(&mut self);
       fn get_pacing_rate(&self) -> f64;
   }
   ```

4. **流量控制模块** (`flow_control.rs`)
   ```rust
   pub struct FlowController {
       send_window: u64,
       recv_window: u64,
       token_bucket: TokenBucket,
   }
   ```

**交付物**:
- [ ] 完整的传输库实现
- [ ] 性能基准测试
- [ ] 与母协议的集成测试

#### 任务 2.2: libomnilink 穿透引擎

**优先级**: 🟡 中

**模块划分**:

1. **NAT 检测模块** (`nat_detection.rs`)
   ```rust
   pub struct NatDetector {
       stun_servers: Vec<SocketAddr>,
   }
   impl NatDetector {
       pub async fn detect(&self) -> Result<NatType>;
       // NAT1, NAT2, NAT3, NAT4
   }
   ```

2. **端口预测模块** (`port_prediction.rs`)
   ```rust
   pub struct PortPredictor {
       history: Vec<u16>,
       pattern: PortPattern,
   }
   impl PortPredictor {
       pub fn predict_next(&self, count: usize) -> Vec<u16>;
       pub fn analyze_pattern(history: &[u16]) -> PortPattern;
   }
   ```

3. **BirthdayPunch 模块** (`birthday_punch.rs`)
   ```rust
   pub struct BirthdayPunch {
       predictor: PortPredictor,
       socket: UdpSocket,
   }
   impl BirthdayPunch {
       pub async fn punch(&mut self, peer_info: &PeerInfo) -> Result<()>;
   }
   ```

4. **智能调度器** (`smart_scheduler.rs`)
   ```rust
   pub struct PunchScheduler {
       methods: Vec<Box<dyn PunchMethod>>,
       priority_list: Vec<PunchPriority>,
   }
   impl PunchScheduler {
       pub async fn try_all(&mut self, peer: &PeerId) -> Result<Connection>;
   }
   ```

**交付物**:
- [ ] 完整的穿透引擎实现
- [ ] NAT 穿透集成测试
- [ ] 跨 NAT 类型测试套件

#### 任务 2.3: libfastdht DHT 网络

**优先级**: 🟡 中

**模块划分**:

1. **Kademlia 核心** (`kademlia.rs`)
   ```rust
   pub struct KademliaDht {
       own_id: NodeId,
       buckets: Vec<KBucket>,
       storage: HashMap<Key, Value>,
   }
   impl KademliaDht {
       pub async fn find_node(&self, target: &NodeId) -> Result<Vec<NodeInfo>>;
       pub async fn find_value(&self, key: &Key) -> Result<Option<Value>>;
       pub async fn store(&mut self, key: &Key, value: &Value);
   }
   ```

2. **中国网络优化** (`china_optimization.rs`)
   ```rust
   pub struct ChinaDhtOptimizer {
       // 同运营商优先
       // 同地区优先
       // 内置引导节点
   }
   ```

**交付物**:
- [ ] 完整的 DHT 实现
- [ ] 节点发现集成测试

---

### Phase 3: 基础子协议（第 11-14 周）

#### 任务 3.1: FastLink-P2P 子协议

**优先级**: 🟡 中

**实现要点**:

1. 继承母协议核心抽象
2. 实现 BirthdayPunch 算法
3. 实现端口预测算法
4. 实现状态机管理
5. 实现重连机制

**交付物**:
- [ ] P2P 子协议完整实现
- [ ] NAT 穿透测试
- [ ] P2P 直连性能测试

#### 任务 3.2: FastLink-Server 子协议

**优先级**: 🟡 中

**实现要点**:

1. 实现节点注册和宣告
2. 实现五维路由算法
3. 实现中继转发
4. 实现信誉系统
5. 实现容灾切换

**交付物**:
- [ ] Server 子协议完整实现
- [ ] 中继功能测试
- [ ] 路由选择测试

#### 任务 3.3: FastLink-Swift 子协议

**优先级**: 🟡 中

**实现要点**:

1. 实现 TLS 指纹模拟
2. 实现流量伪装
3. 实现多路径聚合
4. 实现 QoS 规避
5. 实现会话管理

**交付物**:
- [ ] Swift 子协议完整实现
- [ ] DPI 穿透测试
- [ ] 多路径聚合测试

---

### Phase 4: 高级子协议（第 15-18 周）

#### 任务 4.1: FastLink-Games 游戏子协议

**优先级**: 🟢 低

**实现要点**:

1. 实现房间管理
2. 实现帧同步
3. 实现可靠 UDP 优化
4. 实现 FEC 前向纠错
5. 实现 QoS 优化

**交付物**:
- [ ] Games 子协议完整实现
- [ ] 游戏联机测试
- [ ] Unity/Unreal SDK

#### 任务 4.2: FastLink-Aztec 企业组网子协议

**优先级**: 🟢 低

**实现要点**:

1. 实现 Mesh 组网
2. 实现 OLSR 路由
3. 实现 RBAC 权限控制
4. 实现 VLAN 隔离
5. 实现审计日志

**交付物**:
- [ ] Aztec 子协议完整实现
- [ ] 企业组网测试
- [ ] Web 管理界面

#### 任务 4.3: FastLink-Chat 即时通讯子协议

**优先级**: 🟢 低

**实现要点**:

1. 实现身份管理
2. 实现消息传输
3. 实现离线中继
4. 实现双棘轮加密
5. 实现群聊功能

**交付物**:
- [ ] Chat 子协议完整实现
- [ ] 即时通讯测试
- [ ] 多平台 SDK

---

### Phase 5: 测试与优化（第 19-22 周）

#### 任务 5.1: 完整测试体系

1. **单元测试**: 每个模块 >85% 覆盖率
2. **集成测试**: 完整的端到端测试
3. **性能测试**: 基准测试和性能优化
4. **安全测试**: 渗透测试和代码审计
5. **兼容测试**: 跨平台、跨网络环境测试

#### 任务 5.2: 性能优化

1. **零拷贝优化**: 减少内存复制
2. **SIMD 加速**: 加密算法优化
3. **批处理优化**: 减少系统调用
4. **内存池**: 减少内存分配
5. **内核旁路**: XDP/DPDK 支持（可选）

#### 任务 5.3: 文档完善

1. **API 文档**: rustdoc 自动生成
2. **用户手册**: 详细使用说明
3. **部署指南**: 各类场景部署方案
4. **开发者指南**: 贡献代码规范

---

## 三、里程碑与验收标准

### 3.1 里程碑定义

| 里程碑 | 时间 | 交付物 | 验收标准 |
|--------|------|--------|----------|
| M0 | 第 2 周 | 项目结构 | CI/CD 流水线运行 |
| M1 | 第 6 周 | 核心库 | 所有核心库可编译测试 |
| M2 | 第 10 周 | 传输层 | P2P 直连测试通过 |
| M3 | 第 14 周 | 基础协议 | P2P/Server/Swift 功能完成 |
| M4 | 第 18 周 | 完整协议 | 所有子协议功能完成 |
| M5 | 第 22 周 | 产品化 | 测试完整、性能达标 |

### 3.2 验收标准

#### M1: 核心库验收
- [ ] libcommon 所有模块编译通过
- [ ] libfastcrypto 所有加密算法测试通过
- [ ] 母协议核心抽象完成
- [ ] 单元测试覆盖率 >80%

#### M2: 传输层验收
- [ ] 可靠 UDP 传输测试通过
- [ ] NAT 穿透成功率 >90%
- [ ] DHT 节点发现测试通过
- [ ] 性能测试达到基线

#### M3: 基础协议验收
- [ ] P2P 直连延迟 <100ms
- [ ] Server 中继延迟 <150ms
- [ ] Swift DPI 穿透成功率 >95%
- [ ] 链路自动切换测试通过

#### M4: 完整协议验收
- [ ] Games 游戏联机延迟 <30ms
- [ ] Aztec 企业组网 100 节点测试通过
- [ ] Chat 即时通讯消息送达率 >99%
- [ ] 所有子协议单元测试 >85%

#### M5: 产品化验收
- [ ] 性能测试达标
- [ ] 安全审计通过
- [ ] 文档完整
- [ ] 跨平台测试通过

---

## 四、资源估算

### 4.1 人力需求

| 角色 | 人数 | 职责 |
|------|------|------|
| 项目经理 | 1 | 项目管理、进度跟踪 |
| 核心开发者 | 2 | 母协议、核心库开发 |
| 子协议开发者 | 2 | 子协议实现 |
| 测试工程师 | 1 | 测试体系搭建 |
| 安全工程师 | 1 | 安全审计（兼职） |
| DevOps | 1 | CI/CD、部署 |

### 4.2 测试环境需求

1. **NAT 模拟环境**: 模拟各种 NAT 类型
2. **DPI 测试床**: 部署主流 DPI 设备
3. **多节点集群**: 跨地域测试
4. **性能测试环境**: 高性能服务器

---

## 五、风险与应对

### 5.1 技术风险

| 风险 | 影响 | 概率 | 应对措施 |
|------|------|------|----------|
| NAT 穿透不稳定 | 高 | 中 | 完善降级策略，增加测试环境 |
| TLS 指纹过时 | 中 | 高 | 建立自动更新机制 |
| 性能不达标 | 中 | 中 | 提前进行性能测试和优化 |
| 安全漏洞 | 高 | 低 | 定期安全审计，代码审查 |

### 5.2 项目风险

| 风险 | 影响 | 概率 | 应对措施 |
|------|------|------|----------|
| 开发周期延长 | 高 | 高 | 敏捷开发，及时调整范围 |
| 人员流动 | 中 | 中 | 代码审查，知识共享 |
| 需求变更 | 中 | 中 | 需求管理，优先级排序 |

---

## 六、附录

### A. 相关文档

- [审计报告](FastLink协议体系审计报告.md)
- [母协议文档](FastLink%20母协议%20全维度深度拆解.md)
- [完整技术文档](FastLink完整技术文档与GitHub仓库结构.md)

### B. 技术选型详情

| 组件 | 技术选型 | 版本 | 说明 |
|------|----------|------|------|
| 语言 | Rust | 1.75+ | 性能和安全 |
| 加密 | ring | latest | Google 维护 |
| 序列化 | bincode | latest | 高效二进制 |
| 日志 | tracing | latest | 结构化日志 |
| 异步 | tokio | latest | 异步运行时 |

### C. 编码规范

- 遵循 Rust 官方 Style Guide
- 使用 clippy 进行代码检查
- 所有公共 API 必须文档化
- 单元测试覆盖率 >85%
- 禁止使用 unsafe 代码（除加密库必需部分）

---

**任务计划完成**
