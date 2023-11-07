use anyhow::Context;
use clap::{Parser, Subcommand};
use env_logger::Env;
use horcrust::horcrust_msg_response::Response;
use horcrust::{
    msg_put_share_request, msg_retrieve_secret_request, ConnectionHandler, HorcrustMsgError,
    HorcrustMsgResponse, HorcrustSecret, HorcrustShare, HorcrustStoreKey, Result, SecretSharing,
    TcpConnectionHandler,
};
use log::{debug, info};

/// Create shares out of your secret and stores them to distributed stores. Allows you
/// to safely recover your secret from the shares on a later moment.
#[derive(Parser, Clone, Debug)]
#[command(author, version, about, long_about = None)]
struct CliArgs {
    #[arg(short, long, required = true)]
    /// a list of servers to store your secret. Please provide at least 2 servers.
    servers: Vec<String>,
    #[command(subcommand)]
    subcommands: Command,
}

#[derive(Subcommand, Clone, Debug)]
enum Command {
    StoreSecret {
        key: HorcrustStoreKey,
        secret: HorcrustSecret,
    },
    RetrieveSecret {
        key: HorcrustStoreKey,
    },
}

fn main() {
    // setup env_logger
    env_logger::Builder::from_env(Env::default().default_filter_or("debug")).init();

    let cli = CliArgs::parse();
    if cli.servers.len() < 2 {
        panic!("Please provide at least 2 servers");
    }
    info!("Hello!");
    dbg!(&cli);
    let additive_sharing = horcrust::AdditiveSecretSharing::default();
    let shares_len = cli.servers.len();
    match cli.subcommands {
        Command::RetrieveSecret { key } => {
            info!(
                "Retrieving secret with key '{key}' from servers: {:?}",
                cli.servers
            );
            let shares = cli
                .servers
                .into_iter()
                .map(|server| reterieve_secret(key, server).expect("Retrieve failed"))
                .collect();
            let secret = additive_sharing.combine(shares);
            println!("Recovered secret: {}", secret);
        }
        Command::StoreSecret { key, secret } => {
            info!(
                "Storing secret {secret} with key '{key}' to servers: {:?}",
                cli.servers
            );
            let shares = additive_sharing.split(shares_len, secret);
            shares
                .into_iter()
                .zip(cli.servers)
                .for_each(|(share, server)| {
                    put_share(key, share, server).expect("Store failed");
                });
        }
    }
    //test_tls_rust(client_config);
}

// retrieves a secret from a single server.
fn reterieve_secret(key: HorcrustStoreKey, server: String) -> Result<HorcrustSecret> {
    let socket = std::net::TcpStream::connect(&server).unwrap();
    let mut handler = TcpConnectionHandler::new(socket);
    let request = msg_retrieve_secret_request(key);
    handler.send(request)?;
    let received: HorcrustMsgResponse = handler.receive().unwrap();
    match received.response.unwrap() {
        Response::Error(HorcrustMsgError {
            error,
            error_string,
        }) => {
            panic!(
                "Error response from server '{}' : {} {}",
                server, error, error_string
            );
        }
        Response::ShareResponse(share) => {
            println!("Share received: {}", share.share);
            Ok(share.share as HorcrustSecret)
        }
    }
}

fn put_share(key: HorcrustStoreKey, share: HorcrustShare, server: String) -> Result<()> {
    let req = msg_put_share_request(key, share);
    let socket = std::net::TcpStream::connect(&server).unwrap();
    let mut handler = TcpConnectionHandler::new(socket);
    handler.send(req)?;
    debug!("fetching server response: ");
    let received: HorcrustMsgResponse = handler
        .receive()
        .context(format!("failed handler receive with server: {server}"))?;

    match received.response.unwrap() {
        Response::Error(HorcrustMsgError {
            error,
            error_string,
        }) => {
            if error {
                panic!(
                    "Error response from server '{}' : {} {}",
                    server, error, error_string
                );
            } else {
                println!("Share stored successfully on server: {}", server);
            }
        }
        resp => {
            panic!("Unexpected response from server '{}': {:?} ", server, resp);
        }
    }
    Ok(())
}
