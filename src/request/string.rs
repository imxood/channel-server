use crate::{Body, Error, FromRequest, Request};

impl<'a> FromRequest<'a> for String {
    fn from_request(_req: &'a Request, body: &mut Body) -> Result<Self, Error> {
        let data = body.take()?;
        Ok(String::from_utf8(data.to_vec()).map_err(Error::NotUtf8)?)
    }
}
