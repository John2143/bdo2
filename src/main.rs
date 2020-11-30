#![feature(min_const_generics, cow_is_borrowed)]

use bevy::{input::mouse::MouseMotion, input::mouse::MouseWheel, prelude::*};

mod config;
mod config_read;
mod utils;
mod ui;
mod camera;

use utils::RotatableVector;

fn main() {
    let mut app = App::build();

    app
        .add_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .init_resource::<MouseInputState>()
        .add_startup_system(setup_scene.system())
        .add_startup_system(setup_window.system())
        .add_system(system_update_player_cam.system())
        .add_system(system_update_movement.system())
        .add_system(system_window.system())
        .add_system(system_mouse.system())
        ;

    ui::build(&mut app);
    config::build(&mut app);

    app.run();
}

///based on Bevy-WoW camera
///angles are in radians
struct CameraOrientation {
    yaw: f32,
    ///0 = straight up vector (looking directly down at the ground)
    ///180 = straight down vector (looking up towards bottom of player)
    pitch: f32,

    #[allow(dead_code)]
    roll: f32,

    //meters from cam
    distance: f32,

    attached_entity: Option<Entity>,
}

#[derive(Default)]
struct MouseInputState {
    //set to true if events should be chomped
    no_mouse_inputs: bool,
    mouse_motion_event_reader: EventReader<MouseMotion>,
    mouse_wheel_event_reader: EventReader<MouseWheel>,
}

impl Default for CameraOrientation {
    fn default() -> Self {
        Self {
            yaw: 0.,
            pitch: 60f32.to_radians(),
            roll: 0.,
            distance: 50.,
            attached_entity: None,
        }
    }
}

struct PhysicsProperties {
    movement_speed_ground: f32,
    movement_speed_air: f32,
    movement_acceleration: f32,
}

struct Physics {
    gravity_func: fn(f32, f32) -> f32,
    velocity: Vec3,
    walking_velocity: Vec2,
    dash_velocity: Vec3,
    last_jump: f32,
    dash_cooldown: f32,
}

impl Default for Physics {
    fn default() -> Self {
        Self {
            gravity_func: |_, _| 9.8,
            velocity: Vec3::zero(),
            walking_velocity: Vec2::zero(),
            last_jump: -10.,
            dash_cooldown: 0.0,
            dash_velocity: Vec3::zero(),
        }
    }
}

//marker trait attached to the spawned camera indicating that our ent probably needs to control it
struct PlayerCamera;

/// set up a simple 3D scene
fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    assets_server: Res<AssetServer>,
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
        .current_entity()
        .unwrap();

    let player_mesh = assets_server.load("cube.gltf#Mesh0/Primitive0");

    //commands.spawn_scene(player_mesh);

    let player = commands
        .spawn(PbrComponents {
            mesh: player_mesh,
            transform: Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                ..Default::default()
            },
            material: player_material.clone(),
            ..Default::default()
        })
        .with(CameraOrientation {
            attached_entity: Some(e),
            ..Default::default()
        })
        .with(PhysicsProperties {
            movement_speed_ground: 15.0,
            movement_speed_air: 2.0,
            movement_acceleration: 15.0 * 10.0,
        })
        .with(Physics {
            gravity_func: |_x, _launchvel| {
                //let offset = 25.0;
                //35f32.min((x - 0.5).powf(2.0) + offset)

                30.0

                //35f32.min(15. * x)
            },
            last_jump: -100.0,
            ..Default::default()
        })
        .current_entity();

    //let the camera transform/rotate with the player.
    commands.push_children(player.unwrap(), &[e]);

    commands.spawn(LightComponents {
        transform: Transform {
            translation: Vec3::new(0.0, 5.0, 0.0),
            ..Default::default()
        },
        //light: Light {
        //color: Color::rgb(1.0, 0.5, 0.5),
        //..Default::default()
        //},
        ..Default::default()
    });

    let floor_handle = assets_server.load("floor.png");

    commands.spawn(PbrComponents {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 2000.0 })),
        material: materials.add(floor_handle.into()),
        ..Default::default()
    });

    let cubes = [(5.0, 1.0, 5.0), (25.0, 1.0, 45.0), (-20.0, 1.0, 0.0)];
    for cube in &cubes {
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

    //banana
    commands.spawn(PbrComponents {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.2 })),
        material: materials.add(Color::rgb(1.0, 0.92, 0.21).into()),
        transform: Transform {
            translation: Vec3::new(-5.0, 1.0, 0.0),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn setup_window(mut windows: ResMut<Windows>) {
    let window = windows.get_primary_mut().unwrap();
    window.set_cursor_lock_mode(true);
    window.set_cursor_visibility(false);
    window.set_title("9.99$ game btw".into());
}

fn system_window(
    mut windows: ResMut<Windows>,
    keyboard_input: Res<Input<KeyCode>>,
    mut state: ResMut<MouseInputState>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        let window = windows.get_primary_mut().unwrap();
        window.set_cursor_lock_mode(!window.cursor_locked());
        window.set_cursor_visibility(!window.cursor_visible());
        state.no_mouse_inputs = !state.no_mouse_inputs;
    }
}

