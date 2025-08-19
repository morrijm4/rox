use rox::Rox;

mod args;
mod http;
mod rox;

#[tokio::main]
async fn main() {
    Rox::start().await;
}
