use bytes::Bytes;
use std::error::Error as StdError;
use std::fmt::{Debug, Display, Formatter};
use std::num::NonZeroU8;
use std::string::FromUtf8Error;

pub mod common;
pub mod runtime;

struct hello;

impl Endpoint for hello {
    type Output = Response;

    fn call(&self, req: Request) -> Result<Self::Output> {
        let (req, mut body) = req.split();
        let p0 = <String as FromRequest>::from_request(&req, &mut body).await?;
        fn hello(a: String) -> String {
            a + "1"
        }
        let res = hello(p0);
        let res = poem::error::IntoResult::into_result(res);
        std::result::Result::map(res, poem::IntoResponse::into_response)
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Represents an HTTP request.
#[derive(Default)]
pub struct Request {
    uri: String,
    body: Body,
}

#[derive(Default)]
pub struct Body(Option<BodyKind>);

impl Body {
    #[inline]
    pub fn empty() -> Self {
        Self(None)
    }
    pub fn into_bytes(self) -> Result<Bytes, ReadBodyError> {
        if let Some(body) = self.0 {

        }
        hyper::body::to_bytes(self.0)
            .map_err(|err| ReadBodyError::Io(IoError::new(ErrorKind::Other, err)))
    }
}

impl Debug for Body {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Body").finish()
    }
}

impl From<String> for Body {
    #[inline]
    fn from(data: String) -> Self {
        Self(Some(BodyKind::Once(Some(Bytes::from(data.into_bytes())))))
    }
}

#[derive(Default)]
pub struct RequestBody(Option<Body>);

impl RequestBody {
    /// Create a new request body.
    pub fn new(body: Body) -> Self {
        Self(Some(body))
    }

    /// Take a body, if it has already been taken, an error with the status code
    /// [`StatusCode::INTERNAL_SERVER_ERROR`] is returned.
    pub fn take(&mut self) -> Result<Body, ReadBodyError> {
        Ok(self.0.take().ok_or(ReadBodyError::BodyHasBeenTaken)?)
    }

    /// Returns `true` if body exists.
    #[inline]
    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    /// Returns `true` if body does not exists.
    #[inline]
    pub fn is_none(&self) -> bool {
        self.0.is_none()
    }
}

enum BodyKind {
    Once(Option<Bytes>),
}

impl Request {
    pub fn split(mut self) -> (Request, RequestBody) {
        let body = self.take_body();
        (self, RequestBody::new(body))
    }

    #[inline]
    pub fn take_body(&mut self) -> Body {
        std::mem::take(&mut self.body)
    }
}

impl Debug for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Request")
            .field("uri", &self.uri)
            .field("body", &self.body)
            .finish()
    }
}

pub trait Endpoint: Send + Sync {
    /// Represents the response of the endpoint.
    type Output: IntoResponse;

    /// Get the response to the request.
    fn call(&self, req: Request) -> Result<Self::Output>;

    fn get_response(&self, req: Request) -> Response {
        self.call(req)
            .map(IntoResponse::into_response)
            .unwrap_or_else(|err| err.as_response())
    }
}

/// Represents an HTTP response.
#[derive(Default)]
pub struct Response {
    status: StatusCode,
    body: Body,
}

impl Response {
    pub fn set_body(&mut self, body: impl Into<Body>) {
        self.body = body.into();
    }
    #[inline]
    pub fn set_status(&mut self, status: StatusCode) {
        self.status = status;
    }
    pub fn builder() -> ResponseBuilder {
        ResponseBuilder {
            status: StatusCode::READY,
        }
    }
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl Debug for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Response")
            .field("status", &self.status)
            .field("body", &self.body)
            .finish()
    }
}

pub struct ResponseBuilder {
    status: StatusCode,
}

impl ResponseBuilder {
    /// Sets the HTTP status for this response.
    ///
    /// By default this is [`StatusCode::OK`].
    #[must_use]
    pub fn status(self, status: StatusCode) -> Self {
        Self { status, ..self }
    }

