use bevy::prelude::*;
use rand::{thread_rng, Rng};

#[derive(Component)]
pub struct Enemy {
    pub facing: f32,
    pub facing_vel: f32,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            facing: 0.0,
            facing_vel: 0.0,
        }
    }
}

//use crate::{config::Config, input::InputEvent};

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
            .insert(Enemy::default());
    }

    for (_, mut enemy, mut trans) in enemy_query.iter_mut() {
        trans.rotation = Quat::from_rotation_y(enemy.facing);
        enemy.facing += 1.0 * time.delta_seconds() * enemy.facing_vel;
        enemy.facing_vel += 1.0 * time.delta_seconds() * thread_rng().gen_range(-1.0..1.0);

        let (x, z) = enemy.facing.sin_cos();

        trans.translation += Vec3::new(x, 0.0, z) * time.delta_seconds() * 20.0;
    }
}

fn setup() {}

struct EnemySpawnTimer(Timer);

pub fn build(app: &mut App) {
    app
        //.init_resource::<>()
        .insert_resource(EnemySpawnTimer(Timer::from_seconds(2.0, true)))
        .add_startup_system(setup)
        .add_system(update);
}
