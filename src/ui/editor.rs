use crate::{layer, theme, GameState, GridPoint, BOTTOM_BAR_HEIGHT, GRID_SIZE};
use bevy::prelude::*;
use bevy_prototype_lyon::geometry::ShapeBuilder;
use bevy_prototype_lyon::shapes;
use bevy_prototype_lyon::prelude::*;

pub struct EditorPlugin;
#[derive(Component)]
pub struct EditorScreen;
#[derive(Component)]
struct ExitEditorButton;
#[derive(Component)]
struct Draggable;
#[derive(Component)]
struct Size(Vec2);
#[derive(Resource, Default)]
struct DragState {
    entity: Option<Entity>,
    offset: Vec2,
}

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DragState::default());
        app.add_systems(OnEnter(GameState::Editor), (spawn_grid, editor_ui).chain());
        app.add_systems(Update, exit_editor_button_system);
        app.add_systems(Update, (drag_system).chain());
        app.add_systems(OnExit(GameState::Editor), editor_destroy);
    }
}

fn editor_destroy(
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

fn spawn_grid(
    mut commands: Commands,
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

fn editor_ui(
    mut commands: Commands
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
                align_items: AlignItems::Start,
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

    let gray = 0.2;

    let draggable = commands.spawn((
            Node {
                width: Val::Px(50.0),
                height: Val::Px(50.0),
                position_type: PositionType::Absolute,
                left: Val::Px(90.0),
                bottom: Val::Px(10.0),
                ..default()
            },
            Size(Vec2::new(50.0, 50.0)),
            BackgroundColor(Color::srgba(gray, gray, gray, 0.4)),
            Draggable,
        ))
        .id();

    let out_terminus = commands.spawn(
        (
            Name::new("OutTerminus"),
            Node {
                width: Val::Px(50.0),
                height: Val::Px(50.0),
                position_type: PositionType::Absolute,
                left: Val::Px(160.0),
                bottom: Val::Px(10.0),
                ..default()
            },
            Size(Vec2::new(50.0, 50.0)),
            BackgroundColor(Color::srgba(gray, gray, gray, 0.4)),
            Draggable,
        )
    ).id();

    let in_terminus = commands.spawn(
        (
            Name::new("InTerminus"),
            Node {
                width: Val::Px(50.0),
                height: Val::Px(50.0),
                position_type: PositionType::Absolute,
                left: Val::Px(230.0),
                bottom: Val::Px(10.0),
                ..default()
            },
            Size(Vec2::new(50.0, 50.0)),
            BackgroundColor(Color::srgba(gray, gray, gray, 0.4)),
            Draggable,
        )
    ).id();

    commands
        .entity(editor_root)
        .add_children(&[editor_main_content, editor_bottom_bar, draggable, out_terminus, in_terminus]);
}

fn val_px(v: Val) -> f32 {
    match v {
        Val::Px(x) => x,
        _ => 0.0,
    }
}

fn drag_system(
    mut drag: ResMut<DragState>,
    mouse: Res<ButtonInput<MouseButton>>,
    window: Query<&Window>,
    mut query: Query<(Entity, &mut Node, &Size), With<Draggable>>,
) {
    let window = window.single().unwrap();
    let Some(cursor) = window.cursor_position() else { return; };
    let window_height = window.height();

    // Start drag
    if mouse.just_pressed(MouseButton::Left) && drag.entity.is_none() {
        for (entity, node, size) in query.iter() {
            let left = val_px(node.left);
            let bottom = val_px(node.bottom);

            // Calculate top position from bottom
            let top_pos = window_height - bottom - size.0.y;

            let pos = Vec2::new(left, top_pos);
            let min = pos;
            let max = pos + size.0;

            if cursor.x >= min.x && cursor.x <= max.x &&
                cursor.y >= min.y && cursor.y <= max.y {
                drag.entity = Some(entity);
                drag.offset = pos - cursor;
                break;
            }
        }
    }

    // Move
    if let Some(entity) = drag.entity {
        if let Ok((_, mut node, size)) = query.get_mut(entity) {
            let pos = cursor + drag.offset;
            node.left = Val::Px(pos.x);

            // Calculate bottom from top position
            let bottom = window_height - pos.y - size.0.y;
            node.bottom = Val::Px(bottom);

            // Clear top if it was set (optional)
            node.top = Val::Auto;
        }
    }

    // End drag
    if mouse.just_released(MouseButton::Left) {
        drag.entity = None;
    }
}
