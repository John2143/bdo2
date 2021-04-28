use bevy::prelude::*;
use rand::Rng;

fn setup(
) {
}

struct Proj(Vec3, f32);

fn update(
    mut commands: Commands,
    //time: Res<Time>,
    //config: Res<crate::config::Config>,
    //keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut player_query: Query<(&crate::CameraOrientation, &Transform)>,
) {
    if mouse_input.pressed(MouseButton::Left) {
        let (orientation, pos) = match player_query.iter_mut().next() {
            Some(p) => p,
            None => return,
        };

        commands.spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 0.2 })),
            material: materials.add(Color::PURPLE.into()),
            transform: Transform {
                translation: pos.translation + Vec3::Y * 2.0,
                ..Default::default()
            },
            ..Default::default()
        }).insert(Proj(orientation.xy_vector() * 100.0, 5.0));
    }
}

fn move_proj(
    mut commands: Commands,
    time: Res<Time>,
    mut projs: Query<(Entity, &mut Proj, &mut Transform)>,
) {
    for (id, mut proj_data, mut transform) in projs.iter_mut() {
        transform.translation += time.delta_seconds() * proj_data.0;
        proj_data.1 -= time.delta_seconds();
        if proj_data.1 <= 0.0 {
            commands.entity(id).despawn_recursive();
        }
    }
}

struct TestTimer(Timer);
struct Food;

fn test(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<TestTimer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets_server: Res<AssetServer>,
    mut players: Query<(Entity, &Transform, &mut crate::Physics)>,
    foods: Query<(Entity, &Transform, &Food)>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        let player_mesh = assets_server.load("cube.gltf#Mesh0/Primitive0");
        let r = rand::thread_rng().gen_range(-20.0..20.0);
        //let r = 20.0;

        commands
        .spawn_bundle(PbrBundle {
            mesh: player_mesh,
            material: materials.add(Color::ORANGE_RED.into()),
            transform: Transform {
                translation: Vec3::new(r, 0.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Food);
    }

    let trans_plr = match players.iter_mut().next() {
        Some((_, e, _)) => e,
        None => return,
    };

    for (ent_food, trans_food, _) in foods.iter() {
        if trans_food.translation.distance(trans_plr.translation) < 1.0 {
            info!("ate food");

            commands.entity(ent_food).despawn_recursive();
        }
    }
}

pub fn build(app: &mut AppBuilder) {
    app
        //.init_resource::<>()
        //.add_resource(NetworkingTimer(Timer::from_seconds(1.0 / 120.0, true)))
        .add_startup_system(setup.system())
        .add_system(update.system())
        .add_system(move_proj.system())
        .insert_resource(TestTimer(Timer::from_seconds(2.0, true)))
        .add_system(test.system());
}
