use std::ops::Deref;

use crate::{Body, Error, FromRequest, Request};

pub struct Data<T>(pub T);

impl<T> Deref for Data<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, T: Send + Sync + 'static> FromRequest<'a> for Data<&'a T> {
    fn from_request(req: &'a Request, _body: &mut Body) -> Result<Self, Error> {
        Ok(Data(req.extensions().get::<T>().ok_or_else(|| {
            Error::GetDataError(std::any::type_name::<T>().into())
        })?))
    }
}
