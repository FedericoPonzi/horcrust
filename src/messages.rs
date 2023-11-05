#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HorcrustMsgMessage {
    #[prost(oneof = "horcrust_msg_message::MessageType", tags = "1, 2")]
    pub message_type: ::core::option::Option<horcrust_msg_message::MessageType>,
}
/// Nested message and enum types in `HorcrustMsgMessage`.
pub mod horcrust_msg_message {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum MessageType {
        #[prost(message, tag = "1")]
        Request(super::HorcrustMsgRequest),
        #[prost(message, tag = "2")]
        Response(super::HorcrustMsgResponse),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HorcrustMsgRequest {
    #[prost(oneof = "horcrust_msg_request::Request", tags = "1, 2")]
    pub request: ::core::option::Option<horcrust_msg_request::Request>,
}
/// Nested message and enum types in `HorcrustMsgRequest`.
pub mod horcrust_msg_request {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Request {
        #[prost(message, tag = "1")]
        StatusRequest(super::PutSecretRequest),
        #[prost(message, tag = "2")]
        ChangeRequest(super::GetSecretRequest),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HorcrustMsgResponse {
    #[prost(oneof = "horcrust_msg_response::Response", tags = "1, 2")]
    pub response: ::core::option::Option<horcrust_msg_response::Response>,
}
/// Nested message and enum types in `HorcrustMsgResponse`.
pub mod horcrust_msg_response {
    #[allow(clippy::derive_partial_eq_without_eq)]
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Response {
        #[prost(message, tag = "1")]
        Error(super::HorcrustMsgError),
        #[prost(message, tag = "2")]
        SecretResponse(super::SecretResponse),
    }
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HorcrustMsgError {
    #[prost(bool, tag = "1")]
    pub error: bool,
    #[prost(string, tag = "2")]
    pub error_string: ::prost::alloc::string::String,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PutSecretRequest {
    #[prost(uint32, tag = "1")]
    pub key: u32,
    #[prost(uint32, tag = "2")]
    pub secret: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetSecretRequest {
    #[prost(uint32, tag = "1")]
    pub key: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RefreshSecretRequest {
    #[prost(uint32, tag = "1")]
    pub refresh: u32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SecretResponse {
    #[prost(uint32, tag = "1")]
    pub secret: u32,
}
