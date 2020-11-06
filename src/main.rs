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

fn control_objects(
    time: Res<Time>,
    input: Res<Input<KeyCode>>,
    mut camera_query: Query<(&camera::ModifiedFlyCameraOptions, &Rotation)>,
    mut query: Query<(&Control, &mut Translation, &mut Phys)>,
) {
    let mut cam: Rotation = Rotation::identity();

    for (_options, rotation) in &mut camera_query.iter() {
        cam.clone_from(&rotation);
        break;
    }

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

        use camera::{forward_walk_vector, movement_axis, strafe_vector};

        let axis_h = movement_axis(&input, KeyCode::D, KeyCode::A);
        let axis_v = movement_axis(&input, KeyCode::S, KeyCode::W);

        let delta_f = forward_walk_vector(&cam) * axis_v * 120.0 * time.delta_seconds;
        let delta_strafe = strafe_vector(&cam) * axis_h * 120.0 * time.delta_seconds;

        phys.velocity += delta_f + delta_strafe;

        let xz_move = Vec3::new(phys.velocity.x(), 0.0, phys.velocity.z());
        if xz_move.length_squared() > 0.01 {
            phys.velocity -= xz_move.normalize() * 0.5 * time.delta_seconds;
            dbg!(phys.velocity);
            dbg!(xz_move.normalize());
        }

        translation.0 += phys.velocity * time.delta_seconds;
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_size = 2.0;
    let cube_handle = meshes.add(Mesh::from(shape::Cube { size: cube_size }));
    let cube_material_handle = materials.add(StandardMaterial {
        albedo: Color::rgb(0.5, 0.4, 0.3),
        ..Default::default()
    });

    commands
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 10000.0 })),
            material: materials.add(Color::rgb(0.1, 0.2, 0.1).into()),
            translation: Translation::new(0.0, -cube_size, 0.0),
            ..Default::default()
        })
        .spawn(LightComponents {
            translation: Translation::new(4.0, 5.0, -4.0),
            ..Default::default()
        })
        .spawn(LightComponents {
            translation: Translation::new(4.0, 5.0, 4.0),
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
        .spawn(FlyCamera {
            translation: Translation::new(10.0, 5.0, 4.0),
            rotation: Rotation::from_rotation_x(50.0),
            ..Default::default()
        });
        //.spawn(FlyCamera::default());

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
