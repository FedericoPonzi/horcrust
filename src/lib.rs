mod secret_sharing;

use anyhow::Context;
use prost::Message;
use std::io::{Read, Write};

mod connection;
mod messages;
mod messages_utils;
mod shares_db;

pub use crate::secret_sharing::AdditiveSecretSharing;
pub use crate::secret_sharing::SecretSharing;
pub use connection::{ConnectionHandler, TcpConnectionHandler};
pub use messages::*;
pub use messages_utils::*;
pub use shares_db::SharesDatabase;

/// type alias for the secret. It's just a single number which should be less than Q (defined in secret_sharing).
pub type HorcrustSecret = u64;
/// type alias for the Store Key.
pub type HorcrustStoreKey = u32;
/// A type alias for the Share type.
pub type HorcrustShare = u64;
/// our own result type, TODO: implement using thiserror.
pub type Result<T> = anyhow::Result<T>;
