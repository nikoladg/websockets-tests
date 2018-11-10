#![allow(unused_variables)]
#![feature(generators)]
extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate futures_await as futures;

use std::time::{Instant, Duration};
use actix::prelude::*;
use actix_web::{ws, middleware, server, App, Error, HttpRequest, HttpResponse};
use actix_web::http::header;
use futures::prelude::{async, await};
use futures::Future;

mod utils;

fn ws_index(req: HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    println!("{:?}", req);
    Box::new(
        WSEcho::with_request(req.clone()).and_then(move |actor| {
            utils::websockets::start(&req, actor, |stream| stream.max_size(10 * (1 << 20)))
        }),
    )
}

struct WSEcho {
}

impl Actor for WSEcho {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("WSEcho Actor started");
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for WSEcho {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        println!("ws::Message received: {:?}", msg);
        match msg {
            ws::Message::Binary(bin) => ctx.binary(bin),
            ws::Message::Text(text) => ctx.text(text),
            ws::Message::Ping(msg) => ctx.ping(&msg),
            ws::Message::Pong(msg) => ctx.pong(&msg),
            ws::Message::Close(reason) => ctx.close(reason),
        }
    }
}

impl WSEcho {
    #[async]
    pub fn with_request(req: HttpRequest) -> Result<Self, Error> {
        let result = Self {};
        Ok(result)
    }
}

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=trace");
    env_logger::init();
    let sys = actix::System::new("ws-echo");

    server::new(
        || App::new()
            .middleware(middleware::Logger::default())
            .resource("/wsecho/", move |r| {
                r.with(move |req| ws_index(req))
            }))
        .bind("127.0.0.1:8080").unwrap()
        .start();

    let _ = sys.run();
}
