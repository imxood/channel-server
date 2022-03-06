use add_data::{AddData, AddDataEndpoint};
use ahash::AHashMap;
use bytes::Bytes;
use crossbeam::channel::{bounded, Receiver, Sender};
use extensions::Extensions;
use serde::Serialize;
use std::{
    fmt::{Debug, Formatter},
    ops::Deref,
    sync::{Arc, RwLock},
};

pub mod add_data;
pub mod common;
pub mod extensions;
pub mod request;
pub mod response;

pub mod prelude;

pub use inner_derive::handler;

#[derive(Default, Clone)]
pub struct Body(Option<Bytes>);

#[derive(Clone)]
pub struct Param(Option<String>);

impl Body {
    pub fn empty() -> Body {
        Self(None)
    }

    pub fn take(&mut self) -> Result<Bytes, ChannelError> {
        self.0.take().ok_or(ChannelError::BodyNoData)
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

    pub fn as_ref(&self) -> Result<&String, ChannelError> {
        if let Some(param) = self.0.as_ref() {
            Ok(param)
        } else {
            Err(ChannelError::ParamNoData)
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
    /// 主要是 Middleware 使用的
    extensions: Extensions,
}

impl Request {
    pub fn new(uri: impl Into<String>, param: Param, body: Body) -> Request {
        Self {
            uri: uri.into(),
            param,
            body,
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
    pub fn split(mut self) -> (Request, Body) {
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
    /// 执行成功
    Ok(String),
    /// 执行失败
    Fail(String),
    /// 准备就绪
    Ready(String),
    /// 执行中
    Pending(String),
    /// 还未开始
    NotStart(String),
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

    pub fn not_start() -> Self {
        Self::NotStart("未执行".into())
    }

    pub fn is_ok(&self) -> bool {
        matches!(self, StatusCode::Ok(_))
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

    pub fn is_ok(&self) -> bool {
        self.status.is_ok()
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
            .field("uri", &self.uri)
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
    fn call(&self, req: Request) -> Result<Self::Output, ChannelError>;

    fn get_response(&self, req: Request) -> Response {
        let uri = req.uri_ref().to_string();
        let res = self
            .call(req)
            .map(IntoResponse::into_response)
            .unwrap_or_else(|err| err.into_response());
        res.uri(uri)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelError {
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

impl IntoResponse for ChannelError {
    fn into_response(self) -> Response {
        Response::new().status(StatusCode::Fail(self.to_string()))
    }
}

pub trait FromRequest<'a>: Sized {
    fn from_request(req: &'a Request, body: &mut Body) -> Result<Self, ChannelError>;
    fn from_request_without_body(req: &'a Request) -> Result<Self, ChannelError> {
        Self::from_request(req, &mut Default::default())
    }
}

pub type BoxEndpoint<'a, T = Response> = Box<dyn Endpoint<Output = T> + 'a>;

pub trait EndpointExt: IntoEndpoint {
    fn boxed<'a>(self) -> BoxEndpoint<'a, <Self::Endpoint as Endpoint>::Output>
    where
        Self: Sized + 'a,
    {
        Box::new(self.into_endpoint())
    }

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

#[derive(Clone)]
pub struct Route {
    map: Arc<RwLock<AHashMap<&'static str, BoxEndpoint<'static>>>>,
}

impl Route {
    pub fn new() -> Self {
        Self {
            map: Arc::default(),
        }
    }
}

impl Endpoint for Route {
    type Output = Response;

    fn call(&self, req: Request) -> Result<Self::Output, ChannelError> {
        let map = self.map.read().unwrap();
        if map.contains_key(req.uri_ref()) {
            let ep = &map[req.uri_ref()];
            ep.call(req)
        } else {
            Err(ChannelError::PathNotFoundError(req.uri_ref().into()))
        }
    }
}

impl Route {
    #[must_use]
    pub fn at(self, path: &'static str, ep: impl Endpoint<Output = Response> + 'static) -> Self {
        {
            let mut map = self.map.write().unwrap();
            if map.contains_key(path) {
                panic!("duplicate path: {}", path);
            }
            map.insert(path, ep.boxed());
        }
        self
    }
}

struct ChannelServer {
    rx: Receiver<Request>,
    tx: Sender<Response>,
}

impl ChannelServer {
    pub(crate) fn new(req_rx: Receiver<Request>, res_tx: Sender<Response>) -> ChannelServer {
        Self {
            rx: req_rx,
            tx: res_tx,
        }
    }

    pub fn run(self, ep: impl Endpoint + 'static + Clone) {
        std::thread::spawn(move || {
            while let Ok(req) = self.rx.recv() {
                let ep = ep.clone();
                let tx = self.tx.clone();
                std::thread::spawn(move || {
                    let res = ep.get_response(req);
                    tx.try_send(res).ok();
                });
            }
        });
    }
}

#[derive(Clone)]
pub struct ChannelClient {
    tx: Sender<Request>,
    rx: Receiver<Response>,
    res_queue: Vec<Response>,
}

impl ChannelClient {
    pub fn req_with_param(
        &mut self,
        uri: impl Into<String>,
        param: Param,
    ) -> Result<(), ChannelError> {
        let req = Request::new(uri.into(), param, Body::empty());
        self.req(req)
    }

    pub fn req_with_body(
        &mut self,
        uri: impl Into<String>,
        body: Body,
    ) -> Result<(), ChannelError> {
        let req = Request::new(uri.into(), Param::empty(), body);
        self.req(req)
    }

    pub fn req_with_param_body(
        &mut self,
        uri: impl Into<String>,
        param: Param,
        body: Body,
    ) -> Result<(), ChannelError> {
        let req = Request::new(uri.into(), param, body);
        self.req(req)
    }

    /// 发起请求
    pub fn req(&mut self, req: Request) -> Result<(), ChannelError> {
        // 先检查队列中是否有这个请求
        let item = self
            .res_queue
            .iter()
            .find(|res| res.uri_ref() == req.uri_ref());
        if item.is_some() {
            return Err(ChannelError::ReqExistInQueue);
        }

        // 添加 请求状态
        self.res_queue
            .push(Response::new().uri(req.uri_ref().into()));

        // 发送请求
        self.tx.send(req).map_err(|_e| ChannelError::ReqSendError)
    }

    /// 处理消息队列
    /// 返回值为 true 表示 接收到 响应
    pub fn run_once(&mut self) -> bool {
        // println!("queue: {:?}", self.queue.len());
        let mut recved = false;
        while let Ok(res) = self.rx.try_recv() {
            // 找到对应的req
            let item = self
                .res_queue
                .iter_mut()
                .find(|r| r.uri_ref() == res.uri_ref());
            if let Some(r) = item {
                *r = res;
                recved = true;
            }
        }
        recved
    }

    /// 根据 uri 获得请求结果
    pub fn search(&self, uri: &str) -> Option<&Response> {
        self.res_queue.iter().find(|res| res.uri_ref() == uri)
    }

    /// 清除 response
    pub fn clean(&mut self, uri: &str) {
        self.res_queue.retain(|res| res.uri_ref() != uri);
    }

    pub(crate) fn new(tx: Sender<Request>, rx: Receiver<Response>) -> ChannelClient {
        Self {
            tx,
            rx,
            res_queue: Vec::new(),
        }
    }
}

pub struct ChannelService {}

impl ChannelService {
    pub fn start(ep: impl Endpoint + 'static + Clone) -> ChannelClient {
        let (req_tx, req_rx) = bounded::<Request>(100);
        let (res_tx, res_rx) = bounded::<Response>(100);
        let client = ChannelClient::new(req_tx, res_rx);
        let server = ChannelServer::new(req_rx, res_tx);
        server.run(ep);
        client
    }
}
