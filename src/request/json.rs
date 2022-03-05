use std::ops::{Deref, DerefMut};

use serde::{de::DeserializeOwned, Serialize};

use crate::{Body, Error, FromRequest, IntoResponse, Request, Response, StatusCode};

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Json<T>(pub T);

impl<T> Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Json<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T: DeserializeOwned> FromRequest<'a> for Json<T> {
    fn from_request(_req: &'a Request, body: &mut Body) -> Result<Self, Error> {
        let data = body.take()?;
        Ok(Self(
            serde_json::from_slice(data.as_ref()).map_err(|_e| Error::ParseJsonError)?,
        ))
    }
}

impl<T: Serialize + Send> IntoResponse for Json<T> {
    fn into_response(self) -> Response {
        let data = match serde_json::to_vec(&self.0) {
            Ok(data) => data,
            Err(err) => return Response::new().status(StatusCode::Fail(err.to_string())),
        };
        Response::new().status(StatusCode::ok()).body(data.into())
    }
}
