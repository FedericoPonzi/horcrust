use crate::{HorcrustMsgRequest, HorcrustMsgResponse, RawMessage, Result};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm,
    Key, // Or `Aes128Gcm`
    Nonce,
};
use num_bigint::BigUint;
use num_traits::ToPrimitive;
use prost::Message;
use rand::random;
use std::io::{Read, Write};
use std::net::Shutdown;
use std::time::Duration;

pub trait ConnectionHandler<Req, Res> {
    fn send(&mut self, message: Req) -> Result<()>;
    fn receive(&mut self) -> Result<Res>;
}

pub struct TcpConnectionHandler {
    socket: std::net::TcpStream,
    cipher: Aes256Gcm,
}
impl TcpConnectionHandler {
    pub fn new(socket: std::net::TcpStream) -> Result<Self> {
        // TODO super secret key
        let key: &[u8; 32] = &[42; 32];
        let key: &Key<Aes256Gcm> = key.into();
        let cipher = Aes256Gcm::new(key);
        socket.set_read_timeout(Some(Duration::from_secs(1)))?;
        socket.set_write_timeout(Some(Duration::from_secs(1)))?;
        let mut ret = Self { socket, cipher };
        // comment this to enable replay attacks :D
        ret.handshake()?;
        Ok(ret)
    }
    pub fn handshake(&mut self) -> Result<()> {
        let (public_key, private_key) = generate_pk(P, G);
        self.socket.write_all(&public_key.to_le_bytes())?;
        let public_key_b = self.handshake_receive_pk()?;
        let session_key = generate_session_key(private_key, public_key_b, P);
        let mut key = vec![42; 32];
        // TODO: should do some key expansion instead.
        let session_key = session_key.to_le_bytes();
        for byte in 0..32 {
            key[byte] = session_key[byte % 8];
        }
        let key = Key::<Aes256Gcm>::from_slice(key.as_slice());
        self.cipher = Aes256Gcm::new(key);
        Ok(())
    }
    fn handshake_receive_pk(&mut self) -> Result<u64> {
        let mut pk = [0; 8];
        self.socket.read_exact(&mut pk)?;
        Ok(u64::from_le_bytes(pk))
    }
}

// server side - for simplicty I've duplicated the code as generated protobuf doesn't come with
// a common trait to reuse encode/decode.
impl ConnectionHandler<HorcrustMsgRequest, HorcrustMsgResponse> for TcpConnectionHandler {
    fn send(&mut self, message: HorcrustMsgRequest) -> Result<()> {
        let mut buf = Vec::new();
        message.encode(&mut buf)?;
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
        let encrypted_payload = self.cipher.encrypt(&nonce, buf.as_ref())?;
        let nonce = nonce.to_vec();
        let message = RawMessage {
            nonce,
            encrypted_payload,
        };
        message.encode(&mut buf)?;
        self.socket.write_all(&buf)?;
        self.socket.shutdown(Shutdown::Write)?;
        Ok(())
    }

    fn receive(&mut self) -> Result<HorcrustMsgResponse> {
        let mut buf = Vec::new();
        self.socket.read_to_end(&mut buf)?;
        let buf = decrypt_payload(&self.cipher, buf)?;
        Ok(HorcrustMsgResponse::decode(buf.as_slice())?)
    }
}

/// client side:
impl ConnectionHandler<HorcrustMsgResponse, HorcrustMsgRequest> for TcpConnectionHandler {
    fn send(&mut self, message: HorcrustMsgResponse) -> Result<()> {
        let mut buf = Vec::new();
        message.encode(&mut buf)?;
        self.socket
            .write_all(encrypt_payload(&self.cipher, buf)?.as_slice())?;
        self.socket.shutdown(Shutdown::Write)?;
        Ok(())
    }

    fn receive(&mut self) -> Result<HorcrustMsgRequest> {
        let mut buf = Vec::new();
        self.socket.read_to_end(&mut buf)?;
        let buf = decrypt_payload(&self.cipher, buf)?;
        Ok(HorcrustMsgRequest::decode(buf.as_slice())?)
    }
}

fn decrypt_payload(cipher: &Aes256Gcm, encrypted_payload: Vec<u8>) -> Result<Vec<u8>> {
    let message = RawMessage::decode(encrypted_payload.as_slice())?;
    let nonce = Nonce::from_slice(message.nonce.as_slice());
    Ok(cipher.decrypt(nonce, message.encrypted_payload.as_slice())?)
}
fn encrypt_payload(cipher: &Aes256Gcm, pt_payload: Vec<u8>) -> Result<Vec<u8>> {
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
    let encrypted_payload = cipher.encrypt(&nonce, pt_payload.as_ref())?;
    let nonce = nonce.to_vec();
    let message = RawMessage {
        nonce,
        encrypted_payload,
    };
    let mut buf = Vec::new();
    message.encode(&mut buf)?;
    Ok(buf)
}

// TODO: bigger P.
const P: u64 = 18446744073709551557;
const G: u64 = 2;

pub fn generate_pk(p: u64, g: u64) -> (u64, u64) {
    let priv_a: u64 = random::<u64>();
    let pub_a = modpow(g, priv_a, p);
    (pub_a, priv_a)
}
pub fn generate_session_key(prvkey_a: u64, pk_b: u64, p: u64) -> u64 {
    // s = (B**a) % p.
    modpow(pk_b, prvkey_a, p)
}

pub fn modpow(base: u64, exp: u64, n: u64) -> u64 {
    let base: BigUint = base.into();
    let exp = exp.into();
    let n = n.into();
    base.modpow(&exp, &n).to_u64().unwrap()
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{msg_store_share_request, msg_success_response};
    use std::sync::mpsc;
    #[test]
    fn test_encrypt_decrypt() -> anyhow::Result<()> {
        let key = Aes256Gcm::generate_key(OsRng);
        let cipher = Aes256Gcm::new(&key);
        let pt_payload = b"Hello World!";
        let encrypted_payload = encrypt_payload(&cipher, pt_payload.to_vec())?;
        let decrypted_payload = decrypt_payload(&cipher, encrypted_payload)?;
        assert_eq!(pt_payload, decrypted_payload.as_slice());
        Ok(())
    }

    #[test]
    fn test_tcp_encrypted_channel() -> anyhow::Result<()> {
        let (sender, receiver) = mpsc::channel();
        const REQUEST: HorcrustMsgRequest = msg_store_share_request(1234, 1234);
        const RESPONSE: HorcrustMsgResponse = msg_success_response();
        let server_thread = std::thread::spawn(move || {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            sender.send(port).expect("Failed to send port");
            let (socket, _) = listener.accept().unwrap();
            let mut handler = TcpConnectionHandler::new(socket).unwrap();
            let request = handler.receive().unwrap();
            assert_eq!(REQUEST, request);
            handler.send(RESPONSE).unwrap();
        });
        let port = receiver.recv()?;
        let socket = std::net::TcpStream::connect(format!("127.0.0.1:{}", port))?;
        let mut handler = TcpConnectionHandler::new(socket)?;
        let request = msg_store_share_request(1234, 1234);
        handler.send(request)?;
        assert_eq!(msg_success_response(), handler.receive()?);
        server_thread.join().unwrap();
        Ok(())
    }
}
