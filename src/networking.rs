#![allow(dead_code, unused_variables, unused_mut)]
use bevy::prelude::*;
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
    let host_mode = config.host_mode;
    let (inc, out) = (netqueues.incoming.clone(), netqueues.outgoing.clone());
    let jh = thread::spawn(move || {
        if host_mode {
            start_host(inc, out)
        } else {
            start_player(inc, out)
        }
    });

    let player_mesh = assets_server.load("cube.gltf#Mesh0/Primitive0");

    commands
        .spawn(PbrComponents {
            mesh: player_mesh,
            material: materials.add(Color::GREEN.into()),
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .with(NetworkEnt);
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

fn start_host(inc: NetworkQueue, out: NetworkQueue) {
    thread::spawn(move || {
        let reader = TcpListener::bind("0.0.0.0:8878").expect("couldn't start server D:");
        for stream in reader.incoming() {
            let mut person = stream.unwrap();
            person.set_read_timeout(None).unwrap();

            let inc = inc.clone();
            thread::spawn(move || handle_incoming(person, inc));
        }
    });

    thread::spawn(move || {
        let writer = TcpListener::bind("0.0.0.0:8879").expect("couldn't start server D:");
        for stream in writer.incoming() {
            let mut person = stream.unwrap();
            person.set_write_timeout(None).unwrap();

            let out = out.clone();
            thread::spawn(move || handle_outgoing(person, out));
        }
    });
}

fn start_player(inc: NetworkQueue, out: NetworkQueue) {
    let mut stream_in = TcpStream::connect("john2143.com:8879").unwrap();
    let mut stream_out = TcpStream::connect("john2143.com:8878").unwrap();

    thread::spawn(move || {
        handle_outgoing(stream_out, out);
    });
    thread::spawn(move || {
        handle_incoming(stream_in, inc);
    });
}

struct NetworkingTimer(Timer);

fn system_update_networking(
    mut nets: ResMut<NetworkingQueues>,
    mut timer: ResMut<NetworkingTimer>,
    time: Res<Time>,
    config: Res<crate::config::Config>,
    mut player_query: Query<(&crate::CameraOrientation, &Transform)>,
    mut net_player_query: Query<(&NetworkEnt, &mut Transform)>,
) {
    //prevent flooding of the out queue
    timer.0.tick(time.delta_seconds);

    if !timer.0.just_finished {
        return;
    }

    for (_, mut transform) in player_query.iter() {
        if let Ok(mut out) = nets.outgoing.lock() {
            if out.len() < 3 {
                out.push(NetworkingAction::Location(
                    transform.rotation,
                    transform.translation,
                ));
            }
        }
    }

    for (_, mut transform) in net_player_query.iter_mut() {
        let ins = std::mem::replace(&mut *nets.incoming.lock().unwrap(), Vec::new());
        for item in ins.iter() {
            match item {
                NetworkingAction::Print(s) => {
                    println!("net says {}", s);
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
        .add_resource(NetworkingTimer(Timer::from_seconds(1.0 / 120.0, true)))
        .add_startup_system(setup_networking.system())
        .add_system(system_update_networking.system());
}
