use bevy::input::common_conditions::input_toggle_active;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use slowchop_console::{Actions, ConsolePlugin};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[derive(Actions, Clone, Debug, Event)]
enum MyGameActions {
    Help(String),
    List,
    Color,
    Spawn(f32),

    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

pub fn main() {
    let console_plugin = ConsolePlugin::<MyGameActions>::new();

    let default_filter = "trace,wgpu=error,naga=warn,bevy=info,winit=info,gilrs=info".to_string();
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new(&default_filter))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(tracing_subscriber::fmt::Layer::new().with_ansi(true))
        .with(console_plugin.clone())
        .init();

    App::new()
        .add_plugins((
            DefaultPlugins.build().disable::<LogPlugin>(),
            WorldInspectorPlugin::new().run_if(input_toggle_active(false, KeyCode::F1)),
            console_plugin,
        ))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, handle_actions)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn handle_actions(mut actions: EventReader<MyGameActions>) {
    for action in actions.read() {
        match action {
            MyGameActions::Help(text) => info!("Help: {}", text),
            MyGameActions::List => info!("List"),
            MyGameActions::Color => info!("Color"),
            MyGameActions::Spawn(count) => info!("Spawn: {}", count),
            MyGameActions::Trace => trace!("Tracing 0xABCDEF 0xABCDEF 0xABCDEF"),
            MyGameActions::Debug => debug!("Debug 0xABCDEF"),
            MyGameActions::Info => info!("Some lovely information "),
            MyGameActions::Warn => warn!("Hello, this is a warning."),
            MyGameActions::Error => error!("Error! Error! Error! This is an error message."),
        }
    }
}
