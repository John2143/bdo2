use bevy::prelude::*;
use rand::{random, thread_rng, Rng};

//use crate::{config::Config, input::InputEvent};

pub struct Enemy(f32, f32);

fn update(
    mut commands: Commands,
    time: Res<Time>,

    mut spawn_timer: ResMut<EnemySpawnTimer>,

    //mut input_events: EventReader<InputEvent>,
    //meshes: Res<Assets<Mesh>>,
    //mut materials: ResMut<Assets<StandardMaterial>>,
    mut enemy_query: Query<(Entity, &mut Enemy, &mut Transform)>,
    assets_server: Res<AssetServer>,
) {
    spawn_timer.0.tick(time.delta());

    if spawn_timer.0.finished() {
        let mesh = assets_server.load("cube.gltf#Mesh0/Primitive0");

        commands
            .spawn_bundle(PbrBundle {
                mesh,
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, 0.0),
                    ..Default::default()
                },
                //material: player_material.clone(),
                ..Default::default()
            })
            .insert(Enemy(0.0, 0.0));
    }

    for (_, mut enemy, mut trans) in enemy_query.iter_mut() {
        trans.rotation = Quat::from_rotation_y(enemy.0);
        enemy.0 += 1.0 * time.delta_seconds() * enemy.1;
        enemy.1 += 1.0 * time.delta_seconds() * thread_rng().gen_range(-1.0..1.0);

        let (x, z) = enemy.0.sin_cos();

        trans.translation += Vec3::new(x, 0.0, z) * time.delta_seconds() * 20.0;
    }
}

fn setup() {}

struct EnemySpawnTimer(Timer);

pub fn build(app: &mut AppBuilder) {
    app
        //.init_resource::<>()
        .insert_resource(EnemySpawnTimer(Timer::from_seconds(2.0, true)))
        .add_startup_system(setup.system())
        .add_system(update.system());
}
