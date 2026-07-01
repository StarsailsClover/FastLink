# FastLink 使用指南

**版本**: v26.14-20260608  
**日期**: 2026-06-08

---

## 🚀 启动 FastLink Swift 节点

### 方式一：使用 CLI 命令

```bash
# 启动 Swift 服务器节点（监听模式）
cargo run --bin fastlink-cli -- server \
  --bind 0.0.0.0:8443 \
  --protocol swift \
  --antidpi true

# 启动 Swift 客户端并连接到服务器
cargo run --bin fastlink-cli -- connect \
  --address example.com \
  --port 8443 \
  --protocol swift \
  --antidpi true \
  --fingerprint chrome124 \
  --congestion bbr
```

### 方式二：编程方式（Rust）

```rust
use fastlink_swift::{SwiftClient, SwiftConfig, TransportMode, CongestionAlgorithm};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 配置 Swift 节点
    let config = SwiftConfig {
        local_addr: Some("0.0.0.0:8080".parse()?),
        transport_mode: TransportMode::Reliable,
        max_concurrent_streams: 100,
        initial_window_size: 65535,
        idle_timeout_ms: 30000,
        keep_alive_interval_ms: 5000,
        antidpi_enabled: true,
        fingerprint: Some("chrome124".to_string()),
        isp_type: Some(IspType::CMCC), // 中国移动
        multipath_enabled: true,
        max_paths: 4,
        qos_evasion_enabled: true,
        packet_size_min: 1200,
        packet_size_max: 1400,
        jitter_ms: 5,
        congestion_algorithm: CongestionAlgorithm::Bbr,
    };

    // 创建并启动 Swift 节点
    let mut client = SwiftClient::new(config);
    let server_addr: SocketAddr = "192.168.1.100:8443".parse()?;
    
    // 连接到服务器
    let connection = client.connect(server_addr).await?;
    println!("✅ Swift 节点已连接到 {}", server_addr);

    Ok(())
}
```

---

## 🔗 使用 FastLink P2P 连接到特定地址

### IPv4 地址连接

```bash
# 连接到 IPv4 地址
cargo run --bin fastlink-cli -- p2p connect \
  --address 192.168.1.100 \
  --port 8080 \
  --node-id "peer-12345"

# 或者使用完整命令格式
cargo run --bin fastlink-cli -- p2p connect \
  --target 192.168.1.100:8080
```

### IPv6 地址连接

```bash
# 连接到 IPv6 地址（需用方括号包裹）
cargo run --bin fastlink-cli -- p2p connect \
  --address "[2001:db8::1]" \
  --port 8080

# 或者
cargo run --bin fastlink-cli -- p2p connect \
  --target "[2001:db8::1]:8080"
```

### 编程方式（P2P 节点）

```rust
use fastlink_p2p::{P2PNode, NodeConfig, NodeId};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 配置 P2P 节点
    let config = NodeConfig {
        node_id: NodeId::generate(),
        listen_addr: "0.0.0.0:0".parse()?, // 监听任意端口
        bootstrap_nodes: vec![],
        enable_nat_traversal: true,
        enable_dht: true,
        message_queue_size: 1000,
    };

    // 创建 P2P 节点
    let node = P2PNode::new(config).await?;
    
    // 连接到特定 IPv4 地址
    let ipv4_addr: SocketAddr = "192.168.1.100:8080".parse()?;
    node.connect(&ipv4_addr).await?;
    println!("✅ 已连接到 IPv4 节点: {}", ipv4_addr);

    // 连接到特定 IPv6 地址
    let ipv6_addr: SocketAddr = "[2001:db8::1]:8080".parse()?;
    node.connect(&ipv6_addr).await?;
    println!("✅ 已连接到 IPv6 节点: {}", ipv6_addr);

    // 发送消息
    let target_id = NodeId::from_hex("peer-12345")?;
    node.send(target_id, b"Hello P2P!".to_vec()).await?;

    Ok(())
}
```

---

## 🛡️ Hiddify 配置指南

### FastLink 支持的协议

在 Hiddify 中可以配置以下 FastLink 协议：

| 协议 | 说明 | 推荐场景 |
|------|------|----------|
| `fastlink` | 标准 FastLink 协议 | 一般 P2P 连接 |
| `fastlink-swift` | Swift 高速传输协议 | 高吞吐、抗 DPI |
| `fls` | FastLink Swift 缩写 | 同上 |

### 配置 URL 格式

```
fastlink://<uuid>@<server>:<port>?<参数>

fastlink-swift://<uuid>@<server>:<port>?<参数>
```

