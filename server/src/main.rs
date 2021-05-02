use std::time::Duration;

use message_io::{
    network::{NetEvent, Transport},
    node::{self, NodeEvent},
};

fn server() {
    let (handler, listener) = node::split::<()>();

    handler
        .network()
        .listen(Transport::Tcp, "0.0.0.0:7777")
        .unwrap();
    handler
        .network()
        .listen(Transport::Udp, "0.0.0.0:7777")
        .unwrap();

    listener.for_each(move |event| {
        match event.network() {
            NetEvent::Connected(_, _) => println!("connected"),
            NetEvent::Message(endpoint, data) => {
                println!("Received: {:?}", std::str::from_utf8(data));
                handler.network().send(endpoint, "".as_bytes());
            }
            NetEvent::Disconnected(_) => println!("disconnected"),
        }
    });
}

fn client() {
    let (handler, listener) = node::split::<()>();

    let (server, _) = handler
        .network()
        .connect(Transport::Udp, "127.0.0.1:7777")
        .unwrap();

    handler.network().send(server, "hello".as_bytes());
    let h2 = handler.clone();

    let mut i = 0;
    std::thread::spawn(move || {
        loop {
            i += 1;
            h2.network().send(server, format!("ok {}", i).as_bytes());
            std::thread::sleep(Duration::from_millis(10));
        }
    });

    listener.for_each(move |event| {
        match event {
            NodeEvent::Signal(_s) => {}
            NodeEvent::Network(net_event) => match net_event {
                NetEvent::Message(endpoint, _data) => {
                    //i += 1;
                    //handler.network().send(server, &['b' as u8; 1200]);
                    println!("got some data", );
                }
                NetEvent::Disconnected(_) => {
                    println!("disconnected from server",);
                }
                _ => {}
            },
        }
    });
}

fn main() {
    let a = std::env::args().nth(1);
    let a = match a {
        Some(a) => a,
        None => return println!("start with either client or server arg"),
    };

    if a == "server" {
        server();
    }
    if a == "client" {
        client();
    } else {
        println!("'{}' not recognized as a valid word", a);
    }
}
