//! Packet capture and analysis tool for FastLink

use clap::Parser;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "packet-capture")]
#[command(about = "Capture and analyze FastLink packets")]
struct Args {
    /// Interface to capture on
    #[arg(short, long)]
    interface: Option<String>,
    
    /// Output file
    #[arg(short, long, default_value = "capture.pcap")]
    output: String,
    
    /// Filter expression
    #[arg(short, long)]
    filter: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    
    let args = Args::parse();
    
    info!("Packet capture tool");
    info!("Output: {}", args.output);
    
    if let Some(interface) = args.interface {
        info!("Interface: {}", interface);
    }
    
    if let Some(filter) = args.filter {
        info!("Filter: {}", filter);
    }
    
    // TODO: Implement packet capture
    info!("Capture started... Press Ctrl+C to stop");
    
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
