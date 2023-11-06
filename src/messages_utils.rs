use crate::{
    horcrust_msg_request, horcrust_msg_response, GetShareRequest, HorcrustMsgError,
    HorcrustMsgRequest, HorcrustMsgResponse, HorcrustShare, HorcrustStoreKey, PutShareRequest,
    RefreshShareRequest, ShareResponse,
};

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

pub const fn msg_refresh_share_request(
    key: Vec<HorcrustStoreKey>,
    random: HorcrustShare,
) -> HorcrustMsgRequest {
    HorcrustMsgRequest {
        request: Some(horcrust_msg_request::Request::Refresh(
            RefreshShareRequest { key, random },
        )),
    }
}

pub fn msg_error_response(msg: &str) -> HorcrustMsgResponse {
    HorcrustMsgResponse {
        response: Some(horcrust_msg_response::Response::Error(HorcrustMsgError {
            error: true,
            error_string: msg.to_string(),
        })),
    }
}
