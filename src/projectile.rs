use bevy::prelude::*;

use crate::input::InputEvent;

fn setup() {}

struct Proj(Vec3, f32);

fn update(
    mut commands: Commands,
    //time: Res<Time>,
    mut inputs: EventReader<InputEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<(&crate::CameraOrientation, &Transform)>,
) {
    for input in inputs.iter() {
        info!("{:?}", input);
        if let InputEvent::LRClick | InputEvent::BClick = input {
            let (orientation, pos) = match player_query.iter().next() {
                Some(p) => p,
                None => return,
            };

            commands
                .spawn_bundle(PbrBundle {
                    mesh: meshes.add(Mesh::from(shape::Cube { size: 0.2 })),
                    material: materials.add(Color::PURPLE.into()),
                    transform: Transform {
                        translation: pos.translation + Vec3::Y * 2.0,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Proj(orientation.xy_vector() * 100.0, 5.0));
        }
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

pub fn build(app: &mut AppBuilder) {
    app
        //.init_resource::<>()
        //.add_resource(NetworkingTimer(Timer::from_seconds(1.0 / 120.0, true)))
        .add_startup_system(setup.system())
        .add_system(update.system())
        .add_system(move_proj.system());
}
