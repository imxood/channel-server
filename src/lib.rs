use add_data::{AddData, AddDataEndpoint};
use ahash::AHashMap;
use bytes::Bytes;
use extensions::Extensions;
use std::{
    any::Any,
    fmt::{Debug, Formatter},
    ops::Deref,
};

use crate::json::Json;

pub mod add_data;
pub mod common;
pub mod data;
pub mod extensions;
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
    /// 主要是 Middleware 使用的
    extensions: Extensions,
}

impl Request {
    /// Returns the parameters used by the extractor.
    pub fn split(mut self) -> (Request, Body) {
        let body = std::mem::take(&mut self.body);
        (self, body)
    }

    #[inline]
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Returns a reference to the associated extensions.
    #[inline]
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    /// Returns a mutable reference to the associated extensions.
    #[inline]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
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

    #[error("路径未找到: {0}")]
    PathNotFoundError(String),

    /// 获取Data异常.
    #[error("Get data 异常: {0}")]
    GetDataError(String),

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

pub trait EndpointExt: IntoEndpoint {
    fn data<T>(self, data: T) -> AddDataEndpoint<Self::Endpoint, T>
    where
        T: Clone + Send + Sync + 'static,
        Self: Sized,
    {
        self.with(AddData::new(data))
    }

    fn with<T>(self, middleware: T) -> T::Output
    where
        T: Middleware<Self::Endpoint>,
        Self: Sized,
    {
        middleware.transform(self.into_endpoint())
    }
}

impl<T: IntoEndpoint> EndpointExt for T {}

/// Represents a type that can convert into endpoint.
pub trait IntoEndpoint {
    /// Represents the endpoint type.
    type Endpoint: Endpoint;

    /// Converts this object into endpoint.
    fn into_endpoint(self) -> Self::Endpoint;
}

impl<T: Endpoint> IntoEndpoint for T {
    type Endpoint = T;

    fn into_endpoint(self) -> Self::Endpoint {
        self
    }
}

pub trait Middleware<E: Endpoint> {
    /// New endpoint type.
    ///
    /// If you don't know what type to use, then you can use
    /// [`BoxEndpoint`](crate::endpoint::BoxEndpoint), which will bring some
    /// performance loss, but it is insignificant.
    type Output: Endpoint;

    /// Transform the input [`Endpoint`] to another one.
    fn transform(&self, ep: E) -> Self::Output;
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

/// The top-level builder for an Actix Web application.
///
struct ChannelServer {
    extensions: Extensions,
}

impl ChannelServer {
    pub fn data<U: 'static>(mut self, ext: U) -> Self {
        self.extensions.insert(ext);
        self
    }
}

pub struct Route {
    map: AHashMap<&'static str, Box<dyn Endpoint<Output = Response>>>,
}

impl Endpoint for Route {
    type Output = Response;

    fn call(&self, mut req: Request) -> Result<Self::Output, Error> {
        if self.map.contains_key(req.uri()) {
            let ep = &self.map[req.uri()];
            ep.call(req)
        } else {
            Err(Error::PathNotFoundError(req.uri().into()))
        }
    }
}

impl Route {
    pub fn new() -> Self {
        Self {
            map: AHashMap::new(),
        }
    }

    #[must_use]
    pub fn at(mut self, path: &'static str, ep: Box<dyn Endpoint<Output = Response>>) -> Self {
        if self.map.contains_key(path) {
            panic!("duplicate path: {}", path);
        }
        self.map.insert(path, ep);
        self
    }
}

fn test() {
    let route = Route::new().at("hello", Box::new(hello.data(1)));
}
