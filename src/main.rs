use cli::Cli;

mod args;
mod cli;
mod http;

#[tokio::main]
async fn main() {
    Cli::start().await;
}