    /// Consumes this builder, using the provided body to return a constructed
    /// [Response].
    pub fn body(self, body: impl Into<Body>) -> Response {
        Response {
            status: self.status,
            body: body.into(),
        }
    }

    /// Consumes this builder, using an empty body to return a constructed
    /// [Response].
    pub fn finish(self) -> Response {
        self.body(Body::empty())
    }
}

pub trait IntoResponse: Send {
    /// Consume itself and return [`Response`].
    fn into_response(self) -> Response;

    fn with_status(self, status: StatusCode) -> WithStatus<Self>
    where
        Self: Sized,
    {
        WithStatus {
            inner: self,
            status,
        }
    }

    fn with_body(self, body: impl Into<Body>) -> WithBody<Self>
    where
        Self: Sized,
    {
        WithBody {
            inner: self,
            body: body.into(),
        }
    }
}

pub struct WithStatus<T> {
    inner: T,
    status: StatusCode,
}

impl<T: IntoResponse> IntoResponse for WithStatus<T> {
    fn into_response(self) -> Response {
        let mut resp = self.inner.into_response();
        resp.set_status(self.status);
        resp
    }
}

pub struct WithBody<T> {
    inner: T,
    body: Body,
}

impl<T: IntoResponse> IntoResponse for WithBody<T> {
    fn into_response(self) -> Response {
        let mut resp = self.inner.into_response();
        resp.set_body(self.body);
        resp
    }
}

pub trait ResponseError {
    /// The status code of this error.
    fn status(&self) -> StatusCode;

    /// Convert this error to a HTTP response.
    fn as_response(&self) -> Response
    where
        Self: std::error::Error + Send + Sync + 'static,
    {
        Response {
            status: self.status(),
            body: self.to_string().into(),
        }
    }
}

/// A possible error value when reading the body.
#[derive(Debug, thiserror::Error)]
pub enum ReadBodyError {
    /// Body has been taken by other extractors.
    #[error("the body has been taken")]
    BodyHasBeenTaken,

    /// Body is not a valid utf8 string.
    #[error("parse utf8: {0}")]
    Utf8(#[from] FromUtf8Error),

    /// Payload too large
    #[error("payload too large")]
    PayloadTooLarge,

    /// Io error.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
}

impl ResponseError for ReadBodyError {
    fn status(&self) -> StatusCode {
        StatusCode::FAIL
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StatusCode(NonZeroU8);

impl StatusCode {
    pub fn canonical_reason(&self) -> Option<&'static str> {
        canonical_reason(self.0.get())
    }
}

macro_rules! status_codes {
    (
        $(
            $(#[$docs:meta])*
            ($num:expr, $konst:ident, $phrase:expr);
        )+
    ) => {
        impl StatusCode {
        $(
            $(#[$docs])*
            pub const $konst: StatusCode = StatusCode(unsafe { NonZeroU8::new_unchecked($num) });
        )+

        }

        fn canonical_reason(num: u8) -> Option<&'static str> {
            match num {
                $(
                $num => Some($phrase),
                )+
                _ => None
            }
        }
    }
}

status_codes! {
    (0, SUCCESS, "Success");
    (1, FAIL, "Fail");
    (2, PROCESSING, "Processing");
    (3, READY, "Ready");
}

impl Default for StatusCode {
    #[inline]
    fn default() -> StatusCode {
        StatusCode::READY
    }
}

impl std::fmt::Debug for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, f)
    }
}

impl std::fmt::Display for StatusCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            u8::from(*self),
            self.canonical_reason().unwrap_or("<unknown status code>")
        )
    }
}

impl From<StatusCode> for u8 {
    #[inline]
    fn from(status: StatusCode) -> u8 {
        status.0.get()
    }
}

pub struct Error {
    as_response: AsResponse,
    source: ErrorSource,
}

