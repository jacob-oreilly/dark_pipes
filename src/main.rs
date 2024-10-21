use bevy::{prelude::*, utils::info, window::{PrimaryWindow, WindowResolution}};
use bevy_ecs_ldtk::prelude::*;

#[derive(Component)]
struct CurrentWorld;

#[derive(Component)]
struct Collider;

#[derive(Component)]
struct Player {
    movement_speed: f32,
}

const CAM_LERP_FACTOR: f32 = 2.;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(650., 370.).with_scale_factor_override(1.0),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(LdtkPlugin)
        .add_systems(Startup, (setup_camera, spawn_player))
        .add_systems(Update, (file_drag_and_drop_system, update_camera, update_player))
        .insert_resource(LevelSelection::index(0))
        .run();
}

fn setup_camera(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>, asset_server: Res<AssetServer>) {
    let window = window_query.get_single().unwrap();

    let mut camera_bundle = Camera2dBundle {
        transform: Transform::from_xyz(640.0 / 2.0, 360.0 / 2.0, 0.0),
        ..Default::default()
    };
    camera_bundle.projection.scale /= 1.75;

    commands.spawn(
        camera_bundle    
    );

    commands.spawn((
        LdtkWorldBundle {
            ldtk_handle: asset_server
                .load("pipes2.ldtk"),
            ..default()
        },
        CurrentWorld,
    ));
}

fn update_camera(
    mut camera: Query<&mut Transform, (With<Camera2d>, Without<Player>)>,
    player: Query<&Transform, (With<Player>, Without<Camera2d>)>,
    time: Res<Time>,
) {
    let Ok(mut camera) = camera.get_single_mut() else {
        return;
    };

    let Ok(player) = player.get_single() else {
        return;
    };

    let Vec3 {x, y, ..} = player.translation;
    let direction = Vec3::new(x, y, camera.translation.z);

    camera.translation = camera
        .translation
        .lerp(direction, time.delta_seconds() * CAM_LERP_FACTOR);
}

fn spawn_player(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_query: Query<&Window, With<PrimaryWindow>>
) {
    let window = window_query.get_single().unwrap();
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("../assets/static_character.png"),
            transform: Transform::from_xyz(window.width() / 2.0 , window.height() - 30.0, 10.0),
            ..default()
        },
        Player {
            movement_speed: 100.0,
        },
        Collider,
    ));
}

fn update_player(
    mut player: Query<(&mut Transform, &Player), With<Player>>,
    time: Res<Time>,
    kb_input: Res<ButtonInput<KeyCode>>,
) {
    let (mut player_transform, player) = player.get_single_mut().unwrap();

    let mut direction = Vec2::ZERO;

    if kb_input.pressed(KeyCode::KeyW) {
        direction.y += 1.;
    }

    if kb_input.pressed(KeyCode::KeyS) {
        direction.y -= 1.;
    }

    if kb_input.pressed(KeyCode::KeyA) {
        direction.x -= 1.;
    }

    if kb_input.pressed(KeyCode::KeyD) {
        direction.x += 1.;
    }

    let move_delta = direction.normalize_or_zero() * player.movement_speed * time.delta_seconds();
    player_transform.translation += move_delta.extend(0.);
}

fn file_drag_and_drop_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut events: EventReader<FileDragAndDrop>,
    query_current_world: Query<Entity, With<CurrentWorld>>,
) {
    for event in events.read() {
        match event {
            FileDragAndDrop::DroppedFile {
                window,
                path_buf
            } => {
                for world in &query_current_world {
                    commands
                        .entity(world)
                        .despawn_recursive();
                }
                if !path_buf
                    .extension()
                    .is_some_and(|ext| ext == "ldtk")
                {
                    continue;
                }

                commands.spawn((
                    LdtkWorldBundle {
                        ldtk_handle: asset_server.load(
                            path_buf.display().to_string(),
                        ),
                        ..default()
                    },
                    CurrentWorld,
                ));
            }
            FileDragAndDrop::HoveredFile { 
                window, 
                path_buf 
            } => {
                info!("hovering");
            }
            FileDragAndDrop::HoveredFileCanceled { 
                window 
            } => {
                info!("cancelled");
            }
        }
    }
}
