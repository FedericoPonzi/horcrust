mod secret_sharing;

use anyhow::{Context, Result};
use log::debug;
use prost::Message;
use rustls::RootCertStore;
use rustls_pemfile::certs;
use std::io::{BufReader, Read, Write};
use std::net::TcpStream;

mod messages;
use crate::secret_sharing::SecretSharing;
pub use messages::*;

const SERVERS_CERT: &[u8] = include_bytes!("../cert.pem");

pub type HorcrustSecret = u32;
pub type HorcrustStoreKey = u32;

pub fn load_servers_cert_to_root_store() -> RootCertStore {
    let mut root_store = RootCertStore::empty();
    root_store.add_trust_anchors(webpki_roots::TLS_SERVER_ROOTS.iter().map(|ta| {
        rustls::OwnedTrustAnchor::from_subject_spki_name_constraints(
            ta.subject,
            ta.spki,
            ta.name_constraints,
        )
    }));
    let mut reader = BufReader::new(SERVERS_CERT);
    certs(&mut reader)
        .unwrap()
        .into_iter()
        .map(rustls::Certificate)
        .for_each(|cert| {
            root_store
                .add(&cert)
                .expect("Failed to add certificate to root store")
        });
    root_store
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

pub fn put_secret(mut socket: TcpStream, secret: u32, key: u32) -> Result<()> {
    let req = PutSecretRequest { secret, key };
    let mut buf = Vec::new();
    // Serialize the message into a byte array.
    req.encode(&mut buf)?;
    socket
        .write_all(&buf)
        .context("Failed at writing onto the unix stream")?;
    let mut buf = Vec::new();
    socket.read_to_end(&mut buf)?;
    let received = HorcrustMsgResponse::decode(buf.as_slice())?;
    debug!("Received message: {:?}", received);
    Ok(())
}

pub fn get_secret(mut socket: TcpStream, key: u32) -> Result<HorcrustSecret> {
    let req = GetSecretRequest { key };
    let mut buf = Vec::new();
    // Serialize the message into a byte array.
    req.encode(&mut buf)?;
    socket
        .write_all(&buf)
        .context("Failed at writing onto the unix stream")?;
    let mut buf = Vec::new();
    socket.read_to_end(&mut buf)?;
    let received = HorcrustMsgResponse::decode(buf.as_slice())?;
    debug!("Received message: {:?}", received);
    match received.response.unwrap() {
        horcrust_msg_response::Response::Error(HorcrustMsgError {
            error,
            error_string,
        }) => {
            panic!("Error: {} {}", error, error_string);
        }
        horcrust_msg_response::Response::SecretResponse(secret) => {
            return Ok(secret.secret);
        }
    }
}

pub fn store_secret(secret: HorcrustSecret, key: u32) -> Result<()> {
    unimplemented!()
}

pub fn retrieve_secret(key: u32) -> Result<Vec<u8>> {
    unimplemented!()
}

pub fn initiate_refresh(key: u32) -> Result<()> {
    unimplemented!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
