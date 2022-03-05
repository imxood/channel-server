use add_data::{AddData, AddDataEndpoint};
use ahash::AHashMap;
use bytes::Bytes;
use crossbeam::channel::{bounded, Receiver, Sender};
use extensions::Extensions;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Formatter},
    ops::Deref,
};

use crate::{
    data::Data,
    request::{json::Json, param},
};

pub mod add_data;
pub mod common;
pub mod data;
pub mod extensions;
pub mod request;
pub mod response;

// pub mod runtime;

pub type Stream = crossbeam::channel::Sender<Response>;

#[derive(Default, Clone)]
pub struct Body(Option<Bytes>);

#[derive(Clone)]
pub struct Param(Option<String>);

impl Body {
    pub fn empty() -> Body {
        Self(None)
    }

    pub fn take(&mut self) -> Result<Bytes, Error> {
        self.0.take().ok_or(Error::BodyNoData)
    }

    pub fn from_string(body: String) -> Body {
        Self(Some(body.into()))
    }

    pub fn from_bytes(body: Bytes) -> Body {
        Self(Some(body))
    }
}

impl Param {
    pub fn empty() -> Self {
        Self(None)
    }

    pub fn from_obj(obj: impl Serialize) -> Self {
        Self(Some(serde_json::to_string(&obj).unwrap()))
    }

    pub fn as_ref(&self) -> Result<&String, Error> {
        if let Some(param) = self.0.as_ref() {
            Ok(param)
        } else {
            Err(Error::ParamNoData)
        }
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
    param: Param,
    /// 用于传递大数据
    body: Body,
    tx: Sender<Response>,
    rx: Receiver<Response>,
    /// 主要是 Middleware 使用的
    extensions: Extensions,
}

impl Request {
    pub fn new(uri: String, param: Param, body: Body) -> Request {
        let (tx, rx) = bounded::<Response>(100);
        Self {
            uri,
            param,
            body,
            tx,
            rx,
            extensions: Extensions::new(),
        }
    }

    pub fn with_param(uri: String, param: Param) -> Request {
        Self::new(uri, param, Body::empty())
    }

    pub fn with_body(uri: String, body: Body) -> Request {
        Self::new(uri, Param::empty(), body)
    }

    /// Returns the parameters used by the extractor.
    pub(crate) fn split(mut self) -> (Request, Body) {
        let body = std::mem::take(&mut self.body);
        (self, body)
    }

    #[inline]
    pub fn uri_ref(&self) -> &str {
        &self.uri
    }

