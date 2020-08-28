use bevy::prelude::{
    shape,
    App,
    AppBuilder,
    Commands,
    Input,
    IntoQuerySystem,
    KeyCode,
    Query,
    //IntoForEachSystem,
    //IntoThreadLocalSystem,
    Res,
    ResMut,
    Time,
    Vec3,
};

use bevy_pbr::prelude::*;
use bevy_transform::prelude::Translation;

use bevy_render::prelude::{Color, Mesh, Msaa};

use bevy_asset::Assets;

use bevy::prelude::*;

mod camera;
use camera::{FlyCamera, FlyCameraPlugin};

fn default_plugins() -> AppBuilder {
    let mut app = App::build();

    app.add_resource(Msaa { samples: 4 });
    app.add_plugin(bevy_type_registry::TypeRegistryPlugin::default());
    app.add_plugin(bevy_core::CorePlugin::default());
    app.add_plugin(bevy_transform::TransformPlugin::default());
    app.add_plugin(bevy_diagnostic::DiagnosticsPlugin::default());
    app.add_plugin(bevy_input::InputPlugin::default());
    app.add_plugin(bevy_window::WindowPlugin::default());
    app.add_plugin(bevy_asset::AssetPlugin::default());
    app.add_plugin(bevy_scene::ScenePlugin::default());
    app.add_plugin(bevy_render::RenderPlugin::default());
    //app.add_plugin(bevy_sprite::SpritePlugin::default());
    app.add_plugin(bevy_pbr::PbrPlugin::default());
    //app.add_plugin(bevy_ui::UiPlugin::default());
    //app.add_plugin(bevy_text::TextPlugin::default());

    //app.add_plugin(bevy_audio::AudioPlugin::default());
    //app.add_plugin(bevy_gltf::GltfPlugin::default());
    app.add_plugin(bevy_winit::WinitPlugin::default());
    app.add_plugin(bevy_wgpu::WgpuPlugin::default());

    app
}

fn main() {
    default_plugins()
        .add_plugin(FlyCameraPlugin)
        .add_startup_system(setup.system())
        .add_system(control_objects.system())
        .add_system(global_keys.system())
        .add_system(camera_look_rot.system())
        .run();
}

fn global_keys(
    //events: Res<Events<>>,
    input: Res<Input<KeyCode>>,
) {
    if input.pressed(KeyCode::Escape) {}
}

struct Control(bool);

struct Phys {
    velocity: Vec3,
}

impl Default for Phys {
    fn default() -> Self {
        Self {
            velocity: Default::default(),
        }
    }
}

fn is_on_ground(v: &Vec3) -> bool {
    v.y() <= 0.0
}

fn set_clamped<T: std::cmp::PartialOrd>(val: T, min: T, max: T) -> T {
    if val < min {
        min
    } else if val > max {
        max
    } else {
        val
    }
}

use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref CAMERA_ROT: Mutex<Rotation> = Mutex::new(Rotation::identity());
}

fn camera_look_rot(mut query: Query<(&camera::ModifiedFlyCameraOptions, &Rotation)>) {
    let mut cam = CAMERA_ROT.lock().unwrap();
    for (_options, rotation) in &mut query.iter() {
        cam.clone_from(&rotation);
    }
}

fn control_objects(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut query: Query<(&Control, &mut Translation, &mut Phys)>,
) {
    for (control, mut translation, mut phys) in &mut query.iter() {
        if !control.0 {
            continue;
        }

        let grounded = is_on_ground(&translation.0);

        //gravity
        *phys.velocity.y_mut() -= 20.0 * time.delta_seconds;

        if grounded {
            if phys.velocity.y() < 0.0 {
                *phys.velocity.y_mut() = 0.0;
            }
            *translation.y_mut() = 0.0;

            if input.pressed(KeyCode::Space) {
                *phys.velocity.y_mut() += 8.0;
            }
        }

        let accel = if grounded { 120.0 } else {
            if input.pressed(KeyCode::LShift) {
                300.0
            }else{
                20.0
            }
        };
        let mut is_moving = false;
        if input.pressed(KeyCode::D) {
            let newvel = set_clamped(
                phys.velocity.x() + accel * time.delta_seconds,
                -999990.0,
                20.0,
            );
            *phys.velocity.x_mut() = newvel;
            is_moving = true;
        }
        if input.pressed(KeyCode::A) {
            let newvel = set_clamped(
                phys.velocity.x() - accel * time.delta_seconds,
                -20.0,
                9999990.0,
            );
            *phys.velocity.x_mut() = newvel;
            is_moving = true;
        }

        //drag
        let horizontal_factor = if grounded && !is_moving { 10.0 } else { 0.05 };
        *phys.velocity.x_mut() -= phys.velocity.x() * horizontal_factor * time.delta_seconds;
        *phys.velocity.z_mut() -= phys.velocity.z() * horizontal_factor * time.delta_seconds;

        translation.0 += phys.velocity * time.delta_seconds;
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_handle = meshes.add(Mesh::from(shape::Cube { size: 1.7 }));
    let cube_material_handle = materials.add(StandardMaterial {
        albedo: Color::rgb(0.5, 0.4, 0.3),
        ..Default::default()
    });

    commands
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 10000.0 })),
            material: materials.add(Color::rgb(0.1, 0.2, 0.1).into()),
            translation: Translation::new(0.0, -10.0, 0.0),
            ..Default::default()
        })
        .spawn(LightComponents {
            translation: Translation::new(4.0, 5.0, -4.0),
            ..Default::default()
        })
        // camera
        //.spawn(Camera3dComponents {
        //transform: Transform::new_sync_disabled(Mat4::face_toward(
        //Vec3::new(5.0, 10.0, 10.0),
        //Vec3::new(0.0, 0.0, 0.0),
        //Vec3::new(0.0, 1.0, 0.0),
        //)),
        //..Default::default()
        //});
        .spawn(FlyCamera::default());

    commands
        .spawn(PbrComponents {
            mesh: cube_handle,
            material: cube_material_handle,
            translation: Translation::new(0.0, 2.0, 0.0),
            ..Default::default()
        })
        .with(Control(true))
        .with(Phys::default());
}
