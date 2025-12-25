use crate::{layer, theme, GameState, GridPoint, Handles, RoadGraph, SelectedLevel, BOTTOM_BAR_HEIGHT, GRID_SIZE};

pub struct EditorPlugin;
#[derive(Component)]
pub struct EditorScreen;
#[derive(Component)]
struct ExitEditorButton;

use bevy::prelude::*;
use bevy_prototype_lyon::geometry::ShapeBuilder;
use bevy_prototype_lyon::shapes;
use bevy_prototype_lyon::prelude::*;
use crate::level::Level;
use crate::save::Solutions;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Editor), (spawn_level, editor).chain());
        app.add_systems(Update, exit_editor_button_system);
        app.add_systems(OnExit(GameState::Editor), editor_exit);
    }
}

fn editor_enter() {
    ()
}

fn editor_exit(
    mut commands: Commands,
    query: Query<Entity, With<EditorScreen>>
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn exit_editor_button_system(
    query: Query<(&Interaction, &ExitEditorButton), Changed<Interaction>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (interaction, _) in &query {
        match *interaction {
            Interaction::Pressed => {
                next_state.set(GameState::LevelSelect);
            },
            _ => {}
        }
    }
}

fn spawn_level(
    mut commands: Commands,
    mut graph: ResMut<RoadGraph>,
    levels: Res<Assets<Level>>,
    selected_level: Res<SelectedLevel>,
    handles: Res<Handles>,
    solutions: Res<Solutions>,
) {
    for x in ((-25 * (GRID_SIZE as i32))..=25 * (GRID_SIZE as i32)).step_by(GRID_SIZE as usize) {
        for y in (-15 * (GRID_SIZE as i32)..=15 * (GRID_SIZE as i32)).step_by(GRID_SIZE as usize) {
            commands.spawn((
                ShapeBuilder::with(&shapes::Circle {
                    radius: 2.5,
                    ..default()
                })
                    .fill(theme::GRID)
                    .build(),
                Transform::from_xyz(x as f32, y as f32, layer::GRID),
                GridPoint,
                DespawnOnExit(GameState::Editor),
            ));
        }
    }
}

fn editor(
    mut commands: Commands,
    handles: Res<Handles>,
) {
    let editor_root = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                overflow: Overflow::clip(),
                ..default()
            },
            EditorScreen,
        ))
        .id();

    let editor_bottom_bar = commands
        .spawn((
            Node {
                width: Val::Percent(100.),
                height: Val::Px(BOTTOM_BAR_HEIGHT),
                flex_shrink: 0.0,
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect {
                    left: Val::Px(20.),
                    right: Val::Px(20.),
                    top: Val::Px(10.),
                    bottom: Val::Px(10.),
                },
                ..default()
            },
            BackgroundColor(theme::UI_PANEL_BACKGROUND.into()),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("ExitEditorItem"),
                Node {
                    width: Val::Px(50.0),
                    height: Val::Px(50.0),
                    align_self: AlignSelf::Center,
                    display: Display::Flex,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                ExitEditorButton,
                Button,
            )).with_children(|parent| {
                parent.spawn(
                    Text::new("←"),
                );
            });
        })
        .id();

    let editor_main_content = commands
        .spawn((Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            padding: UiRect::all(Val::Px(20.)),
            column_gap: Val::Px(20.),
            display: Display::Grid,
            grid_template_columns: vec![GridTrack::flex(0.75), GridTrack::flex(0.25)],
            ..default()
        },))
        .id();

    commands
        .entity(editor_root)
        .add_children(&[editor_main_content, editor_bottom_bar]);
}