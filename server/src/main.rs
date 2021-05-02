use message_io::{
    network::{NetEvent, Transport},
    node,
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
                //println!("Received: {:?}", std::str::from_utf8(data));
                handler.network().send(endpoint, "".as_bytes());
            }
            NetEvent::Disconnected(_) => println!("disconnected"),
        }
    });
}

fn main() {
    server();
}
