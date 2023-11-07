use std::net::TcpListener;
use std::sync::{Arc, Mutex};

use clap::Parser;
use env_logger::Env;
use log::{debug, info};
use rand::random;

use horcrust::{
    horcrust_msg_request, horcrust_msg_response, msg_error_response, msg_refresh_share_request,
    msg_share_response, msg_success_response, AdditiveSecretSharing, ConnectionHandler,
    HorcrustMsgError, HorcrustMsgRequest, HorcrustMsgResponse, Result, SecretSharing,
    TcpConnectionHandler,
};
use horcrust_server::SharesDatabase;

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
        println!("Please provide at least 2 servers. Include this server's address as well.");
    }
    debug!("cli: {:?}", cli);
    run(cli.port, cli.servers).unwrap();
}
fn run(port: u16, servers: Vec<String>) -> Result<()> {
    // listen on port port
    let listener = TcpListener::bind(("0000000", port))?;
    info!("Listening on port {}", port);
    let db = Arc::new(Mutex::new(SharesDatabase::new()));
    let secret_sharing = AdditiveSecretSharing::default();
    spawn_refresher(servers, db.clone());
    for stream in listener.incoming() {
        let mut connection = TcpConnectionHandler::new(stream?);
        let received: HorcrustMsgRequest = connection.receive()?;
        debug!("Received request: {:?}", received);
        match received.request? {
            horcrust_msg_request::Request::PutShare(put_share) => {
                info!("Received put share request: {:?}", put_share);
                // this overwrites whatever was there before
                let mut db_lock = db.lock()?;
                db_lock.insert(put_share.key, put_share.share);
                let response = msg_success_response();
                connection.send(response)?;
            }
            horcrust_msg_request::Request::GetShare(get_share) => {
                info!("Received get share request: {:?}", get_share);
                let db_lock = db.lock()?;
                let share_opt = db_lock.get(get_share.key);
                if let Some(share) = share_opt {
                    let response = msg_share_response(share);
                    connection.send(response)?;
                } else {
                    let response =
                        msg_error_response("Key not found. Use store-key to store a key first.");
                    connection.send(response)?;
                }
            }
            horcrust_msg_request::Request::Refresh(refresh) => {
                info!("Received refresh request: {:?}", refresh);
                let r = refresh.random;
                let mut db_lock = db.lock()?;
                for key in refresh.key {
                    db_lock.modify(key, |v| secret_sharing.refresh_share(r, v));
                }
                let response = msg_success_response();
                connection.send(response)?;
            }
        }
    }
    unreachable!();
}

pub fn spawn_refresher(servers: Vec<String>, db: Arc<Mutex<SharesDatabase>>) {
    std::thread::spawn(move || refresher(servers, db));
}
pub fn refresher(servers: Vec<String>, db: Arc<Mutex<SharesDatabase>>) -> Result<()> {
    info!("Spawned refresher thread.");
    loop {
        // wait at least 2 seconds + between 1 and 10 seconds
        let time_to_wait = 2 + (random::<f32>() * 10.0) as u64;
        debug!("Waiting for {} seconds", time_to_wait);
        std::thread::sleep(std::time::Duration::from_secs(time_to_wait));
        debug!("Starting refreshing");
        let refreshers = AdditiveSecretSharing::default().generate_refreshers(servers.len());
        let db_lock = db.lock()?;
        let stale_keys = db_lock.stale_keys();
        drop(db_lock);
        // all good
        if stale_keys.is_empty() {
            debug!("No stale keys to refresh.");
            continue;
        }
        for (server, r) in servers.iter().zip(refreshers) {
            let socket = std::net::TcpStream::connect(server)?;
            let mut handler = TcpConnectionHandler::new(socket);
            let request = msg_refresh_share_request(stale_keys.clone(), r);
            handler.send(request)?;
            let response: HorcrustMsgResponse = handler.receive()?;
            match response.response.? {
                horcrust_msg_response::Response::Error(HorcrustMsgError {
                    error,
                    error_string,
                }) => {
                    if error {
                        info!(
                            "Failed to refresh shares on server {}, error: {}",
                            server, error_string
                        );
                    }
                }
                _ => {
                    info!("Unknown response from server {}", server);
                }
            }
        }
    }
}
