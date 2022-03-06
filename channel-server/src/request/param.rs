use std::ops::{Deref, DerefMut};

use serde::de::DeserializeOwned;

use crate::{Body, ChannelError, FromRequest, Request};

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct ReqParam<T>(pub T);

impl<T> Deref for ReqParam<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for ReqParam<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T: DeserializeOwned> FromRequest<'a> for ReqParam<T> {
    fn from_request(req: &'a Request, _body: &mut Body) -> Result<Self, ChannelError> {
        let param = req.param().as_ref()?;
        Ok(Self(
            serde_json::from_slice(param.as_bytes()).map_err(|_e| ChannelError::ParseJsonError)?,
        ))
    }
}
