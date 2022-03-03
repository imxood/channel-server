use bytes::Bytes;
use std::{
    fmt::{Debug, Formatter},
    ops::Deref,
};

use crate::json::Json;

pub mod common;
pub mod json;
// pub mod runtime;

pub type Stream = crossbeam::channel::Sender<Response>;

#[derive(Default)]
pub struct Body(Option<Bytes>);

impl Body {
    pub fn take(&mut self) -> Result<Bytes, Error> {
        self.0.take().ok_or(Error::BodyHasBeenTaken)
    }
}

impl Deref for Body {
    type Target = Option<Bytes>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct Request {
    uri: String,
    /// 用于传递参数， json序列化字符串
    headers: String,
    /// 用于传递大数据
    body: Body,
    /// 接收端 用它 和请求端通信
    stream: Stream,
}

impl Request {
    /// Returns the parameters used by the extractor.
    pub fn split(mut self) -> (Request, Body) {
        let body = std::mem::take(&mut self.body);
        (self, body)
    }

    #[inline]
    pub fn uri(&self) -> &String {
        &self.uri
    }
}

impl Debug for Request {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Request").field("uri", &self.uri).finish()
    }
}

#[derive(Debug)]
pub enum StatusCode {
    Ok(String),
    Fail(String),
    Pending(String),
    Ready(String),
}

impl Default for StatusCode {
    fn default() -> Self {
        Self::ready()
    }
}

impl StatusCode {
    pub fn ok() -> Self {
        Self::Ok("执行成功".into())
    }
    pub fn fail() -> Self {
        Self::Fail("执行失败".into())
    }
    pub fn pending() -> Self {
        Self::Pending("正在执行".into())
    }
    pub fn ready() -> Self {
        Self::Ready("准备就绪".into())
    }
}

#[derive(Default)]
pub struct Response {
    status: StatusCode,
    body: Body,
}

impl Response {
    pub fn new() -> Self {
        Self {
            status: StatusCode::ready(),
            body: Body(None),
        }
    }

    pub fn body(mut self, body: Bytes) -> Self {
        self.body = Body(Some(body));
        self
    }

    pub fn status(mut self, status: StatusCode) -> Self {
        self.status = status;
        self
    }

    #[inline]
    pub fn take_body(&mut self) -> Body {
        std::mem::take(&mut self.body)
    }
}

impl Debug for Response {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let len = if let Some(body) = self.body.as_ref() {
            body.len()
        } else {
            0
        };
        f.debug_struct("Response")
            .field("status", &self.status)
            .field("body length", &len)
            .finish()
    }
}

pub trait IntoResponse: Send {
    fn into_response(self) -> Response;
}

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

pub trait Endpoint: Send + Sync {
    /// Represents the response of the endpoint.
    type Output: IntoResponse;

    /// Get the response to the request.
    fn call(&self, req: Request) -> Result<Self::Output, Error>;

    fn get_response(&self, req: Request) -> Response {
        self.call(req)
            .map(IntoResponse::into_response)
            .unwrap_or_else(|err| err.into_response())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Io error.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("解析json异常")]
    ParseJsonError,

    /// Body has been taken by other extractors.
    #[error("the body has been taken")]
    BodyHasBeenTaken,

    /// Body is not a valid utf8 string.
    #[error("parse utf8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("异常: {0}")]
    Custom(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        Response::new().status(StatusCode::Fail(self.to_string()))
    }
}

pub trait FromRequest<'a>: Sized {
    /// Extract from request head and body.
    fn from_request(req: &'a Request, body: &mut Body) -> Result<Self, Error>;

    /// Extract from request head.
    ///
    fn from_request_without_body(req: &'a Request) -> Result<Self, Error> {
        Self::from_request(req, &mut Default::default())
    }
}

impl<'a> FromRequest<'a> for String {
    fn from_request(_req: &'a Request, body: &mut Body) -> Result<Self, Error> {
        let data = body.take()?;
        Ok(String::from_utf8(data.to_vec()).map_err(Error::Utf8)?)
    }
}

// #[handler]
// fn hello(name: String) -> String {
//     format!("hello: {}", name)
// }

struct hello;

impl Endpoint for hello {
    type Output = Response;

    fn call(&self, req: Request) -> Result<Self::Output, Error> {
        let (req, mut body) = req.split();
        let p0 = <String as FromRequest>::from_request(&req, &mut body)?;
        fn hello(name: String) -> String {
            format!("hello: {}", name)
        }
        let res = hello(p0);
        Ok(res.into_response())
    }
}

// #[handler]
// fn hello_json(name: Json<String>) -> () {
//     format!("hello: {:?}", &name);
// }

struct hello_json;

impl Endpoint for hello_json {
    type Output = Response;

    fn call(&self, req: Request) -> Result<Self::Output, Error> {
        let (req, mut body) = req.split();
        let p0 = <Json<String> as FromRequest>::from_request(&req, &mut body)?;
        fn hello_json(name: Json<String>) -> () {
            format!("hello: {:?}", &name);
        }
        let res = hello_json(p0);
        Ok(res.into_response())
    }
}

