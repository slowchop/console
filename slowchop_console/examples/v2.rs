use bevy::prelude::*;
use slowchop_console::{Actions, ActionsHandler, Console, ConsolePlugin};
use slowchop_console_derive::Actions2;

#[derive(Actions2, Clone, Debug, Event)]
enum MyGameActions {
    Nope,
    // RefEnum(Vec2),
    RefEnum(EmuSub),
    // RefEnumWithF32(EmuSub, f32),
    // InlineStructToEnum { x: f32, y: Option<EmuSub> },
    // RefStructToEnum(Struct),
}

#[derive(Actions2, Clone, Debug)]
enum EmuSub {
    One,
}

// #[derive(Actions2, Clone, Debug)]
// struct Struct {
//     x: f32,
//     y: Option<EmuSub>,
// }

pub fn main() {
    let default_filter =
        "trace,slowchop_console=info,wgpu=error,naga=warn,bevy=info,winit=info,gilrs=info"
            .to_string();
    std::env::set_var("RUST_LOG", default_filter);

    let console_plugin = ConsolePlugin::<MyGameActions>::default();

    return;

    App::new()
        .add_plugins((
            DefaultPlugins.set(bevy::log::LogPlugin {
                update_subscriber: Some(console_plugin.update_subscriber()),
                ..default()
            }),
            console_plugin,
            // WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::F1)),
        ))
        .add_systems(Startup, (setup_camera, start_with_console_open))
        .add_systems(Update, handle_actions)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn start_with_console_open(mut console: ResMut<Console<MyGameActions>>) {
    console.open();

    trace!(?console.background_color, "This is a test trace message with a variable.");
    debug!("This is a test debug message: 0xABCDEF");
    info!("Press ` (backtick) or ~ (tilde) (KeyCode::Backquote) to toggle the console.");
    warn!("Press Escape to close the console.");
    error!("This is a test error message: 0xABCDEF");
}

fn handle_actions(mut actions: EventReader<MyGameActions>) {
    for action in actions.read() {
        info!(?action);
    }
}
