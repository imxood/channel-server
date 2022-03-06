use std::time::Instant;

use channel_server::prelude::*;

#[derive(Debug, Serialize, Deserialize)]
struct User {
    name: String,
}

#[handler]
fn hello(name: String) -> String {
    let res = format!("hello: {}", name);
    println!("\t{}", &res);
    res
}

#[handler]
fn hello_json(user: ReqParam<User>, json: Json<String>, data: Data<&i32>) -> String {
    let res = format!("user: {:?}, json: {}, data: {}", user, json.0, data.0);
    println!("\t{}", &res);
    res
}

fn main() -> Result<(), ChannelError> {
    let ep = Route::new()
        .at("hello", hello.boxed())
        .at("hello_json", hello_json.boxed())
        .data(1);

    let (mut client, server) = ChannelService::new().split();

    // 服务端
    std::thread::spawn(move || {
        server.run(ep);
    });

    println!("执行开始");
    let start_time = Instant::now();

    // 客户端
    let uri1 = "hello".to_string();
    let request = Request::with_body(
        uri1.clone(),
        Body::from_string("this is a string".to_string()),
    );
    client.req(request)?;

    let uri2 = "hello_json".to_string();
    let request = Request::new(
        uri2.clone(),
        Param::from_obj(User {
            name: "maxu".to_string(),
        }),
        Body::from_string(serde_json::to_string("this is a json string").unwrap()),
    );
    client.req(request)?;

    loop {
        let mut res1_ok = false;
        let mut res2_ok = true;
        // 运行队列, 接收 response
        if client.run_once() {
            // 查询执行结果
            if let Some(res) = client.search(&uri1) {
                println!("{:?}", res);
                // 如果执行成功, 清除响应
                if res.is_ok() {
                    client.clean(&uri1);
                    res1_ok = true;
                }
            }

            if let Some(res) = client.search(&uri2) {
                println!("{:?}", res);
                // 如果执行成功, 清除响应
                if res.is_ok() {
                    client.clean(&uri2);
                    res2_ok = true;
                }
            }

            if res1_ok && res2_ok {
                println!("执行完成");
                break;
            }
        }
        std::thread::yield_now();
    }

    println!("执行时间: {:?}", start_time.elapsed());

    Ok(())
}