    /// Returns a reference to the associated extensions.
    #[inline]
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    #[inline]
    pub fn param(&self) -> &Param {
        &self.param
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

#[derive(Debug, Clone)]
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

#[derive(Default, Clone)]
pub struct Response {
    uri: String,
    status: StatusCode,
    body: Body,
}

impl Response {
    pub fn new() -> Self {
        Self {
            uri: String::new(),
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

    pub fn status_ref(&self) -> &StatusCode {
        &self.status
    }

    pub fn uri(mut self, uri: String) -> Response {
        self.uri = uri;
        self
    }

    pub fn uri_ref(&self) -> &String {
        &self.uri
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
    #[error("请求已经在队列中")]
    ReqExistInQueue,
    #[error("请求发送失败")]
    ReqSendError,

    /// Io error.
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("解析json异常")]
    ParseJsonError,

    /// Body has been taken by other extractors.
    #[error("the request body has no data")]
    BodyNoData,

    #[error("the request param has no data")]
    ParamNoData,

    /// Body is not a valid utf8 string.
    #[error("parse utf8: {0}")]
    NotUtf8(#[from] std::string::FromUtf8Error),

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
    fn from_request(req: &'a Request, body: &mut Body) -> Result<Self, Error>;
    fn from_request_without_body(req: &'a Request) -> Result<Self, Error> {
        Self::from_request(req, &mut Default::default())
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

pub trait IntoEndpoint {
    type Endpoint: Endpoint;
    fn into_endpoint(self) -> Self::Endpoint;
}

impl<T: Endpoint> IntoEndpoint for T {
    type Endpoint = T;

    fn into_endpoint(self) -> Self::Endpoint {
        self
    }
}

pub trait Middleware<E: Endpoint> {
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
            let res = format!("hello: {}", name);
            println!("\t{}", &res);
            res
        }
        let res = hello(p0);
        Ok(res.into_response())
    }
}

// #[handler]
// fn hello_json(name: Json<String>) -> () {
//     format!("hello: {:?}", &name);
// }

#[derive(Default, Debug, Serialize, Deserialize)]
struct User {
    name: String,
}

struct hello_json;

impl Endpoint for hello_json {
    type Output = Response;

    fn call(&self, req: Request) -> Result<Self::Output, Error> {
        let (req, mut body) = req.split();
        let p0 = <param::Param<User> as FromRequest>::from_request(&req, &mut body)?;
        let p1 = <Json<String> as FromRequest>::from_request(&req, &mut body)?;
        let p2 = <Data<&i32> as FromRequest>::from_request(&req, &mut body)?;
        fn hello_json(user: param::Param<User>, json: Json<String>, data: Data<&i32>) -> String {
            let res = format!("user: {:?}, json: {}, data: {}", user, json.0, data.0);
            println!("\t{}", &res);
            res
        }
        let res = hello_json(p0, p1, p2);
        Ok(res.into_response())
    }
}

#[derive(Default)]
pub struct Route {
    map: AHashMap<&'static str, Box<dyn Endpoint<Output = Response>>>,
}

impl Endpoint for Route {
    type Output = Response;

    fn call(&self, req: Request) -> Result<Self::Output, Error> {
        if self.map.contains_key(req.uri_ref()) {
            let ep = &self.map[req.uri_ref()];
            ep.call(req)
        } else {
            Err(Error::PathNotFoundError(req.uri_ref().into()))
        }
    }
}

impl Route {
    #[must_use]
    pub fn at(mut self, path: &'static str, ep: Box<dyn Endpoint<Output = Response>>) -> Self {
        if self.map.contains_key(path) {
            panic!("duplicate path: {}", path);
        }
        self.map.insert(path, ep);
        self
    }
}

struct ChannelServer {
    rx: Receiver<Request>,
    tx: Sender<Response>,
}

impl ChannelServer {
    fn new(req_rx: Receiver<Request>, res_tx: Sender<Response>) -> ChannelServer {
        Self {
            rx: req_rx,
            tx: res_tx,
        }
    }

    pub fn run(&self, ep: impl Endpoint + Clone + Send + 'static) {
        while let Ok(req) = self.rx.recv() {
            let ep = ep.clone();
            let tx = self.tx.clone();
            std::thread::spawn(move || {
                let res = ep.get_response(req);
                tx.try_send(res).ok();
            });
        }
    }
}
struct ChannelClient {
    tx: Sender<Request>,
    rx: Receiver<Response>,
    res_queue: Vec<Response>,
}

impl ChannelClient {
    pub fn new(tx: Sender<Request>, rx: Receiver<Response>) -> ChannelClient {
        Self {
            tx,
            rx,
            res_queue: Vec::new(),
        }
    }
    /// 发起请求
    pub fn req(&mut self, req: Request) -> Result<(), Error> {
        // 先检查队列中是否有这个请求
        let item = self
            .res_queue
            .iter()
            .find(|res| res.uri_ref() == req.uri_ref());
        if item.is_some() {
            return Err(Error::ReqExistInQueue);
        }

        // 添加 请求状态
        self.res_queue
            .push(Response::new().uri(req.uri_ref().into()));

        // 发送请求
        self.tx.try_send(req).map_err(|e| Error::ReqSendError)
    }

    /// 处理消息队列, 返回一个消息用于显示消息状态
    pub fn poll_once(&mut self) -> Option<&Response> {
        // println!("queue: {:?}", self.queue.len());
        while let Ok(res) = self.rx.try_recv() {
            // 找到对应的req
            let item = self
                .res_queue
                .iter_mut()
                .find(|r| r.uri_ref() == res.uri_ref());
            if let Some(r) = item {
                *r = res;
            }
        }
        self.res_queue.last()
    }
}

struct ChannelService {
    client: ChannelClient,
    server: ChannelServer,
}

impl ChannelService {
    pub fn new() -> ChannelService {
        let (req_tx, req_rx) = bounded::<Request>(100);
        let (res_tx, res_rx) = bounded::<Response>(100);
        let client = ChannelClient::new(req_tx, res_rx);
        let server = ChannelServer::new(req_rx, res_tx);
        Self { client, server }
    }

    pub fn split(mut self) -> (ChannelClient, ChannelServer) {
        let Self { client, server } = self;
        (client, server)
    }
}

#[test]
fn test() {
    let route = Route::default()
        .at("hello", Box::new(hello))
        .at("hello_json", Box::new(hello_json))
        .data(1);

    // 测试1
    let request = Request::with_body(
        "hello".into(),
        Body::from_string("this is a string".to_string()),
    );
    let res = route.get_response(request);
    println!("res: {:?}", res);

    // 测试2
    let request = Request::new(
        "hello_json".into(),
        Param::from_obj(User {
            name: "maxu".to_string(),
        }),
        Body::from_string(serde_json::to_string("this is a json string").unwrap()),
    );
    let res = route.get_response(request);
    println!("res: {:?}", res);
}
