#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2021::*;
#[macro_use]
extern crate std;
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};
use ahash::AHashMap;
use channel_server::{
    data::Data,
    request::{json::Json, param},
    Body, ChannelService, Endpoint, EndpointExt, Error, FromRequest, IntoResponse, Param, Request,
    Response, Route, handler,
};
use serde::{Deserialize, Serialize};
struct hello;
impl Endpoint for hello {
    type Output = Response;
    fn call(&self, req: Request) -> Result<Self::Output, Error> {
        let (req, mut body) = req.split();
        let p0 = <String as FromRequest>::from_request(&req, &mut body)?;
        fn hello(name: String) -> String {
            let res = {
                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                    &["hello: "],
                    &[::core::fmt::ArgumentV1::new_display(&name)],
                ));
                res
            };
            {
                ::std::io::_print(::core::fmt::Arguments::new_v1(
                    &["\t", "\n"],
                    &[::core::fmt::ArgumentV1::new_display(&&res)],
                ));
            };
            res
        }
        let res = hello(p0);
        Ok(res.into_response())
    }
}
struct User {
    name: String,
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::default::Default for User {
    #[inline]
    fn default() -> User {
        User {
            name: ::core::default::Default::default(),
        }
    }
}
#[automatically_derived]
#[allow(unused_qualifications)]
impl ::core::fmt::Debug for User {
    fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
        match *self {
            User {
                name: ref __self_0_0,
            } => {
                let debug_trait_builder = &mut ::core::fmt::Formatter::debug_struct(f, "User");
                let _ =
                    ::core::fmt::DebugStruct::field(debug_trait_builder, "name", &&(*__self_0_0));
                ::core::fmt::DebugStruct::finish(debug_trait_builder)
            }
        }
    }
}
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl _serde::Serialize for User {
        fn serialize<__S>(
            &self,
            __serializer: __S,
        ) -> _serde::__private::Result<__S::Ok, __S::Error>
        where
            __S: _serde::Serializer,
        {
            let mut __serde_state = match _serde::Serializer::serialize_struct(
                __serializer,
                "User",
                false as usize + 1,
            ) {
                _serde::__private::Ok(__val) => __val,
                _serde::__private::Err(__err) => {
                    return _serde::__private::Err(__err);
                }
            };
            match _serde::ser::SerializeStruct::serialize_field(
                &mut __serde_state,
                "name",
                &self.name,
            ) {
                _serde::__private::Ok(__val) => __val,
                _serde::__private::Err(__err) => {
                    return _serde::__private::Err(__err);
                }
            };
            _serde::ser::SerializeStruct::end(__serde_state)
        }
    }
};
#[doc(hidden)]
#[allow(non_upper_case_globals, unused_attributes, unused_qualifications)]
const _: () = {
    #[allow(unused_extern_crates, clippy::useless_attribute)]
    extern crate serde as _serde;
    #[automatically_derived]
    impl<'de> _serde::Deserialize<'de> for User {
        fn deserialize<__D>(__deserializer: __D) -> _serde::__private::Result<Self, __D::Error>
        where
            __D: _serde::Deserializer<'de>,
        {
            #[allow(non_camel_case_types)]
            enum __Field {
                __field0,
                __ignore,
            }
            struct __FieldVisitor;
            impl<'de> _serde::de::Visitor<'de> for __FieldVisitor {
                type Value = __Field;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "field identifier")
                }
                fn visit_u64<__E>(self, __value: u64) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        0u64 => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_str<__E>(
                    self,
                    __value: &str,
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        "name" => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
                fn visit_bytes<__E>(
                    self,
                    __value: &[u8],
                ) -> _serde::__private::Result<Self::Value, __E>
                where
                    __E: _serde::de::Error,
                {
                    match __value {
                        b"name" => _serde::__private::Ok(__Field::__field0),
                        _ => _serde::__private::Ok(__Field::__ignore),
                    }
                }
            }
            impl<'de> _serde::Deserialize<'de> for __Field {
                #[inline]
                fn deserialize<__D>(
                    __deserializer: __D,
                ) -> _serde::__private::Result<Self, __D::Error>
                where
                    __D: _serde::Deserializer<'de>,
                {
                    _serde::Deserializer::deserialize_identifier(__deserializer, __FieldVisitor)
                }
            }
            struct __Visitor<'de> {
                marker: _serde::__private::PhantomData<User>,
                lifetime: _serde::__private::PhantomData<&'de ()>,
            }
            impl<'de> _serde::de::Visitor<'de> for __Visitor<'de> {
                type Value = User;
                fn expecting(
                    &self,
                    __formatter: &mut _serde::__private::Formatter,
                ) -> _serde::__private::fmt::Result {
                    _serde::__private::Formatter::write_str(__formatter, "struct User")
                }
                #[inline]
                fn visit_seq<__A>(
                    self,
                    mut __seq: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::SeqAccess<'de>,
                {
                    let __field0 =
                        match match _serde::de::SeqAccess::next_element::<String>(&mut __seq) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        } {
                            _serde::__private::Some(__value) => __value,
                            _serde::__private::None => {
                                return _serde::__private::Err(_serde::de::Error::invalid_length(
                                    0usize,
                                    &"struct User with 1 element",
                                ));
                            }
                        };
                    _serde::__private::Ok(User { name: __field0 })
                }
                #[inline]
                fn visit_map<__A>(
                    self,
                    mut __map: __A,
                ) -> _serde::__private::Result<Self::Value, __A::Error>
                where
                    __A: _serde::de::MapAccess<'de>,
                {
                    let mut __field0: _serde::__private::Option<String> = _serde::__private::None;
                    while let _serde::__private::Some(__key) =
                        match _serde::de::MapAccess::next_key::<__Field>(&mut __map) {
                            _serde::__private::Ok(__val) => __val,
                            _serde::__private::Err(__err) => {
                                return _serde::__private::Err(__err);
                            }
                        }
                    {
                        match __key {
                            __Field::__field0 => {
                                if _serde::__private::Option::is_some(&__field0) {
                                    return _serde::__private::Err(
                                        <__A::Error as _serde::de::Error>::duplicate_field("name"),
                                    );
                                }
                                __field0 = _serde::__private::Some(
                                    match _serde::de::MapAccess::next_value::<String>(&mut __map) {
                                        _serde::__private::Ok(__val) => __val,
                                        _serde::__private::Err(__err) => {
                                            return _serde::__private::Err(__err);
                                        }
                                    },
                                );
                            }
                            _ => {
                                let _ = match _serde::de::MapAccess::next_value::<
                                    _serde::de::IgnoredAny,
                                >(&mut __map)
                                {
                                    _serde::__private::Ok(__val) => __val,
                                    _serde::__private::Err(__err) => {
                                        return _serde::__private::Err(__err);
                                    }
                                };
                            }
                        }
                    }
                    let __field0 = match __field0 {
                        _serde::__private::Some(__field0) => __field0,
                        _serde::__private::None => {
                            match _serde::__private::de::missing_field("name") {
                                _serde::__private::Ok(__val) => __val,
                                _serde::__private::Err(__err) => {
                                    return _serde::__private::Err(__err);
                                }
                            }
                        }
                    };
                    _serde::__private::Ok(User { name: __field0 })
                }
            }
            const FIELDS: &'static [&'static str] = &["name"];
            _serde::Deserializer::deserialize_struct(
                __deserializer,
                "User",
                FIELDS,
                __Visitor {
                    marker: _serde::__private::PhantomData::<User>,
                    lifetime: _serde::__private::PhantomData,
                },
            )
        }
    }
};
#[allow(non_camel_case_types)]
struct hello_json;
impl channel_server::Endpoint for hello_json {
    type Output = channel_server::Response;
    #[allow(unused_mut)]
    fn call(
        &self,
        mut req: channel_server::Request,
    ) -> Result<Self::Output, channel_server::Error> {
        let (req, mut body) = req.split();
        let p0 =
            <param::Param<User> as channel_server::FromRequest>::from_request(&req, &mut body)?;
        let p1 = <Json<String> as channel_server::FromRequest>::from_request(&req, &mut body)?;
        let p2 = <Data<&i32> as channel_server::FromRequest>::from_request(&req, &mut body)?;
        fn hello_json(user: param::Param<User>, json: Json<String>, data: Data<&i32>) -> String {
            let res = {
                let res = ::alloc::fmt::format(::core::fmt::Arguments::new_v1(
                    &["user: ", ", json: ", ", data: "],
                    &[
                        ::core::fmt::ArgumentV1::new_debug(&user),
                        ::core::fmt::ArgumentV1::new_display(&json.0),
                        ::core::fmt::ArgumentV1::new_display(&data.0),
                    ],
                ));
                res
            };
            {
                ::std::io::_print(::core::fmt::Arguments::new_v1(
                    &["\t", "\n"],
                    &[::core::fmt::ArgumentV1::new_display(&&res)],
                ));
            };
            res
        }
        let res = hello_json(p0, p1, p2);
        let res = channel_server::error::IntoResult::into_result(res);
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
    let request = Request::with_body(
        "hello".into(),
        Body::from_string("this is a string".to_string()),
    );
    let res = ep.read().unwrap().get_response(request);
    {
        ::std::io::_print(::core::fmt::Arguments::new_v1(
            &["res: ", "\n"],
            &[::core::fmt::ArgumentV1::new_debug(&res)],
        ));
    };
    let request = Request::new(
        "hello_json".into(),
        Param::from_obj(User {
            name: "maxu".to_string(),
        }),
        Body::from_string(serde_json::to_string("this is a json string").unwrap()),
    );
    let res = ep.read().unwrap().get_response(request);
    {
        ::std::io::_print(::core::fmt::Arguments::new_v1(
            &["res: ", "\n"],
            &[::core::fmt::ArgumentV1::new_debug(&res)],
        ));
    };
}
fn main() -> Result<(), Error> {
    let ep = Arc::new(RwLock::new(
        Route::default()
            .at("hello", hello.boxed())
            .at("hello_json", hello_json.boxed())
            .data(1),
    ));
    let (mut client, server) = ChannelService::new().split();
    let ep = ep.clone();
    std::thread::spawn(move || {
        let ep = ep.clone();
        server.run(ep);
    });
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
        if client.run_once() {
            if let Some(res) = client.search(&uri1) {
                {
                    ::std::io::_print(::core::fmt::Arguments::new_v1(
                        &["", "\n"],
                        &[::core::fmt::ArgumentV1::new_debug(&res)],
                    ));
                };
                if res.is_ok() {
                    client.clean(&uri1);
                    res1_ok = true;
                }
            }
            if let Some(res) = client.search(&uri2) {
                {
                    ::std::io::_print(::core::fmt::Arguments::new_v1(
                        &["", "\n"],
                        &[::core::fmt::ArgumentV1::new_debug(&res)],
                    ));
                };
                if res.is_ok() {
                    client.clean(&uri2);
                    res2_ok = true;
                }
            }
            if res1_ok && res2_ok {
                {
                    ::std::io::_print(::core::fmt::Arguments::new_v1(
                        &["\u{6267}\u{884c}\u{5b8c}\u{6210}\n"],
                        &[],
                    ));
                };
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    Ok(())
}
