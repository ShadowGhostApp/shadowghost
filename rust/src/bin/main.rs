use shadowghost::ui::CliInterface;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cli = CliInterface::new();
    cli.run().await?;
    Ok(())
}
