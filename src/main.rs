use clap::Parser;

use reverse_proxy::start_reverse_proxy;

mod reverse_proxy;
mod github_api;


/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target URL of reverse proxy
    #[arg(short, long)]
    target: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    start_reverse_proxy(args.target).await
}
