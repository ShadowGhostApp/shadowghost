use shadowghost::cli::CliInterface;
use shadowghost::core::ShadowGhostCore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let core = ShadowGhostCore::new()?;
    let mut cli = CliInterface::new(core);
    cli.run().await?;
    Ok(())
}
