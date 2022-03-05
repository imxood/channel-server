use bytes::Bytes;

use crate::{Response, StatusCode, IntoResponse};

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        Response::new().status(StatusCode::ok()).body(self.into())
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        Response::new().status(StatusCode::ok()).body(self.into())
    }
}

impl IntoResponse for &'static [u8] {
    fn into_response(self) -> Response {
        Response::new().status(StatusCode::ok()).body(self.into())
    }
}

impl IntoResponse for Bytes {
    fn into_response(self) -> Response {
        Response::new().status(StatusCode::ok()).body(self)
    }
}

impl IntoResponse for Vec<u8> {
    fn into_response(self) -> Response {
        Response::new().status(StatusCode::ok()).body(self.into())
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        Response::new().status(StatusCode::ok())
    }
}
