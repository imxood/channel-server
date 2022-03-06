use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use ahash::AHashMap;
use channel_server::{
    data::Data,
    request::{json::Json, param},
    Body, ChannelService, Endpoint, EndpointExt, Error, FromRequest, IntoResponse, Param, Request,
    Response, Route,
};
use serde::{Deserialize, Serialize};

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

fn basic() {
    let ep = Arc::new(RwLock::new(
        Route::default()
            .at("hello", hello.boxed())
            .at("hello_json", hello_json.boxed())
            .data(1),
    ));

    // 测试1
    let request = Request::with_body(
        "hello".into(),
        Body::from_string("this is a string".to_string()),
    );
    let res = ep.read().unwrap().get_response(request);
    println!("res: {:?}", res);

    // 测试2
    let request = Request::new(
        "hello_json".into(),
        Param::from_obj(User {
            name: "maxu".to_string(),
        }),
        Body::from_string(serde_json::to_string("this is a json string").unwrap()),
    );
    let res = ep.read().unwrap().get_response(request);
    println!("res: {:?}", res);
}

fn main() -> Result<(), Error> {
    let ep = Arc::new(RwLock::new(
        Route::default()
            .at("hello", hello.boxed())
            .at("hello_json", hello_json.boxed())
            .data(1),
    ));

    let (mut client, server) = ChannelService::new().split();

    // 服务端
    let ep = ep.clone();
    std::thread::spawn(move || {
        let ep = ep.clone();
        server.run(ep);
    });

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

        std::thread::sleep(Duration::from_millis(50));
    }

    Ok(())
}
