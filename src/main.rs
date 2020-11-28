use bevy::{input::mouse::MouseMotion, prelude::*};

fn main() {
    App::build()
        //.add_resource(Msaa { samples: 4 })
        .init_resource::<State>()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(update_player.system())
        .run();
}

//based on Bevy-WoW camera
struct CameraOrientation {
    yaw: f32,
    pitch: f32,
    roll: f32,

    distance: f32,

    attached_entity: Option<Entity>,
}

#[derive(Default)]
struct State {
    mouse_motion_event_reader: EventReader<MouseMotion>,
}

impl Default for CameraOrientation {
    fn default() -> Self {
        Self {
            yaw: 0.,
            pitch: 0.,
            roll: 0.,
            distance: 100.,
            attached_entity: None,
        }
    }
}

struct PlayerCamera;

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let player_material = materials.add({
        let mut q = StandardMaterial::from(Color::rgb(1.0, 0.5, 0.0));
        q.shaded = true;
        q
    });

    // add entities to the world
    let e = commands
        .spawn(Camera3dComponents::default())
        .with(PlayerCamera)
        .current_entity();

    let player = commands
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.9 })),
            transform: Transform {
                translation: Vec3::new(0.0, 1.9, 0.0),
                ..Default::default()
            },
            material: player_material.clone(),
            ..Default::default()
        })
        .with(CameraOrientation {
            attached_entity: e,
            ..Default::default()
        })
        .current_entity();

    commands.push_children(player.unwrap(), &[e.unwrap()]);
    commands.spawn(LightComponents::default());
    commands.spawn(PbrComponents {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 300.0 })),
        material: materials.add(Color::rgb(0.1, 0.5, 0.7).into()),
        ..Default::default()
    });

    let cubes = [
        (5.0, 1.0, 5.0),
        (25.0, 1.0, 45.0),
        (-20.0, 1.0, 0.0),
    ];
    for cube in &cubes{
        commands.spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.5, 0.5, 0.5).into()),
            transform: Transform {
                translation: Vec3::new(cube.0, cube.1, cube.2),
                ..Default::default()
            },
            ..Default::default()
        });
    }
}

fn mouse(
    time: Res<Time>,
    mut state: ResMut<State>,
    mouse_motion: Res<Events<MouseMotion>>,
    mut query: Query<&mut CameraOrientation>,
) {
}

trait RemappableFloat {}
impl RemappableFloat for f64 {}
impl RemappableFloat for f32 {}

fn remap<T>(src: T, (src_min, src_max): (T, T), (dest_min, dest_max): (T, T)) -> T
where
    T: RemappableFloat
        + Copy
        + std::ops::Sub<Output = T>
        + std::cmp::PartialOrd
        + std::ops::Mul<Output = T>
        + std::ops::Div<Output = T>
        + std::ops::Add<Output = T>,
{
    if src < src_min {
        return dest_min;
    } else if src > src_max {
        return dest_max;
    }

    let off = src - src_min;
    let off_pct = off / (src_max - src_min);

    let dest_range = dest_max - dest_min;

    dest_min + dest_range * off_pct
}

fn update_player(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut player_query: Query<(&mut CameraOrientation, &mut Transform)>,
    mut camera_query: Query<(&mut PlayerCamera, Entity, &mut Transform)>,
) {
    let mut movement = Vec2::zero();
    if keyboard_input.pressed(KeyCode::E) {
        *movement.y_mut() += 1.;
    }
    if keyboard_input.pressed(KeyCode::D) {
        *movement.y_mut() -= 1.;
    }
    if keyboard_input.pressed(KeyCode::F) {
        *movement.x_mut() += 1.;
    }
    if keyboard_input.pressed(KeyCode::S) {
        *movement.x_mut() -= 1.;
    }

    if movement != Vec2::zero() {
        movement.normalize();
    }

    let move_speed = 10.0;
    movement *= time.delta_seconds * move_speed;

    for (mut player, mut transform) in player_query.iter_mut() {
        player.pitch = player.pitch.max(1f32.to_radians()).min(179f32.to_radians());
        player.distance = player.distance.max(5.).min(30.);

        //transform.translation += Vec3::new(0.0, 1.0, 0.0) * time.delta_seconds;

        player.pitch = remap(
            (time.seconds_since_startup * 0.2).sin(),
            (-1.0, 1.0),
            (25.0f64.to_radians(), 90.0f64.to_radians()),
        ) as f32;

        player.yaw = remap(
            (time.seconds_since_startup * 0.3).cos(),
            (-1.0, 1.0),
            (-180.0f64.to_radians(), 180.0f64.to_radians()),
        ) as f32;

        if player.pitch > 179f32.to_radians() {
            player.pitch -= 179f32.to_radians()
        }

        //println!("{} {}", player.yaw, player.pitch);

        transform.translation += Vec3::new(0.0, 0.0, 0.0);
        transform.rotation = Quat::from_rotation_y(-player.yaw);

        if let Some(camera_entity) = player.attached_entity {
            let cam_pos = Vec3::new(0., player.pitch.cos(), -player.pitch.sin()).normalize()
                * player.distance;
            for (_, e, mut camera3dtrans) in camera_query.iter_mut() {
                if camera_entity != e {
                    continue;
                }
                camera3dtrans.translation = cam_pos;
                let look = Mat4::face_toward(cam_pos, Vec3::zero(), Vec3::new(0.0, 1.0, 0.0));
                camera3dtrans.rotation = look.to_scale_rotation_translation().1;
            }
        }
    }
}
