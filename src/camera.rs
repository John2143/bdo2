use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use bevy_render::camera::{Camera, PerspectiveProjection, VisibleEntities};
use bevy_render::render_graph::base;

pub struct ModifiedFlyCameraOptions {
    pub speed: f32,
    pub sensitivity: f32,
    pub pitch: f32,
    pub yaw: f32,

    pub free: bool,

    pub key_forward: KeyCode,
    pub key_backward: KeyCode,
    pub key_left: KeyCode,
    pub key_right: KeyCode,
    pub key_up: KeyCode,
    pub key_down: KeyCode,
}

impl Default for ModifiedFlyCameraOptions {
    fn default() -> Self {
        Self {
            speed: 10.0,
            sensitivity: 3.0,
            pitch: 0.0,
            yaw: 0.0,

            free: true,

            key_forward: KeyCode::W,
            key_backward: KeyCode::S,
            key_left: KeyCode::A,
            key_right: KeyCode::D,
            key_up: KeyCode::Space,
            key_down: KeyCode::LShift,
        }
    }
}

#[derive(Bundle)]
pub struct FlyCamera {
    pub options: ModifiedFlyCameraOptions,
    pub camera: Camera,
    pub perspective_projection: PerspectiveProjection,
    pub visible_entities: VisibleEntities,
    pub transform: Transform,
    pub translation: Translation,
    pub rotation: Rotation,
    pub scale: Scale,
}

impl Default for FlyCamera {
    fn default() -> Self {
        Self {
            options: ModifiedFlyCameraOptions::default(),
            camera: Camera {
                name: Some(base::camera::CAMERA3D.to_string()),
                ..Default::default()
            },
            perspective_projection: Default::default(),
            visible_entities: Default::default(),
            transform: Default::default(),
            translation: Default::default(),
            rotation: Default::default(),
            scale: Default::default(),
        }
    }
}

fn forward_vector(rotation: &Rotation) -> Vec3 {
    rotation.mul_vec3(Vec3::unit_z()).normalize()
}

pub fn forward_walk_vector(rotation: &Rotation) -> Vec3 {
    let f = forward_vector(rotation);
    Vec3::new(f.x(), 0.0, f.z()).normalize()
}

pub fn strafe_vector(rotation: &Rotation) -> Vec3 {
    // Rotate it 90 degrees to get the strafe direction
    Rotation::from_rotation_y(90.0f32.to_radians())
        .mul_vec3(forward_walk_vector(rotation))
        .normalize()
}

pub fn movement_axis(input: &Res<Input<KeyCode>>, plus: KeyCode, minus: KeyCode) -> f32 {
    let mut axis = 0.0;
    if input.pressed(plus) {
        axis += 1.0;
    }
    if input.pressed(minus) {
        axis -= 1.0;
    }
    axis
}

fn camera_movement_system(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut ModifiedFlyCameraOptions, &mut Translation, &Rotation)>,
) {
    for (mut options, mut translation, rotation) in &mut query.iter() {
        if keyboard_input.just_pressed(KeyCode::C) {
            options.free = !options.free;
        }
        if !options.free {
            continue;
        }

        let axis_h = movement_axis(&keyboard_input, options.key_right, options.key_left);
        let axis_v = movement_axis(&keyboard_input, options.key_backward, options.key_forward);

        let axis_float = movement_axis(&keyboard_input, options.key_up, options.key_down);
        let delta_f = forward_walk_vector(rotation) * axis_v * options.speed * time.delta_seconds;

        let delta_strafe = strafe_vector(rotation) * axis_h * options.speed * time.delta_seconds;

        let delta_float = Vec3::unit_y() * axis_float * options.speed * time.delta_seconds;

        translation.0 += delta_f + delta_strafe + delta_float;
    }
}

#[derive(Default)]
struct State {
    mouse_motion_event_reader: EventReader<MouseMotion>,
}

fn mouse_motion_system(
    time: Res<Time>,
    mut state: ResMut<State>,
    mouse_motion_events: Res<Events<MouseMotion>>,
    mut query: Query<(&mut ModifiedFlyCameraOptions, &mut Rotation)>,
) {
    let mut delta: Vec2 = Vec2::zero();
    for event in state.mouse_motion_event_reader.iter(&mouse_motion_events) {
        delta += event.delta;
    }
    if delta == Vec2::zero() {
        return;
    }

    for (mut options, mut rotation) in &mut query.iter() {
        options.yaw -= delta.x() * options.sensitivity * time.delta_seconds;
        options.pitch += delta.y() * options.sensitivity * time.delta_seconds;

        if options.pitch > 89.9 {
            options.pitch = 89.9;
        }
        if options.pitch < -89.9 {
            options.pitch = -89.9;
        }
        // println!("pitch: {}, yaw: {}", options.pitch, options.yaw);

        let yaw_radians = options.yaw.to_radians();
        let pitch_radians = options.pitch.to_radians();

        rotation.0 = Quat::from_axis_angle(Vec3::unit_y(), yaw_radians)
            * Quat::from_axis_angle(-Vec3::unit_x(), pitch_radians);
    }
}

pub struct FlyCameraPlugin;

impl Plugin for FlyCameraPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<State>()
            .add_system(camera_movement_system.system())
            .add_system(mouse_motion_system.system());
    }
}
