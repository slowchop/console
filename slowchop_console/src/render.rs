use bevy::{log::Level, prelude::*, window::PrimaryWindow};

use crate::{plugin::EntityEntry, subscriber::LogEvent, Console};

#[derive(Component)]
pub struct Root;

#[derive(Component)]
pub struct History;

#[derive(Component)]
pub(crate) struct InputText;

pub fn setup_console(
    mut commands: Commands,
    console: Res<Console>,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };

    let mut group = commands.spawn((
        Name::new("Console"),
        Root,
        Node {
            padding: UiRect::all(Val::Px(0.)),
            margin: UiRect::all(Val::Px(0.)),
            width: Val::Percent(100.),
            height: Val::Px(window.resolution.height() * console.expand_fraction),
            flex_direction: FlexDirection::ColumnReverse,
            ..default()
        },
        Visibility::Hidden,
        GlobalZIndex(console.z_index),
        BackgroundColor(console.background_color.into()),
    ));

    group.with_children(|parent| {
        parent
            .spawn((
                Name::new("Input Container"),
                Node {
                    flex_grow: 0.,
                    padding: UiRect::new(Val::Px(10.), Val::Px(10.), Val::Px(2.), Val::Px(2.)),
                    margin: UiRect::top(Val::Px(7.)),
                    ..default()
                },
                BackgroundColor(console.input_background_color.into()),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Name::new("Input"),
                    InputText,
                    Node {
                        min_height: Val::Px(console.font_size),
                        flex_grow: 0.,
                        padding: UiRect::all(Val::Px(0.)),
                        margin: UiRect::all(Val::Px(0.)),
                        ..default()
                    },
                    Text::new(""),
                    TextFont {
                        font_size: console.font_size,
                        ..default()
                    },
                    TextColor(console.input_text_color),
                    Transform::from_translation(Vec3::new(0., 0., 0.)),
                ));
            });

        parent.spawn((
            Name::new("History"),
            History,
            Node {
                overflow: Overflow::clip_y(),
                flex_direction: FlexDirection::Column,
                align_content: AlignContent::FlexStart,
                ..default()
            },
        ));
    });
}

pub fn update_history(
    mut commands: Commands,
    mut console: ResMut<Console>,
    mut history_query: Query<Entity, With<History>>,
    mut log_events: EventReader<LogEvent>,
) {
    let history = history_query.single_mut();

    // For each new item, spawn a new entity with a Text component and add it to the children of the
    // history node.
    for log_event in log_events.read() {
        let color = match log_event.level {
            Level::TRACE => console.trace_text_color,
            Level::DEBUG => console.debug_text_color,
            Level::INFO => console.info_text_color,
            Level::WARN => console.warn_text_color,
            Level::ERROR => console.error_text_color,
        };

        let entity = commands
            .spawn((
                Name::new("HistoryItem"),
                Text::new(&log_event.message),
                TextFont {
                    font_size: console.font_size,
                    ..default()
                },
                TextColor(color),
                Node {
                    margin: UiRect::new(Val::Px(10.), Val::Px(10.), Val::Px(0.), Val::Px(0.)),
                    ..default()
                },
            ))
            .id();

        commands.entity(history).add_children(&[entity]);

        console.entity_entries.push_back(EntityEntry { entity });
    }

    // Check for older items that need to be removed from the history node. Remove them from the
    // children and destroy them.
    while console.entity_entries.len() > console.max_lines {
        if let Some(entry) = console.entity_entries.pop_front() {
            commands.entity(entry.entity).despawn_recursive();
        }
    }
}
