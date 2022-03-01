use std::time::Instant;

use anyhow::Result;
use crossbeam::channel::{bounded, Receiver, Sender};
use egui::{ComboBox, Label, RichText, Ui, WidgetText, Color32};

use crate::common::utils::RcCell;

#[derive(Debug, Clone)]
pub struct Message {
    pub start: Instant,
    pub end_second: f32,
    pub content: String,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FunId {
    AvailablePort,
    OpenPort,
    ClosePort,
    ChangeId,
    Init,
    ScanDevices,

    CurrentPos,
    CurrentSpeed,
    CurrentAccSpeed,

    SetId,
    SetAngle,
    SetIncPos,
    SetSpeed,
    SetAccSpeed,
    SetAutoEnable,
    SetInitAutoBack,
    SetFactor,

    SaveParam,
}

impl ToString for FunId {
    fn to_string(&self) -> String {
        match *self {
            FunId::AvailablePort => "查询有效端口".into(),
            FunId::OpenPort => "打开端口".into(),
            FunId::ClosePort => "关闭端口".into(),
            FunId::ChangeId => "修改设备ID".into(),
            FunId::Init => "使能设备".into(),
            FunId::ScanDevices => "扫描设备".into(),
            FunId::CurrentPos => "获取当前位置".into(),
            FunId::CurrentSpeed => "获取当前速度".into(),
            FunId::CurrentAccSpeed => "获取当前加速度".into(),
            FunId::SetId => "设置ID".into(),
            FunId::SetAngle => "设置绝对位置".into(),
            FunId::SetIncPos => "设置增量位置".into(),
            FunId::SetSpeed => "设置速度".into(),
            FunId::SetAccSpeed => "设置加速度".into(),
            FunId::SetAutoEnable => "设置上电后自动启用".into(),
            FunId::SetInitAutoBack => "设置上电后回到初始位置".into(),
            FunId::SetFactor => "设置齿轮比例".into(),
            FunId::SaveParam => "保存参数".into(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Request {
    AvailablePort,
    OpenPort(String),
    ClosePort,
    ChangeId(u8),
    Init(u8),
    ScanDevices,

    CurrentPos(u8),
    CurrentSpeed(u8),
    CurrentAccSpeed(u8),

    SetId(u8),
    SetAngle(u8, f64),
    SetIncPos(u8, i32),
    SetSpeed(u8, u16),
    SetAccSpeed(u8, u16),
    SetAutoEnable(u8, bool),
    SetInitAutoBack(u8, bool),
    SetFactor(f64),

    SaveParam(u8),
}

impl Request {
    pub fn to_fun_id(&self) -> FunId {
        match *self {
            Request::AvailablePort => FunId::AvailablePort,
            Request::OpenPort(_) => FunId::OpenPort,
            Request::ClosePort => FunId::ClosePort,
            Request::ChangeId(_) => FunId::ChangeId,
            Request::Init(_) => FunId::Init,
            Request::ScanDevices => FunId::ScanDevices,
            Request::CurrentPos(_) => FunId::CurrentPos,
            Request::CurrentSpeed(_) => FunId::CurrentSpeed,
            Request::CurrentAccSpeed(_) => FunId::CurrentAccSpeed,
            Request::SetId(_) => FunId::SetId,
            Request::SetAngle(_, _) => FunId::SetAngle,
            Request::SetIncPos(_, _) => FunId::SetIncPos,
            Request::SetSpeed(_, _) => FunId::SetSpeed,
            Request::SetAccSpeed(_, _) => FunId::SetAccSpeed,
            Request::SetAutoEnable(_, _) => FunId::SetAutoEnable,
            Request::SetInitAutoBack(_, _) => FunId::SetInitAutoBack,
            Request::SetFactor(_) => FunId::SetFactor,
            Request::SaveParam(_) => FunId::SaveParam,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseCode {
    Fail = -1,   /* 执行失败 */
    Ok = 0,      /* 执行成功 */
    Ready = 1,   /* 未执行 */
    Pending = 2, /* 正在处理中 */
}

impl Default for ResponseCode {
    fn default() -> Self {
        Self::Ready
    }
}

impl ToString for ResponseCode {
    fn to_string(&self) -> String {
        match *self {
            ResponseCode::Fail => "执行失败".to_string(),
            ResponseCode::Ok => "执行成功".to_string(),
            ResponseCode::Ready => "准备就绪".to_string(),
            ResponseCode::Pending => "等待执行完成".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Response {
    pub fun_id: FunId,
    pub code: ResponseCode,
    pub msg: String,
}

impl Response {
    pub fn new(req: &Request) -> Self {
        let code = ResponseCode::default();
        Self {
            fun_id: req.to_fun_id(),
            code,
            msg: code.to_string(),
        }
    }

    pub fn pending(req: &Request) -> Self {
        let code = ResponseCode::Pending;
        Self {
            fun_id: req.to_fun_id(),
            code,
            msg: code.to_string(),
        }
    }

    pub fn operation(&self) -> String {
        self.fun_id.to_string()
    }

    pub fn status(&self) -> String {
        self.code.to_string()
    }
}

// 用于管理 Request的队列, 处理请求响应
pub struct RequestHandler {
    queue: Vec<(Request, Response)>,
    res_sender: Sender<Response>,
    res_receiver: Receiver<Response>,
    services: Vec<RcCell<Box<dyn Service>>>,
}

impl RequestHandler {
    pub fn new(res_sender: Sender<Response>, res_receiver: Receiver<Response>) -> Self {
        Self {
            queue: Vec::new(),
            res_sender,
            res_receiver,
            services: Vec::new(),
        }
    }
}

impl RequestHandler {
    /// 发起请求
    pub fn req(&mut self, req: Request) -> Result<()> {
        // 先检查队列中是否有这个请求
        let item = self.queue.iter().find(|(q, _s)| q == &req);
        if item.is_some() {
            return Err(anyhow::anyhow!("请求正在处理中"));
        }
        // 添加新的请求
        self.queue.push((req.clone(), Response::new(&req)));

        // 发送请求
        for service in self.services.iter() {
            service.as_ref().request(req.clone(), self.res_sender.clone());
        }
        Ok(())
    }

    /// 处理消息队列, 返回一个消息用于显示消息状态
    pub fn run_once(&mut self) -> Option<Response> {
        // println!("queue: {:?}", self.queue.len());
        while let Ok(res) = self.res_receiver.try_recv() {
            // 找到对应的req
            let item = self.queue.iter_mut().find(|(req, res)| res.fun_id == req.to_fun_id());
            if item.is_some() {
                let (_, item_res) = item.unwrap();
                *item_res = res;
            }
        }
        match self.queue.last() {
            Some(item) => Some(item.1.clone()),
            None => None,
        }
    }

    /// 注册服务
    pub fn register(&mut self, service: RcCell<Box<dyn Service>>) {
        self.services.push(service);
    }

    pub fn clean(&mut self, res: Response) {
        self.queue.retain(|(_q, s)| res.fun_id != s.fun_id);
    }
}

pub trait Service {
    /// 处理 请求/响应
    fn request(&self, req: Request, res_sender: Sender<Response>) -> bool {
        false
    }

    /// 发布数据
    fn publish(&self) -> Vec<Topic> {
        Vec::new()
    }
}

#[derive(Debug, PartialEq)]
pub struct TopicAddr(String);

impl TopicAddr {
    pub fn name(&self) -> String {
        self.0.clone()
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TopicId {
    MotorData,
}

#[derive(Clone)]
pub struct Topic {
    pub id: TopicId,
    pub msg: String,
}

pub struct TopicHandler {
    services: Vec<RcCell<Box<dyn Service>>>,
}

impl TopicHandler {
    pub fn new() -> Self {
        Self { services: Vec::new() }
    }

    pub fn run_once(&self) -> Vec<Topic> {
        let mut topics = Vec::new();
        for service in self.services.iter() {
            topics.extend(service.as_ref().publish());
        }
        topics
    }

    pub fn register(&mut self, service: RcCell<Box<dyn Service>>) {
        self.services.push(service);
    }
}

pub struct AppRuntime {
    // 处理请求
    req_handler: RequestHandler,
    // 处理订阅
    topic_handler: TopicHandler,

    /// 端口列表
    pub ports: Vec<String>,
    /// 扫描到 485 id列表
    pub ids: Vec<u8>,

    /// 选中的端口
    pub port: String,
    pub port_opened: bool,
    /// 选中的 485 ID
    pub id: String,

    /// 当前操作
    pub operation: Option<FunId>,
    /// 操作状态
    pub status: Option<ResponseCode>,
    /// 异常信息
    pub error_msg: Option<String>,
    /// 提示信息
    pub info_msg: Option<String>,

    /// 当前位置
    pub current_pos: i32,
    /// 当前速度
    pub current_speed: u16,
    /// 当前加速度
    pub current_acc_speed: u16,
    /// 初始位置
    pub init_pos: u16,

    /// 当前是否 上电自动启用
    pub current_auto_enable: bool,
    /// 上电后自动回到初始位置
    pub current_init_auto_back: bool,

    /// 设置绝对位置
    pub angle: f64,
    /// 设置增量位置
    pub inc_pos: i32,
    /// 设置速度
    pub speed: u16,
    /// 设置加速度
    pub acc_speed: u16,
    /// 上电后自动启用
    pub auto_enable: bool,
    /// 启用 上电后自动回到初始位置
    pub init_auto_back: bool,
    // changed_id: String,
    pub factor: f64,
}

impl Default for AppRuntime {
    fn default() -> Self {
        // 处理请求/响应, 有消息队列, 会检查每一个消息执行的状态: 准备/成功/失败/阻塞
        let (res_sender, res_receiver) = bounded::<Response>(100);
        let mut req_handler = RequestHandler::new(res_sender, res_receiver);

        // 订阅数据, 后端有数据就更新
        let (topic_pub, topic_sub) = bounded::<Topic>(100);
        let mut topic_handler = TopicHandler::new();

        // let motor_service = RcCell::new(Box::new(MotorService::new()) as Box<dyn Service>);

        // // 处理电机请求
        // req_handler.register(motor_service.clone());

        // // 处理电机的发布数据
        // topic_handler.register(motor_service);

        Self {
            req_handler,
            topic_handler,
            ports: Default::default(),
            ids: Default::default(),
            port: Default::default(),
            port_opened: Default::default(),
            id: Default::default(),
            operation: Default::default(),
            status: Default::default(),
            error_msg: Default::default(),
            info_msg: Default::default(),
            current_pos: 0,
            current_speed: 0,
            current_acc_speed: 0,
            current_auto_enable: false,
            current_init_auto_back: false,
            init_pos: 0,
            angle: 0.0,
            inc_pos: 0,
            speed: 0,
            acc_speed: 0,
            auto_enable: false,
            init_auto_back: false,

            factor: 2.04,
        }
    }
}

impl AppRuntime {
    pub fn port(&self) -> String {
        self.port.clone()
    }

    pub fn id(&self) -> u8 {
        self.id.parse().unwrap()
    }
}

impl AppRuntime {
    pub fn call(&mut self, req: Request) {
        if let Err(e) = self.req_handler.req(req) {
            let msg = format!("异常: {:?}", &e);
            log::error!("{:?}", &msg);
            self.error(msg, None);
        }
    }

    /// 一般信息提示
    pub fn info(&mut self, msg: String, operation: Option<FunId>) {
        self.error_msg = None;
        self.info_msg = Some(msg);
        self.operation = operation;
    }

    /// 错误提示
    pub fn error(&mut self, msg: String, operation: Option<FunId>) {
        self.error_msg = Some(msg);
        self.operation = operation;
    }

    /// 执行命令处理
    pub fn run_once(&mut self) {
        let res = self.req_handler.run_once();
        if let Some(res) = res {
            self.handle_request(res);
        }

        let res = self.topic_handler.run_once();
        self.handle_topic(res);
    }

    /// 处理 请求/响应
    pub fn handle_request(&mut self, res: Response) {
        match res.code {
            // 请求异常
            ResponseCode::Fail => {
                self.error(res.msg.clone(), Some(res.fun_id));
                self.req_handler.clean(res);
            }
            // 请求成功
            ResponseCode::Ok => {
                match res.fun_id {
                    FunId::AvailablePort => {
                        self.ports = serde_json::from_str(&res.msg).unwrap();
                    }
                    FunId::OpenPort => {
                        self.port_opened = true;
                    }
                    FunId::ClosePort => {
                        self.port_opened = false;
                    }
                    FunId::ScanDevices => {
                        self.ids = serde_json::from_str(&res.msg).unwrap();
                    }
                    FunId::ChangeId => {}
                    FunId::Init => {}
                    FunId::CurrentPos => {
                        self.current_pos = serde_json::from_str(&res.msg).unwrap();
                    }
                    FunId::CurrentSpeed => {
                        self.current_speed = serde_json::from_str(&res.msg).unwrap();
                    }
                    FunId::CurrentAccSpeed => {
                        self.current_acc_speed = serde_json::from_str(&res.msg).unwrap();
                    }
                    FunId::SetId | FunId::SetAngle | FunId::SetIncPos | FunId::SetSpeed | FunId::SetAccSpeed => {}
                    FunId::SetAutoEnable => {}
                    FunId::SetInitAutoBack => {}
                    FunId::SetFactor => {}
                    FunId::SaveParam => {}
                }
                self.info(res.status(), Some(res.fun_id));
                self.req_handler.clean(res);
            }
            // 请求 正在处理中
            ResponseCode::Ready | ResponseCode::Pending => {
                self.info(res.msg.clone(), Some(res.fun_id));
            }
        }
    }

    /// 处理 发布消息
    pub fn handle_topic(&mut self, topics: Vec<Topic>) {
        for topic in topics {
            match topic.id {
                TopicId::MotorData => {
                    let data: MotorData = serde_json::from_str(&topic.msg).unwrap();
                    self.current_pos = data.pos;
                    self.current_speed = data.speed;
                    self.current_acc_speed = data.acc_speed;
                    self.init_pos = data.init_pos;
                }
            }
        }
    }
}

impl AppRuntime {
    pub fn status_bar(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // 当前操作
            if let Some(operation) = self.operation.as_ref() {
                let operation = WidgetText::RichText(RichText::new(operation.to_string()).color(Color32::RED).size(13.0));
                ui.add(Label::new(operation));
            }

            // 有错误信息
            if let Some(error_msg) = &self.error_msg {
                let error_msg = WidgetText::RichText(RichText::new(error_msg.to_string()).color(Color32::RED).size(13.0));
                ui.add(Label::new(error_msg));
            }
            // 如果没有错误信息, 就显示 info 信息
            else if let Some(info_msg) = &self.info_msg {
                let info_msg = WidgetText::RichText(RichText::new(info_msg.to_string()).color(Color32::RED).size(13.0));
                ui.add(Label::new(info_msg));
            }
            // 如果没有错误信息, 没有 info 信息, 就显示操作过程
            else if let Some(status) = self.status.as_ref() {
                let status = WidgetText::RichText(RichText::new(status.to_string()).color(Color32::RED).size(13.0));
                ui.add(Label::new(status));
            }
        });
    }

    pub fn ids_ui(&mut self, ui: &mut Ui) {
        ComboBox::from_id_source("select_ids")
            .selected_text(self.id.to_string())
            .show_ui(ui, |ui| {
                for id in self.ids.clone() {
                    let changed = ui.selectable_value(&mut self.id, id.to_string(), id.to_string()).changed();
                    if changed {
                        self.call(Request::SetId(id));
                    }
                }
            });
    }

    pub fn ports_ui(&mut self, ui: &mut Ui) {
        if self.port == "" && self.ports.len() != 0 {
            self.port = self.ports.last().unwrap().clone();
        }
        ComboBox::from_id_source("select_ports")
            .selected_text(self.port.clone())
            .show_ui(ui, |ui| {
                for port in &self.ports {
                    ui.selectable_value(&mut self.port, port.clone(), port.clone());
                }
            });
    }
}
