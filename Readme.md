# AppRuntime 的设计

在 AppRuntime 中实现 Request/Response 和 Message Publish/Subject 和 弹出框管理


```rust
pub struct RequestHandler {
    queue: Vec<(Request, Response)>,
    res_sender: Sender<Response>,
    res_receiver: Receiver<Response>,
    services: Vec<RcCell<Box<dyn Service>>>,
}
```

想实现下 http的web server/client 这种模式, 用在 egui上,  用 channel 充当 http的数据通道, egui端 通过channel 发起请求: url + 序列化数据, 

在 channel的接收端 我会注册服务, 用url + 注册函数(跟web服务中一样的, 使用不同的参数), 接收端解析 请求, 并把数据传递给服务函数

接受请求, 并把请求发到独立的 thread 中






有没有高效地二进制序列化的库呢? 想实现: 类似http的web server/client 这种模式, 用 channel的发送端发送序列化的数据, 在接收端, 把数据

RequestHandler 用于处理 Request 的发起, 以及在 ui 更新时 查询 Response 状态. 它 维护了 一个 queue, 用于保存每一个请求 和 响应的结果.

当执行 fn req(&mut self, req: Request) 时, 如果请求不存在queue中, 则会把当前请求发送出去, 设置 Response状态为 Ready.

RequestServer {

}