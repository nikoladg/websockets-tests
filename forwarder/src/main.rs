#![allow(unused_variables)]
#![feature(generators)]
extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate futures_await as futures;

use actix::prelude::*;
use actix_web::{ws, middleware, server, App, Error, HttpRequest, HttpResponse};
use actix_web::http::header;
use futures::prelude::{async, await};
use futures::Future;

mod utils;

fn ws_index(req: HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>> {
    println!("{:?}", req);
    Box::new(
        Forwarder::with_request(req.clone()).and_then(move |actor| {
            utils::websockets::start(&req, actor, |stream| stream.max_size(10 * (1 << 20)))
        }),
    )
}

struct Forwarder {
    reader: ws::ClientReader,
    writer: ws::ClientWriter,
}

impl Actor for Forwarder {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Forwarder Actor started");
    }
}

impl StreamHandler<ws::Message, ws::ProtocolError> for Forwarder {
    fn handle(&mut self, msg: ws::Message, ctx: &mut Self::Context) {
        println!("ws::Message received: {:?}", msg);
        match msg {
            ws::Message::Text(text) => self.writer.text(text),
            ws::Message::Binary(bin) => self.writer.binary(bin),
            ws::Message::Ping(msg) => self.writer.ping(&msg),
            ws::Message::Pong(msg) => self.writer.pong(&msg),
            ws::Message::Close(reason) => self.writer.close(reason),
        }
    }
}

impl Forwarder {
    #[async]
    pub fn with_request(req: HttpRequest) -> Result<Self, Error> {
        let fut_reader_writer = await!({
            let mut client = ws::Client::new("ws://localhost:8080/wsecho/");
            client = client.header(header::AUTHORIZATION, "Basic bmlrb2xhQGRlZXBncmFtLmNvbTpwd2Q=".to_string()); // temporary
            client.connect().map_err(|e| {
                println!("Error: {}", e);
                ()
            })
        });

        let (reader, writer) = fut_reader_writer.unwrap(); // will panic if unable to get the reader and writer

        let result = Self {
            reader,
            writer,
        };

        Ok(result)
    }
}

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=trace");
    env_logger::init();
    let sys = actix::System::new("wsforwarder");

    server::new(
        || App::new()
            .middleware(middleware::Logger::default())
            .resource("/wsforwarder/", move |r| {
                r.with(move |req| ws_index(req))
            }))
        .bind("127.0.0.1:8081").unwrap()
        .start();

    let _ = sys.run();
}
