use bevy::prelude::*;

///All info Displayed in Debug screen, updated by various systems
//TODO: should probably be an Arc mutex to help parallelization, not sure if bevy does that by
//default
#[derive(Default)]
pub struct UIDebugInfo {
    pub speed: f32,
    pub updates: usize,
    pub fr: f64,
}

impl std::fmt::Display for UIDebugInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{:3.2} m/s walk", self.speed)?;
        writeln!(f, "{:3.0} fps", self.fr)?;

        Ok(())
    }
}

///Component for Debug UI Text
struct UIDebugMarker;

fn setup_debug_info(
    commands: &mut Commands,
    mut c_materials: ResMut<Assets<ColorMaterial>>,
    assets_server: Res<AssetServer>,
) {
    commands
        .spawn(CameraUiBundle {
            ..Default::default()
        })
        .spawn(NodeBundle {
            style: Style {
                //Entire screen
                size: Size::new(Val::Auto, Val::Auto),
                ..Default::default()
            },
            material: c_materials.add(Color::NONE.into()),
            ..Default::default()
        })
        .with_children(|thing| {
            thing
                .spawn(TextBundle {
                    text: Text {
                        value: "Something wrong with debug text monkaS".to_string(),
                        font: assets_server.load("JetBrainsMono-Regular.ttf"),
                        style: TextStyle {
                            font_size: 25.0,
                            color: Color::RED,
                            alignment: TextAlignment::default(),
                        },
                    },
                    style: Style {
                        margin: Rect {
                            bottom: Val::Px(64.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with(UIDebugMarker);
        });
}

fn system_update_debug_info(info: Res<UIDebugInfo>, mut text: Query<(&mut Text, &UIDebugMarker)>) {
    for (mut text, _) in text.iter_mut() {
        //prevent allocs by copying strings
        use std::fmt::Write;
        text.value.truncate(0);
        write!(&mut text.value, "{}", *info).unwrap();
    }
}

pub fn build(app: &mut AppBuilder) {
    app.init_resource::<UIDebugInfo>()
        .add_startup_system(setup_debug_info.system())
        .add_system(system_update_debug_info.system());
}
