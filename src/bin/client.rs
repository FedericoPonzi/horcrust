use clap::{Parser, Subcommand};
use horcrust::{load_servers_cert_to_root_store, HorcrustSecret, HorcrustStoreKey};
use log::info;
use rustls::ClientConfig;
use std::convert::TryInto;
use std::io::{stdout, Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::Arc;

/// Create shares out of your secret and stores them to distributed services. Allows you
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
    dbg!(&cli);
    match cli.subcommands {
        Command::RetrieveSecret { key } => {
            info!(
                "Retrieving secret with key '{key}' from servers: {:?}",
                cli.servers
            );
        }
        Command::StoreSecret { key, secret } => {
            info!(
                "Storing secret {secret} with key '{key}' to servers: {:?}",
                cli.servers
            );
        }
    }
    //test_tls_rust(client_config);
}

fn test_tls_rust(client_config: Arc<ClientConfig>) {
    let server_name = "www.rust-lang.org".try_into().unwrap();
    let mut conn = rustls::ClientConnection::new(client_config, server_name).unwrap();
    let mut sock = TcpStream::connect("www.rust-lang.org:443").unwrap();
    let mut tls = rustls::Stream::new(&mut conn, &mut sock);
    tls.write_all(
        concat!(
            "GET / HTTP/1.1\r\n",
            "Host: www.rust-lang.org\r\n",
            "Connection: close\r\n",
            "Accept-Encoding: identity\r\n",
            "\r\n"
        )
        .as_bytes(),
    )
    .unwrap();
    let mut plaintext = Vec::new();
    tls.read_to_end(&mut plaintext).unwrap();
    stdout().write_all(&plaintext).unwrap();
}
