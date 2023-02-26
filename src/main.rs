use clap::Parser;

use reverse_proxy::start_reverse_proxy;

use crate::github_api::get_org_members;

mod reverse_proxy;
mod github_api;


/// Reverse proxy that uses GitHub credentials to validate that a user belongs to an org
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Address to bind to
    #[arg(env, long, default_value = "127.0.0.1:8000")]
    bind: String,

    /// Auth method to to use (currently only supported is GH_BASIC)
    #[arg(env, long)]
    auth_method: String,

    /// Target URL of reverse proxy
    #[arg(env, long)]
    target: String,

    /// Token to use to read organisation members
    #[arg(env, long, default_value = "")]
    gh_org_token: String,

    /// Organisation to check membership of
    #[arg(env, long)]
    gh_org: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    if args.auth_method == "GH_BASIC" {
        get_org_members(args.gh_org, args.gh_org_token).await;

        start_reverse_proxy(args.target, args.bind).await
    } else {
        eprintln!("Auth method must be set to GH_BASIC");
        std::process::exit(1);
    }
}
