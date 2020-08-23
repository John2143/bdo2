use bevy::prelude::*;
use bevy_fly_camera::{FlyCamera, FlyCameraPlugin};

/// This example illustrates how to create parent->child relationships between entities how parent transforms
/// are propagated to their descendants
fn main() {
    App::build()
        .add_resource(Msaa { samples: 4 })
        .add_default_plugins()
        .add_plugin(FlyCameraPlugin)
        .add_startup_system(setup.system())
        .add_system(control_objects.system())
        .run();
}

/// this component indicates what entities should rotate
struct Controlable;

/// rotates the parent, which will result in the child also rotating
fn control_objects(time: Res<Time>, mut query: Query<(&Controlable, &mut Translation)>) {
    for (_, mut translation) in &mut query.iter() {
        *translation.0.x_mut() += 1.0 * time.delta_seconds;
    }
}

/// set up a simple scene with a "parent" cube and a "child" cube
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_handle = meshes.add(Mesh::from(shape::Cube { size: 1.0 }));
    let cube_material_handle = materials.add(StandardMaterial {
        albedo: Color::rgb(0.5, 0.4, 0.3),
        ..Default::default()
    });

    commands
        // plane
        .spawn(PbrComponents {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 10000.0 })),
            material: materials.add(Color::rgb(0.1, 0.2, 0.1).into()),
            ..Default::default()
        })
        .spawn(PbrComponents {
            mesh: cube_handle,
            material: cube_material_handle,
            translation: Translation::new(0.0, 0.0, 1.0),
            ..Default::default()
        })
        .with(Controlable)
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
        .spawn(FlyCamera::default())
        ;
}
