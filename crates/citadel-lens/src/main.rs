//! Lens Node binary
//!
//! A Citadel mesh node for distributed content distribution.

use citadel_lens::{LensConfig, LensNode};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "lens_node=info,citadel=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Lens Node");

    // Load config (TODO: from args/file)
    let config = LensConfig::default();

    // Create and run node
    let node = LensNode::new(config).await?;
    node.run().await?;

    Ok(())
}
