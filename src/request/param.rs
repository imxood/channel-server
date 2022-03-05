use std::ops::{Deref, DerefMut};

use serde::{de::DeserializeOwned, Serialize};

use crate::{Body, Error, FromRequest, IntoResponse, Request, Response, StatusCode};

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Param<T>(pub T);

impl<T> Deref for Param<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Param<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<'a, T: DeserializeOwned> FromRequest<'a> for Param<T> {
    fn from_request(req: &'a Request, _body: &mut Body) -> Result<Self, Error> {
        let param = req.param().as_ref()?;
        Ok(Self(
            serde_json::from_slice(param.as_bytes()).map_err(|_e| Error::ParseJsonError)?,
        ))
    }
}
