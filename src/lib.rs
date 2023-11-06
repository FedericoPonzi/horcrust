mod secret_sharing;

use anyhow::Context;
use prost::Message;
use std::io::{Read, Write};

mod connection;
mod messages;
mod shares_db;

pub use crate::secret_sharing::AdditiveSecretSharing;
pub use crate::secret_sharing::SecretSharing;
pub use connection::{ConnectionHandler, TcpConnectionHandler};
pub use messages::*;
pub use shares_db::SharesDatabase;

pub type HorcrustSecret = u64;
pub type HorcrustStoreKey = u32;
pub type HorcrustShare = u64;
pub type Result<T> = anyhow::Result<T>;

pub fn store_secret(secret: HorcrustSecret, key: HorcrustStoreKey) -> Result<()> {
    unimplemented!()
}

pub fn retrieve_secret(key: HorcrustStoreKey) -> Result<Vec<u8>> {
    unimplemented!()
}

pub fn initiate_refresh(key: u32) -> Result<()> {
    unimplemented!()
}

pub const fn msg_success_response() -> HorcrustMsgResponse {
    HorcrustMsgResponse {
        response: Some(horcrust_msg_response::Response::Error(HorcrustMsgError {
            error: false,
            error_string: String::new(),
        })),
    }
}
pub const fn msg_share_response(share: HorcrustShare) -> HorcrustMsgResponse {
    HorcrustMsgResponse {
        response: Some(horcrust_msg_response::Response::ShareResponse(
            ShareResponse { share },
        )),
    }
}

pub const fn msg_store_share_request(
    key: HorcrustStoreKey,
    share: HorcrustShare,
) -> HorcrustMsgRequest {
    HorcrustMsgRequest {
        request: Some(horcrust_msg_request::Request::PutShare(PutShareRequest {
            key,
            share,
        })),
    }
}
pub const fn msg_retrieve_secret_request(key: HorcrustStoreKey) -> HorcrustMsgRequest {
    HorcrustMsgRequest {
        request: Some(horcrust_msg_request::Request::GetShare(GetShareRequest {
            key,
        })),
    }
}

pub const fn msg_put_share_request(
    key: HorcrustStoreKey,
    share: HorcrustShare,
) -> HorcrustMsgRequest {
    HorcrustMsgRequest {
        request: Some(horcrust_msg_request::Request::PutShare(PutShareRequest {
            key,
            share,
        })),
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_simple() {
        let vec = vec!["1", "2"];
        assert!(vec.contains(&"1"));
    }
}
