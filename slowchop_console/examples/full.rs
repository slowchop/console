use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use slowchop_console::{Actions, Console, ConsolePlugin, Error};

#[derive(Actions, Debug, Event)]
enum GameActions {
    Help(String),
    List,
    Color,
    Spawn(f32),
}

pub fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            WorldInspectorPlugin::new(),
            ConsolePlugin::<GameActions>::new(),
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
