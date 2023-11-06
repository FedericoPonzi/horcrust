use std::net::TcpListener;

use clap::Parser;
use env_logger::Env;
use log::{debug, info};

use horcrust::{
    horcrust_msg_request, msg_share_response, msg_success_response, AdditiveSecretSharing,
    ConnectionHandler, HorcrustMsgRequest, Result, TcpConnectionHandler,
};
use horcrust::{SecretSharing, SharesDatabase};

/// Create shares out of your secret and stores them to distributed services. Allows you
/// to safely recover your secret from the shares on a later moment.
#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    #[arg(short, long, required = true)]
    /// a list of servers to store your secret. Please provide at least 2 servers.
    servers: Vec<String>,
    /// a port to bind to
    #[arg(short, long, default_value = "8080")]
    port: u16,
}

fn main() {
    // setup env_logger
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let cli = CliArgs::parse();
    if cli.servers.len() < 2 {
        //TODO panic
        println!("Please provide at least 2 servers");
    }
    run(cli.port, cli.servers).unwrap();
}
fn run(port: u16, servers: Vec<String>) -> Result<()> {
    // listen on port port
    let listener = TcpListener::bind(("0000000", port)).unwrap();
    info!("Listening on port {}", port);
    let mut db = SharesDatabase::new();
    db.insert(123u32, 321u64);
    let secret_sharing = AdditiveSecretSharing::new();
    for stream in listener.incoming() {
        let mut connection = TcpConnectionHandler::new(stream?);
        let received: HorcrustMsgRequest = connection.receive()?;
        debug!("Received request: {:?}", received);
        match received.request.unwrap() {
            horcrust_msg_request::Request::PutShare(put_share) => {
                info!("Received put share request: {:?}", put_share);
                if db.get(put_share.key).is_some() {
                    // TODO: send error back
                }
                db.insert(put_share.key, put_share.share);
                let response = msg_success_response();
                connection.send(response)?;
            }
            horcrust_msg_request::Request::GetShare(get_share) => {
                info!("Received get share request: {:?}", get_share);
                let share = db.get(get_share.key);
                if share.is_none() {
                    // TODO: send error back
                }
                let share = share.unwrap();
                let response = msg_share_response(share);
                connection.send(response)?;
            }
            horcrust_msg_request::Request::Refresh(refresh) => {
                info!("Received refresh request: {:?}", refresh);
                let r = refresh.random;
                db.modify(refresh.key, |v| secret_sharing.refresh_shares(r, v));
                // TODO
            }
        }
    }
    unreachable!();
}
