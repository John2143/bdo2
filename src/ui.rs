use bevy::prelude::*;

///All info Displayed in Debug screen, updated by various systems
//TODO: should probably be an Arc mutex to help parallelization, not sure if bevy does that by
//default
#[derive(Default)]
pub struct UIDebugInfo {
    pub speed: f32,
}

impl std::fmt::Display for UIDebugInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:3.2}", self.speed)?;

        Ok(())
    }
}

///Component for Debug UI Text
struct UIDebugMarker;

fn setup_debug_info(
    mut commands: Commands,
    mut c_materials: ResMut<Assets<ColorMaterial>>,
    assets_server: Res<AssetServer>,
) {

    commands
        .spawn(UiCameraComponents {
            ..Default::default()
        })
        .spawn(NodeComponents {
            style: Style {
                //Entire screen
                size: Size::new(Val::Auto, Val::Auto),
                ..Default::default()
            },
            material: c_materials.add(Color::hex("F44").unwrap().into()),
            ..Default::default()
        })
        .spawn(TextComponents {
            text: Text {
                value: "Something wrong with debug text monkaS".to_string(),
                font: assets_server.load("JetBrainsMono-Regular.ttf"),
                style: TextStyle {
                    font_size: 25.0,
                    color: Color::WHITE
                },
            },
            ..Default::default()
        })
        .with(UIDebugMarker);
}

fn system_update_debug_info(
    info: Res<UIDebugInfo>,
    mut text: Query<(&mut Text, &UIDebugMarker)>
) {
    for (mut text, _) in text.iter_mut() {
        //prevent allocs by copying strings
        use std::fmt::Write;
        text.value.truncate(0);
        write!(&mut text.value, "{}", *info).unwrap();
    }
}

pub fn build(app: &mut AppBuilder) { 
    app
        .init_resource::<UIDebugInfo>()
        .add_startup_system(setup_debug_info.system())
        .add_system(system_update_debug_info.system());
}
