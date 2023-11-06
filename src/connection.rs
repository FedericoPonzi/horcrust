use crate::{HorcrustMsgRequest, HorcrustMsgResponse, RawMessage, Result};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm,
    Key, // Or `Aes128Gcm`
    Nonce,
};
use prost::Message;
use std::io::{Read, Write};
use std::net::Shutdown;

pub trait ConnectionHandler<Req, Res> {
    fn send(&mut self, message: Req) -> Result<()>;
    fn receive(&mut self) -> Result<Res>;
}

pub struct TcpConnectionHandler {
    socket: std::net::TcpStream,
    cipher: Aes256Gcm,
}
impl TcpConnectionHandler {
    pub fn new(socket: std::net::TcpStream) -> Self {
        // TODO super secret key
        let key: &[u8; 32] = &[42; 32];
        let key: &Key<Aes256Gcm> = key.into();
        let cipher = Aes256Gcm::new(key);

        Self { socket, cipher }
    }
}

// server side - for simplicty I've duplicated the code as generated protobuf doesn't come with
// a common trait to reuse encode/decode.
impl ConnectionHandler<HorcrustMsgRequest, HorcrustMsgResponse> for TcpConnectionHandler {
    fn send(&mut self, message: HorcrustMsgRequest) -> Result<()> {
        let mut buf = Vec::new();
        message.encode(&mut buf).unwrap();
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
        let encrypted_payload = self.cipher.encrypt(&nonce, buf.as_ref()).unwrap();
        let nonce = nonce.to_vec();
        let message = RawMessage {
            nonce,
            encrypted_payload,
        };
        message.encode(&mut buf).unwrap();
        self.socket.write_all(&buf)?;
        self.socket.shutdown(Shutdown::Write)?;
        Ok(())
    }

    fn receive(&mut self) -> Result<HorcrustMsgResponse> {
        let mut buf = Vec::new();
        self.socket.read_to_end(&mut buf)?;
        let buf = decrypt_payload(&self.cipher, buf);
        Ok(HorcrustMsgResponse::decode(buf.as_slice())?)
    }
}

/// client side:
impl ConnectionHandler<HorcrustMsgResponse, HorcrustMsgRequest> for TcpConnectionHandler {
    fn send(&mut self, message: HorcrustMsgResponse) -> Result<()> {
        let mut buf = Vec::new();
        message.encode(&mut buf).unwrap();
        self.socket
            .write_all(encrypt_payload(&self.cipher, buf).as_slice())
            .unwrap();
        self.socket.shutdown(Shutdown::Write)?;
        Ok(())
    }

    fn receive(&mut self) -> Result<HorcrustMsgRequest> {
        let mut buf = Vec::new();
        self.socket.read_to_end(&mut buf).unwrap();
        let buf = decrypt_payload(&self.cipher, buf);
        Ok(HorcrustMsgRequest::decode(buf.as_slice())?)
    }
}

fn decrypt_payload(cipher: &Aes256Gcm, encrypted_payload: Vec<u8>) -> Vec<u8> {
    let message = RawMessage::decode(encrypted_payload.as_slice()).unwrap();
    let nonce = Nonce::from_slice(message.nonce.as_slice());
    cipher
        .decrypt(nonce, message.encrypted_payload.as_slice())
        .unwrap()
}
fn encrypt_payload(cipher: &Aes256Gcm, pt_payload: Vec<u8>) -> Vec<u8> {
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng); // 96-bits; unique per message
    let encrypted_payload = cipher.encrypt(&nonce, pt_payload.as_ref()).unwrap();
    let nonce = nonce.to_vec();
    let message = RawMessage {
        nonce,
        encrypted_payload,
    };
    let mut buf = Vec::new();
    message.encode(&mut buf).unwrap();
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{msg_store_share_request, msg_success_response};
    use std::sync::mpsc;
    #[test]
    fn test_encrypt_decrypt() {
        let key = Aes256Gcm::generate_key(OsRng);
        let cipher = Aes256Gcm::new(&key);
        let pt_payload = b"Hello World!";
        let encrypted_payload = encrypt_payload(&cipher, pt_payload.to_vec());
        let decrypted_payload = decrypt_payload(&cipher, encrypted_payload);
        assert_eq!(pt_payload, decrypted_payload.as_slice());
    }

    #[test]
    fn test_tcp_encrypted_channel() {
        let (sender, receiver) = mpsc::channel();
        const REQUEST: HorcrustMsgRequest = msg_store_share_request(1234, 1234);
        const RESPONSE: HorcrustMsgResponse = msg_success_response();
        let server_thread = std::thread::spawn(move || {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let port = listener.local_addr().unwrap().port();
            sender.send(port).expect("Failed to send port");
            let (socket, _) = listener.accept().unwrap();
            let mut handler = TcpConnectionHandler::new(socket);
            let request = handler.receive().unwrap();
            assert_eq!(REQUEST, request);
            handler.send(RESPONSE).unwrap();
        });
        let port = receiver.recv().unwrap();
        let socket = std::net::TcpStream::connect(format!("127.0.0.1:{}", port)).unwrap();
        let mut handler = TcpConnectionHandler::new(socket);
        let request = msg_store_share_request(1234, 1234);
        handler.send(request).unwrap();
        assert_eq!(msg_success_response(), handler.receive().unwrap());
        server_thread.join().unwrap();
    }
}
