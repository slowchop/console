use bevy::prelude::*;
use slowchop_console::{slowchop_log_layer, Console, ConsolePlugin, FunctionArgs};

pub fn main() {
    let default_filter = "info,full=trace".to_string();
    std::env::set_var("RUST_LOG", default_filter);
    let console_plugin = ConsolePlugin::default();

    let mut console = Console::default();
    console.background_color = Color::BLACK;

    let mut app = App::new();

    console.add_system_command(
        "quit",
        app.world_mut().register_system(
            |args: In<FunctionArgs>, mut app_exit: EventWriter<AppExit>| {
                info!("Quitting the application.");
                info!("got some args though: {:?}", args);
                app_exit.send(AppExit::Success);
            },
        ),
    );

    app.insert_resource(console);

    app.add_plugins((
        DefaultPlugins.set(bevy::log::LogPlugin {
            custom_layer: slowchop_log_layer,
            ..default()
        }),
        console_plugin,
        // WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::F1)),
    ))
    .add_systems(Startup, (setup_camera, start_with_console_open))
    .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2d);
}

fn start_with_console_open(mut console: ResMut<Console>) {
    console.open();

    trace!(?console.background_color, "This is a test trace message with a variable.");
    debug!("This is a test debug message: 0xABCDEF");
    info!("Press ` (backtick) or ~ (tilde) (KeyCode::Backquote) to toggle the console.");
    warn!("Press Escape to close the console.");
    error!("This is a test error message: 0xABCDEF");
}