impl Error {
    #[inline]
    pub fn downcast_ref<T: StdError + Send + Sync + 'static>(&self) -> Option<&T> {
        match &self.source {
            ErrorSource::BoxedError(err) => err.downcast_ref::<T>(),
            #[cfg(feature = "anyhow")]
            ErrorSource::Anyhow(err) => err.downcast_ref::<T>(),
            #[cfg(feature = "eyre06")]
            ErrorSource::Eyre06(err) => err.downcast_ref::<T>(),
        }
    }
    /// Consumes this to return a response object.
    pub fn as_response(&self) -> Response {
        self.as_response.as_response(self)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Error")
            .field("source", &self.source)
            .finish()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.source {
            ErrorSource::BoxedError(err) => Display::fmt(err, f),
            #[cfg(feature = "anyhow")]
            ErrorSource::Anyhow(err) => Display::fmt(err, f),
            #[cfg(feature = "eyre06")]
            ErrorSource::Eyre06(err) => Display::fmt(err, f),
        }
    }
}

enum AsResponse {
    Status(StatusCode),
    Fn(BoxAsResponseFn),
}

impl AsResponse {
    #[inline]
    fn from_status(status: StatusCode) -> Self {
        AsResponse::Status(status)
    }

    fn from_type<T: ResponseError + StdError + Send + Sync + 'static>() -> Self {
        AsResponse::Fn(Box::new(|err| {
            let err = err.downcast_ref::<T>().expect("valid error");
            err.as_response()
        }))
    }

    fn as_response(&self, err: &Error) -> Response {
        match self {
            AsResponse::Status(status) => Response::builder().status(*status).body(err.to_string()),
            AsResponse::Fn(f) => f(err),
        }
    }
}

type BoxAsResponseFn = Box<dyn Fn(&Error) -> Response + Send + Sync + 'static>;

enum ErrorSource {
    BoxedError(Box<dyn StdError + Send + Sync>),
    #[cfg(feature = "anyhow")]
    Anyhow(anyhow::Error),
    #[cfg(feature = "eyre06")]
    Eyre06(eyre06::Report),
}

impl Debug for ErrorSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSource::BoxedError(err) => Debug::fmt(err, f),
            #[cfg(feature = "anyhow")]
            ErrorSource::Anyhow(err) => Debug::fmt(err, f),
            #[cfg(feature = "eyre06")]
            ErrorSource::Eyre06(err) => Debug::fmt(err, f),
        }
    }
}

pub trait IntoResult<T: IntoResponse> {
    fn into_result(self) -> Result<T>;
}

impl<T, E> IntoResult<T> for Result<T, E>
where
    T: IntoResponse,
    E: Into<Error> + Debug + Send + Sync + 'static,
{
    #[inline]
    fn into_result(self) -> Result<T> {
        self.map_err(Into::into)
    }
}

impl<T: IntoResponse> IntoResult<T> for T {
    #[inline]
    fn into_result(self) -> Result<T> {
        Ok(self)
    }
}

pub trait FromRequest<'a>: Sized {
    /// Extract from request head and body.
    fn from_request(req: &'a Request, body: &mut RequestBody) -> Result<Self>;

    /// Extract from request head.
    ///
    /// If you know that this type does not need to extract the body, then you
    /// can just use it.
    ///
    /// For example [`Query`], [`Path`] they only extract the content from the
    /// request head, using this method would be more convenient.
    /// `String`,`Vec<u8>` they extract the body of the request, using this
    /// method will cause `ReadBodyError` error.
    fn from_request_without_body(req: &'a Request) -> Result<Self> {
        Self::from_request(req, &mut Default::default())
    }
}

impl<'a> FromRequest<'a> for String {
    fn from_request(_req: &'a Request, body: &mut RequestBody) -> Result<Self> {
        let data = body.take()?.into_bytes()?;
        Ok(String::from_utf8(data.to_vec()).map_err(ReadBodyError::Utf8)?)
    }
}
