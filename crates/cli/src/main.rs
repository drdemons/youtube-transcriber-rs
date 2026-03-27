use clap::Parser;
use yt_transcriber_cli::cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    yt_transcriber_cli::run(cli).await
}
