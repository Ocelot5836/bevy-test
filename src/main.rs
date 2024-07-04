use std::ops::Deref;

use bevy::core_pipeline::experimental::taa::TemporalAntiAliasBundle;
use bevy::core_pipeline::fxaa::Fxaa;
use bevy::pbr::ScreenSpaceAmbientOcclusionBundle;
use bevy::prelude::*;
use bevy_atmosphere::prelude::*;

use crate::axis::AxisPlugin;
use crate::physics::{PhysicsPlugin, Velocity};
use crate::player_controller::{CameraRotation, Player, PlayerControllerPlugin};
use crate::voxel_mesher::{ClientWorld, schedule, VoxelPlugin};
use crate::world::VoxelWorld;

mod physics;
mod player_controller;
mod voxel_mesher;
mod voxel_renderer;
mod world;
mod hud;
mod axis;

fn main() {
    App::new()
        .insert_resource(Msaa::Off)
        .insert_resource(AmbientLight {
            brightness: 100.0,
            ..default()
        })
        .add_plugins((DefaultPlugins,
                      AtmospherePlugin,
                      VoxelPlugin,
                      PlayerControllerPlugin,
                      // HudPlugin,
                      AxisPlugin,
                      PhysicsPlugin::default(),
                      // TemporalAntiAliasPlugin
        ))
        .add_systems(Startup, spawn_view_model)
        .add_systems(Update, spawn_mesh)
        .run();
}

fn spawn_view_model(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Floor
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d::default().mesh().size(10.0, 10.0)),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3)),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });

    // ZY Wall
    let wall_material = materials.add(StandardMaterial {
        base_color: Color::rgb(0.4, 0.8, 0.5),
        ..default()
    });
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d {
            normal: Direction3d::NEG_X
        }.mesh().size(10.0, 10.0)),
        material: wall_material.clone(),
        transform: Transform::from_xyz(5.0, 5.0, 0.0),
        ..default()
    });

    // XY Wall
    commands.spawn(PbrBundle {
        mesh: meshes.add(Plane3d {
            normal: Direction3d::NEG_Z
        }.mesh().size(10.0, 10.0)),
        material: wall_material,
        transform: Transform::from_xyz(0.0, 5.0, 5.0),
        ..default()
    });

    commands.spawn(SpotLightBundle {
        transform: Transform::from_xyz(3.5, 4.0, 3.5)
            .looking_at(Vec3::new(0.0, -1.0, 0.0), Vec3::Y),
        spot_light: SpotLight {
            intensity: 100_000.0,
            color: Color::WHITE,
            shadows_enabled: true,
            inner_angle: 0.6,
            outer_angle: 0.8,
            ..default()
        },
        ..default()
    });

    commands
        .spawn((
            Player,
            SpatialBundle::default(),
            Velocity::default(),
            CameraRotation::default()
        ))
        .with_children(|parent| {
            parent.spawn((
                Camera3dBundle {
                    camera: Camera {
                        hdr: true,
                        ..default()
                    },
                    projection: PerspectiveProjection {
                        fov: 90.0_f32.to_radians(),
                        ..default()
                    }.into(),
                    transform: Transform::from_xyz(0.0, 1.35, 0.0),
                    ..default()
                },
                AtmosphereCamera::default(),
                Fxaa::default()
            )).insert(ScreenSpaceAmbientOcclusionBundle::default()).insert(TemporalAntiAliasBundle::default());
        });

    // commands.run_system(create_voxel_mesh)
}

fn spawn_mesh(commands: Commands,
              keyboard_input: Res<ButtonInput<KeyCode>>,
              client_world: Res<ClientWorld>,
              camera_transform: Query<&Transform, With<Player>>) {
    if keyboard_input.just_pressed(KeyCode::KeyE) {
        let mut world = client_world.0.write().unwrap();
        world.set_block(camera_transform.single().translation.floor().as_ivec3(), VoxelWorld::STONE);
    }
    if keyboard_input.just_pressed(KeyCode::KeyE) || keyboard_input.just_pressed(KeyCode::KeyK) {
        // FIXME delete old mesh
        schedule(commands, client_world.0.clone(), (camera_transform.single().translation / Vec3::splat(VoxelWorld::CHUNK_SIZE as f32)).floor().as_ivec3());
    }
}