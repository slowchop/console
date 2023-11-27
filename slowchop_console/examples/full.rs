use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use slowchop_console::{Actions, Console, ConsolePlugin, Error};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;

#[derive(Actions, Clone, Debug, Event)]
enum GameActions {
    Help(String),
    List,
    Color,
    Spawn(f32),
}

pub fn main() {
    let console_plugin = ConsolePlugin::<GameActions>::new();

    let default_filter = "info,fart=debug,wgpu=error,naga=warn".to_string();
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
            WorldInspectorPlugin::new(),
            console_plugin,
        ))
        .add_systems(Startup, setup_camera)
        .add_systems(Update, handle_actions)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn handle_actions(mut actions: EventReader<GameActions>) {
    for action in actions.read() {
        match action {
            GameActions::Help(text) => info!("Help: {}", text),
            GameActions::List => info!("List"),
            GameActions::Color => info!("Color"),
            GameActions::Spawn(count) => info!("Spawn: {}", count),
        }
    }
}
