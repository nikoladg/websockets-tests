#![allow(unused_variables)]
extern crate actix;
extern crate actix_web;
extern crate env_logger;
extern crate futures_await as futures;

use std::{io, thread};
use actix::*;
use actix_web::ws::{Client, ClientWriter, Message, ProtocolError};
use actix_web::http::header;
use futures::Future;

fn main() {
    ::std::env::set_var("RUST_LOG", "actix_web=trace");
    let _ = env_logger::init();
    let sys = actix::System::new("wssender");

    Arbiter::spawn(
        Client::new("ws://127.0.0.1:8081/wsforwarder/")
            .header(header::AUTHORIZATION, "Basic bmlrb2xhQGRlZXBncmFtLmNvbTpwd2Q=".to_string())
            .connect()
            .map_err(|e| {
                println!("Error: {}", e);
                ()
            })
            .map(|(reader, writer)| {
                let addr = Sender::create(|ctx| {
                    Sender::add_stream(reader, ctx);
                    Sender(writer)
                });

                thread::spawn(move || loop {
                    let mut cmd = String::new();
                    if io::stdin().read_line(&mut cmd).is_err() {
                        println!("error");
                        return;
                    }
                    addr.do_send(StdinCommand(cmd));
                });

                ()
            }),
    );

    let _ = sys.run();
}

struct Sender(ClientWriter);

#[derive(Message)]
struct StdinCommand(String);

impl Actor for Sender {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Context<Self>) {
        println!("Sender Actor started");
    }

    fn stopped(&mut self, _: &mut Context<Self>) {
        println!("Sender Actor Disconnected");
        System::current().stop();
    }
}

impl Sender {
}

impl Handler<StdinCommand> for Sender {
    type Result = ();

    fn handle(&mut self, msg: StdinCommand, ctx: &mut Context<Self>) {
        self.0.text(msg.0) // self.0 is a ClientWriter
    }
}

impl StreamHandler<Message, ProtocolError> for Sender {
    fn handle(&mut self, msg: Message, ctx: &mut Context<Self>) {
        println!("ws::Message received: {:?}", msg);
        match msg {
            _ => (),
        }
    }

    fn started(&mut self, ctx: &mut Context<Self>) {
        println!("Sender StreamHandler starts");
    }

    fn finished(&mut self, ctx: &mut Context<Self>) {
        println!("Sender StreamHandler disconnected");
        ctx.stop()
    }
}
