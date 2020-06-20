mod cli;
mod client;

use clap::Clap;

const BASE_URL: &str = "https://xkcd.com";

fn main() -> anyhow::Result<()> {
    let args = cli::Args::parse();
    let client = client::XkcdClient::new(args);
    client.run()
}
