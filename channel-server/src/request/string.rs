use crate::{Body, ChannelError, FromRequest, Request};

impl<'a> FromRequest<'a> for String {
    fn from_request(_req: &'a Request, body: &mut Body) -> Result<Self, ChannelError> {
        let data = body.take()?;
        Ok(String::from_utf8(data.to_vec()).map_err(ChannelError::NotUtf8)?)
    }
}
