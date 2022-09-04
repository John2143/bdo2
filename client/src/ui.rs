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
#[derive(Component)]
struct UIDebugMarker;

fn setup_debug_info(mut commands: Commands, assets_server: Res<AssetServer>) {
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                //Entire screen
                size: Size::new(Val::Auto, Val::Auto),
                ..Default::default()
            },
            ..Default::default()
        })
        .with_children(|thing| {
            thing
                .spawn_bundle(TextBundle {
                    text: Text {
                        alignment: TextAlignment::default(),
                        sections: vec![TextSection {
                            value: "Something wrong with debug text monkaS".to_string(),
                            style: TextStyle {
                                font_size: 25.0,
                                font: assets_server.load("JetBrainsMono-Regular.ttf"),
                                color: Color::RED,
                            },
                        }],
                    },
                    style: Style {
                        margin: UiRect {
                            bottom: Val::Px(64.0),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(UIDebugMarker);
        });
}

fn system_update_debug_info(info: Res<UIDebugInfo>, mut text: Query<(&mut Text, &UIDebugMarker)>) {
    for (mut text, _) in text.iter_mut() {
        //prevent allocs by copying strings
        use std::fmt::Write;
        let s = &mut text.sections[0].value;
        s.truncate(0);
        write!(s, "{}", *info).unwrap();
    }
}

pub fn build(app: &mut App) {
    app.init_resource::<UIDebugInfo>()
        .add_startup_system(setup_debug_info)
        .add_system(system_update_debug_info);
}