fn system_mouse(
    mut state: ResMut<MouseInputState>,
    config: Res<config::Config>,
    mouse_motion: Res<Events<MouseMotion>>,
    mouse_wheel: Res<Events<MouseWheel>>,
    mut query: Query<&mut CameraOrientation>,
) {
    let mut camera = query.iter_mut().next().unwrap();
    let mut look = if let Some(event) = state.mouse_motion_event_reader.iter(&mouse_motion).next() {
        event.delta
    } else {
        Vec2::zero()
    };

    let wheel = if let Some(event) = state.mouse_wheel_event_reader.iter(&mouse_wheel).next() {
        event.y
    } else {
        0.0
    };

    if state.no_mouse_inputs {
        return;
    }

    let look_sens = config.sens.to_radians();
    look *= look_sens;

    camera.yaw += look.x();
    camera.pitch -= look.y();
    camera.distance -= wheel * config.zoom_sens;

    camera.pitch = camera
        .pitch
        .max(0f32.to_radians() + f32::EPSILON)
        .min(180f32.to_radians() - f32::EPSILON);
    camera.distance = camera.distance.max(5.).min(100.);
}

fn system_update_movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut ui_debug: ResMut<ui::UIDebugInfo>,
    config: Res<config::Config>,
    mut player_query: Query<(
        &CameraOrientation,
        &mut Transform,
        &mut Physics,
        &PhysicsProperties,
    )>,
) {
    let mut movement2d = Vec2::zero();
    let [m_up, m_left, m_down, m_right] = config.movement;
    if keyboard_input.pressed(m_up) {
        *movement2d.y_mut() += 1.;
    }
    if keyboard_input.pressed(m_down) {
        *movement2d.y_mut() -= 1.;
    }
    if keyboard_input.pressed(m_right) {
        *movement2d.x_mut() += 1.;
    }
    if keyboard_input.pressed(m_left) {
        *movement2d.x_mut() -= 1.;
    }

    if movement2d != Vec2::zero() {
        movement2d.normalize();
    }

    use utils::Vec2toVec3;

    for (player_cam, mut player_transform, mut phys, phys_prop) in player_query.iter_mut() {
        let is_in_air = if player_transform.translation.y() <= 0.0 {
            false
        } else {
            true
        };
        movement2d *= if is_in_air {
            phys_prop.movement_speed_air
        } else {
            phys_prop.movement_speed_ground
        };
        movement2d *= phys_prop.movement_acceleration / phys_prop.movement_speed_ground;

        let delta_y_vel =
            (phys.gravity_func)(time.seconds_since_startup as f32 - phys.last_jump, 5.0);
        let delta_y_vel = delta_y_vel * time.delta_seconds;

        phys.velocity -= Vec3::new(0.0, delta_y_vel, 0.0);

        let movement2d = movement2d.rotate(player_cam.yaw - 90.0f32.to_radians());
        if !is_in_air {
            phys.walking_velocity /= 1.05 / (1.0 - time.delta_seconds);
            if phys.walking_velocity.length_squared() < 0.7 {
                phys.walking_velocity /= 2.0 / (1.0 - time.delta_seconds);
            }
        }
        phys.walking_velocity += movement2d * time.delta_seconds;

        if phys.walking_velocity.length() > phys_prop.movement_speed_ground {
            phys.walking_velocity =
                phys.walking_velocity.normalize() * phys_prop.movement_speed_ground;
        }

        ui_debug.speed = phys.walking_velocity.length();

        player_transform.translation +=
            (phys.velocity + phys.walking_velocity.xz3()) * time.delta_seconds;

        if !is_in_air {
            player_transform.translation.set_y(0.0);
            phys.velocity.set_y(0.0);

            if keyboard_input.pressed(config.jump) {
                phys.velocity.set_y(15.0);
                player_transform.translation.set_y(0.0 + f32::EPSILON);
                phys.last_jump = time.seconds_since_startup as f32;
            }
        }
    }
}

fn system_update_player_cam(
    mut player_query: Query<(&CameraOrientation, &mut Transform)>,
    mut camera_query: Query<(&PlayerCamera, Entity, &mut Transform)>,
) {
    for (player_cam, mut player_transform) in player_query.iter_mut() {
        //player.yaw = remap(
        //(time.seconds_since_startup * 0.3).cos(),
        //(-1.0, 1.0),
        //(-180.0f64.to_radians(), 180.0f64.to_radians()),
        //) as f32;

        player_transform.rotation = Quat::from_rotation_y(-player_cam.yaw);

        if let Some(camera_entity) = player_cam.attached_entity {
            let (pitch_sin, pitch_cos) = player_cam.pitch.sin_cos();
            let cam_pos = Vec3::new(0., pitch_cos, pitch_sin).normalize() * player_cam.distance;
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
