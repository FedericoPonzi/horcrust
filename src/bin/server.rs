use clap::Parser;
use horcrust::{
    horcrust_msg_request, msg_share_response, msg_success_response, AdditiveSecretSharing,
    HorcrustMsgRequest, Result, ShareResponse,
};
use horcrust::{
    horcrust_msg_response, load_servers_cert_to_root_store, HorcrustMsgError, HorcrustMsgResponse,
    HorcrustSecret, HorcrustShare, HorcrustStoreKey, SecretSharing, SharesDatabase,
};
use log::{debug, info};
use prost::Message;
use rustls::ClientConfig;
use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;

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
    env_logger::init();

    let root_store = load_servers_cert_to_root_store();
    let client_config = Arc::new(
        ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(root_store)
            .with_no_client_auth(),
    );
    let cli = CliArgs::parse();
    if cli.servers.len() < 2 {
        panic!("Please provide at least 2 servers");
    }
    run(cli.port, cli.servers);
}
fn run(port: u16, servers: Vec<String>) -> Result<()> {
    // listen on port port
    let listener = TcpListener::bind(("0000000", port)).unwrap();
    info!("Listening on port {}", port);
    let mut db = SharesDatabase::new();
    let secret_sharing = AdditiveSecretSharing::new();
    for stream in listener.incoming() {
        let mut stream = stream?;
        let mut buf = Vec::new();
        stream.read_to_end(&mut buf)?;
        let received = HorcrustMsgRequest::decode(buf.as_slice())?;
        debug!("Received request: {:?}", received);
        match received.request.unwrap() {
            horcrust_msg_request::Request::PutShare(put_share) => {
                if db.get(put_share.key).is_some() {
                    // TODO: send error back
                }
                db.insert(put_share.key, put_share.share);
                let response = msg_success_response();
                stream.write_all(response.encode_to_vec().as_slice())?;
            }
            horcrust_msg_request::Request::GetShare(get_share) => {
                let share = db.get(get_share.key);
                if share.is_none() {
                    // TODO: send error back
                }
                let share = share.unwrap();
                let response = msg_share_response(share);
                stream.write_all(response.encode_to_vec().as_slice())?;
            }
            horcrust_msg_request::Request::Refresh(refresh) => {
                let r = refresh.random;
                db.modify(refresh.key, |v| secret_sharing.refresh_shares(r, v));
            }
        }
    }
    unreachable!();
}
