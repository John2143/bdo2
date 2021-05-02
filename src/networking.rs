#![allow(dead_code, unused_variables, unused_mut)]
use bevy::prelude::*;
use message_io::{network::{NetEvent, Transport}, node::{self, NodeEvent}};
use std::thread;
use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
    sync::Mutex,
    time::Duration,
};

use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Clone, Debug)]
enum NetworkingAction {
    Print(String),
    Location(Quat, Vec3),
    Done,
}

type NetworkQueue = Arc<Mutex<Vec<NetworkingAction>>>;

#[derive(Default)]
struct NetworkingQueues {
    setup: bool,
    incoming: NetworkQueue,
    outgoing: NetworkQueue,
}

struct NetworkEnt;

fn setup_networking(
    mut commands: Commands,
    mut netqueues: ResMut<NetworkingQueues>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    config: Res<crate::config::Config>,
    assets_server: Res<AssetServer>,
) {
    let (inc, out) = (netqueues.incoming.clone(), netqueues.outgoing.clone());
    let jh = thread::spawn(move || {
        start_player(inc, out)
    });
}

fn handle_incoming(mut person: TcpStream, inc: NetworkQueue) {
    loop {
        let mut buf = [0; 10000];
        let read = person.read(&mut buf).unwrap();

        if read == 0 {
            thread::sleep(Duration::new(0, 100_000));
            continue;
        }

        let msg = String::from_utf8_lossy(&buf[..read]);

        for json in msg.split("\n") {
            if json.len() == 0 {
                break;
            }

            info!("{}", &json);

            let k: NetworkingAction = serde_json::from_str(&json).unwrap();

            inc.lock().unwrap().push(k);
        }
    }
}

fn handle_outgoing(mut stream: TcpStream, out: NetworkQueue) {
    loop {
        //empty the outs queue because we're using it now
        let outs = std::mem::replace(&mut *out.lock().unwrap(), Vec::new());

        for action in outs {
            let message = serde_json::to_vec(&action).unwrap();

            stream.write(&message).unwrap();
            stream.write("\n".as_bytes()).unwrap();
        }
    }
}

fn start_player(inc: NetworkQueue, out: NetworkQueue) {
    info!("starting player");
    let (handler, listener) = node::split::<()>();

    let (server, _) = handler
        .network()
        .connect(Transport::Udp, "172.18.97.249:7777")
        .unwrap();

    info!("probably connected");

    handler.network().send(server, "hello".as_bytes());
    let h2 = handler.clone();

    let mut i = 0;
    std::thread::spawn(move || {
        loop {
            h2.signals().send(());
            info!("sending");
            i += 1;
            h2.network().send(server, format!("heartbeat {}", i).as_bytes());
            std::thread::sleep(Duration::from_millis(100));
        }
    });

    listener.for_each(move |event| {
        match event {
            NodeEvent::Signal(_s) => {
                info!("signal...");
            },
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

struct NetworkingTimer(Timer);

fn system_update_networking(
    mut nets: ResMut<NetworkingQueues>,
    mut timer: ResMut<NetworkingTimer>,
    time: Res<Time>,
    config: Res<crate::config::Config>,
    mut player_query: QuerySet<(
        Query<(&crate::CameraOrientation, &Transform)>,
        Query<(&NetworkEnt, &mut Transform)>,
    )>,
) {
    if !nets.setup {
        return;
    }
    //prevent flooding of the out queue
    timer.0.tick(time.delta());

    if !timer.0.just_finished() {
        return;
    }

    for (_, mut transform) in player_query.q0().iter() {
        if let Ok(mut out) = nets.outgoing.lock() {
            if out.len() < 3 {
                out.push(NetworkingAction::Location(
                    transform.rotation,
                    transform.translation,
                ));
            }
        }
    }

    for (_, mut transform) in player_query.q1_mut().iter_mut() {
        let ins = std::mem::replace(&mut *nets.incoming.lock().unwrap(), Vec::new());
        for item in ins.iter() {
            match item {
                NetworkingAction::Print(s) => {
                    info!("net says {}", s);
                }
                NetworkingAction::Location(rot, tran) => {
                    transform.rotation = *rot;
                    transform.translation = *tran;
                }
                NetworkingAction::Done => {}
            }
        }
    }
}

pub fn build(app: &mut AppBuilder) {
    app.init_resource::<NetworkingQueues>()
        .insert_resource(NetworkingTimer(Timer::from_seconds(1.0 / 120.0, true)))
        .add_startup_system(setup_networking.system())
        .add_system(system_update_networking.system());
}
