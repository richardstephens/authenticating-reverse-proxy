use clap::Parser;

use reverse_proxy::start_reverse_proxy;

use crate::github_api::get_org_members;

mod reverse_proxy;
mod github_api;


/// Reverse proxy that uses GitHub credentials to validate that a user belongs to an org
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Target URL of reverse proxy
    #[arg(short, long)]
    target: String,

    /// Token to use to read organisation members
    #[arg(long)]
    gh_org_token: String,

    /// Organisation to check membership of
    #[arg(long)]
    gh_org: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();
    get_org_members(args.gh_org, args.gh_org_token).await;

    start_reverse_proxy(args.target).await
}
