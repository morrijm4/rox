use std::env;

use args::Args;
use proxy::Proxy;

mod args;
mod http;
mod proxy;

#[tokio::main]
async fn main() {
    let args = match Args::parse(&mut env::args()) {
        Ok(a) => a,
        Err(e) => {
            eprintln!("{}", e);
            return help();
        }
    };

    if args.help {
        return help();
    }

    if args.version {
        return version();
    }

    Proxy::new(args).run().await
}

fn version() {
    println!("{}", env!("CARGO_PKG_VERSION"))
}

fn help() {
    println!(
        "
USAGE: rox[EXE] [OPTIONS] [-P | --protocol] <PROTOCOL>

OPTIONS:
    -h, --help                  Print help
    -v, --version               Print version
    -p, --port <PORT>           Specify port for proxy server [default: protocol convention]
    -P, --protocol <PROTOCOL>   Specify proxy protocol [default: http]

PROTOCOLS:
    http (default)
"
    )
}