### 完整配置示例

#### 1. FastLink Swift 配置（推荐）

```
fastlink-swift://550e8400-e29b-41d4-a716-446655440000@example.com:443?\
fp=chrome124&\
isp=cmcc&\
congestion=bbr&\
antidpi=true&\
multipath=true&\
max_paths=4&\
qos=true&\
packet_min=1200&\
packet_max=1400&\
jitter=5&\
mode=reliable&\
#hiddify=true
```

**参数说明**:
- `fp`: 浏览器指纹 (chrome124, firefox125, safari17, edge124)
- `isp`: ISP 类型 (cmcc, ctc, cnc, unicom, telecom, mobile, other)
- `congestion`: 拥塞控制算法 (bbr, cubic, reno)
- `antidpi`: 启用反 DPI (true/false)
- `multipath`: 多路径传输 (true/false)
- `max_paths`: 最大路径数 (1-8)
- `qos`: QoS 规避 (true/false)
- `packet_min`: 最小包大小 (1000-1400)
- `packet_max`: 最大包大小 (1200-1500)
- `jitter`: 抖动时间毫秒 (0-20)
- `mode`: 传输模式 (reliable, unreliable, partially_reliable)
- `#hiddify`: Hiddify 专用标记

#### 2. 中国移动优化配置

```
fastlink-swift://550e8400-e29b-41d4-a716-446655440000@hk.example.com:443?\
fp=chrome124&\
isp=cmcc&\
congestion=bbr&\
antidpi=true&\
multipath=true&\
max_paths=4&\
qos=true&\
packet_min=1200&\
packet_max=1400&\
jitter=5&\
mode=reliable&\
#hiddify=true
```

#### 3. 中国电信优化配置

```
fastlink-swift://550e8400-e29b-41d4-a716-446655440000@jp.example.com:443?\
fp=edge124&\
isp=ctc&\
congestion=bbr&\
antidpi=true&\
multipath=true&\
max_paths=3&\
qos=true&\
packet_min=1200&\
packet_max=1400&\
jitter=3&\
mode=reliable&\
#hiddify=true
```

### Hiddify App 配置步骤

1. **打开 Hiddify App**
   - 点击右上角 "+" 添加配置

2. **选择导入方式**
   - 选择 "从剪贴板导入" 或 "手动输入"

3. **粘贴配置 URL**
   ```
   fastlink-swift://...#hiddify=true
   ```

4. **保存并连接**
   - 配置文件会自动解析
   - 点击连接按钮测试

### 高级配置（带 TLS/REALITY）

```
fastlink-swift://550e8400-e29b-41d4-a716-446655440000@example.com:443?\
security=tls&\
sni=www.example.com&\
fp=chrome124&\
isp=cmcc&\
congestion=bbr&\
antidpi=true&\
multipath=true&\
max_paths=4&\
qos=true&\
packet_min=1200&\
packet_max=1400&\
jitter=5&\
mode=reliable&\
#hiddify=true
```

---

## 📊 推荐配置组合

### 场景一：家庭宽带（电信/联通）
```
fastlink-swift://...?fp=chrome124&isp=ctc&congestion=bbr&\
antidpi=true&multipath=true&max_paths=4&qos=true
```

### 场景二：中国移动 4G/5G
```
fastlink-swift://...?fp=chrome124&isp=cmcc&congestion=bbr&\
antidpi=true&multipath=true&max_paths=4&qos=true&\
packet_min=1200&packet_max=1400&jitter=5
```

### 场景三：高延迟国际线路
```
fastlink-swift://...?fp=firefox125&congestion=bbr&\
antidpi=true&multipath=true&max_paths=2&\
mode=reliable&jitter=10
```

### 场景四：游戏低延迟模式
```
fastlink-swift://...?fp=chrome124&congestion=bbr&\
antidpi=false&multipath=false&mode=reliable&jitter=0
```

---

## 🔧 故障排除

### 连接失败
```bash
# 诊断网络问题
cargo run --bin fastlink-cli -- diagnose \
  --target example.com:443 \
  --protocol swift

# 测试 NAT 类型
cargo run --bin fastlink-cli -- diagnose --nat-test
```

### 性能调优
```bash
# 运行基准测试
cargo run --bin fastlink-cli -- benchmark \
  --target example.com:443 \
  --duration 60
```

---

## 📞 更多帮助

- **GitHub**: https://github.com/StarsailsClover/FastLink
- **邮箱**: sailshuang@gmail.com
