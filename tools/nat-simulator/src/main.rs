//! NAT Simulator for testing FastLink P2P
//! 
//! Simulates various NAT behaviors:
//! - Full Cone NAT
//! - Restricted Cone NAT
//! - Port Restricted Cone NAT
//! - Symmetric NAT

use clap::Parser;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "nat-sim")]
#[command(about = "NAT behavior simulator for FastLink testing")]
struct Args {
    /// NAT type to simulate
    #[arg(short, long, default_value = "symmetric")]
    nat_type: String,
    
    /// External IP address
    #[arg(short, long, default_value = "127.0.0.1")]
    external_ip: String,
    
    /// Port range start
    #[arg(long, default_value_t = 10000)]
    port_start: u16,
    
    /// Port range end
    #[arg(long, default_value_t = 60000)]
    port_end: u16,
    
    /// Mapping timeout in seconds
    #[arg(long, default_value_t = 300)]
    mapping_timeout: u64,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    info!("Starting NAT simulator");
    info!("NAT type: {}", args.nat_type);
    info!("External IP: {}", args.external_ip);
    info!("Port range: {}-{}", args.port_start, args.port_end);
    info!("Mapping timeout: {}s", args.mapping_timeout);
    
    // TODO: Implement NAT simulation logic
    info!("NAT simulator running... Press Ctrl+C to stop");
    
    // Keep running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
