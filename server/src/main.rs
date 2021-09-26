use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use bevy::prelude::*;

use message_io::{
    network::{NetEvent, Transport},
    node,
};

use shared::NetworkingAction;

pub struct NetStruct<T: Send + 'static> {
    pub handler: node::NodeHandler<T>,
    //pub listener: node::NodeListener<T>,
    pub is_crashed: Arc<AtomicBool>,
}

type Net = NetStruct<()>;

fn server() -> Net {
    let (handler, listener) = node::split::<()>();

    handler
        .network()
        .listen(Transport::Tcp, "0.0.0.0:7777")
        .unwrap();
    handler
        .network()
        .listen(Transport::Udp, "0.0.0.0:7777")
        .unwrap();

    let listen_handle = handler.clone();
    let is_crashed = Arc::new(AtomicBool::new(false));
    let listen_is_crashed = is_crashed.clone();

    std::thread::spawn(move || {
        info!("Starting server");

        let handler = listen_handle;
        listener.for_each(|event| match event.network() {
            NetEvent::Connected(_, _) => println!("connected"),
            NetEvent::Message(endpoint, data) => {
                let j = match std::str::from_utf8(data) {
                    Ok(d) => d,
                    Err(_) => {
                        info!("someone sent invalid utf8");
                        return;
                    }
                };

                let _action: NetworkingAction = match serde_json::from_str(j) {
                    Ok(packet) => packet,
                    Err(_) => {
                        info!("someone sent an unknown packet: {}", j);
                        return;
                    }
                };



                handler.network().send(endpoint, "ack".as_bytes());
            }
            NetEvent::Disconnected(_) => println!("disconnected"),
        });

        error!("Server crashed...");
        listen_is_crashed.store(true, Ordering::Relaxed);
    });

    Net {
        handler,
        is_crashed,
    }
}

struct TestECS;

fn startup(mut commands: Commands) {
    commands.spawn().insert(TestECS);
}

fn add_networking(app: &mut AppBuilder) {
    app.insert_resource(server())
        .add_startup_system(startup.system());
    //.add_system(check_server.system());
}

fn main() {
    //server();

    let mut app = App::build();

    app.add_plugin(bevy::log::LogPlugin)
        .add_plugin(bevy::core::CorePlugin)
        .add_plugin(bevy::transform::TransformPlugin)
        .add_plugin(bevy::diagnostic::DiagnosticsPlugin)
        .add_plugin(bevy::app::ScheduleRunnerPlugin::default());


    add_networking(&mut app);

    app.run();
}
