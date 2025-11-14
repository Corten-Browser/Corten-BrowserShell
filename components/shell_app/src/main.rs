//! Main entry point for CortenBrowser
//!
//! This file provides the standalone executable entry point.

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    shell_app::run(args).await?;
    Ok(())
}
