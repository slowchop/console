use bevy::prelude::*;
use slowchop_console::{slowchop_log_layer, Actions, Console, ConsolePlugin};

#[derive(Actions, Clone, Debug, Event)]
enum MyGameActions {
    Spawn(f32),
    Quit,

    /// Demonstrates how trace messages are displayed.
    Trace,
    /// Demonstrates how debug messages are displayed.
    Debug,
    /// Demonstrates how info messages are displayed.
    Info,
    /// Demonstrates how warn messages are displayed.
    Warn,
    /// Demonstrates how error messages are displayed.
    Error,
}

pub fn main() {
    let default_filter = "info,full=trace".to_string();
    std::env::set_var("RUST_LOG", default_filter);
    let console_plugin = ConsolePlugin::<MyGameActions>::default();

    App::new()
        .add_plugins((
            DefaultPlugins.set(bevy::log::LogPlugin {
                custom_layer: slowchop_log_layer,
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
    // commands.spawn(Camera2dBundle::default());
    commands.spawn(Camera2d);
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
        match action {
            MyGameActions::Quit => std::process::exit(0),
            MyGameActions::Spawn(count) => info!("Spawning {} entities.", count),
            MyGameActions::Trace => trace!("Tracing 0xABCDEF 0xABCDEF 0xABCDEF"),
            MyGameActions::Debug => debug!("Debug 0xABCDEF"),
            MyGameActions::Info => info!("Some lovely information."),
            MyGameActions::Warn => warn!("Hello, this is a warning."),
            MyGameActions::Error => error!("Error! Error! Error! This is an error message."),
        }
    }
}
