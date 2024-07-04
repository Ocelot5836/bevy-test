use bevy::app::{App, Plugin, Startup};
use bevy::asset::Assets;
use bevy::math::UVec2;
use bevy::prelude::{Camera, Camera2dBundle, Circle, ClearColorConfig, Color, ColorMaterial, Commands, default, Mesh, ResMut};
use bevy::render::camera::Viewport;
use bevy::render::view::RenderLayers;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};

#[derive(Debug)]
pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands,
         mut meshes: ResMut<Assets<Mesh>>,
         mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.spawn((Camera2dBundle {
        camera: Camera {
            // renders after / on top of the main camera
            order: 1,
            clear_color: ClearColorConfig::None,
            viewport: Some(Viewport{
                physical_size: UVec2::new(464, 64),
                ..default()
            }),
            ..default()
        },
        ..default()
    }, RenderLayers::layer(1)));

    let color = Color::hsl(1.0, 0.95, 0.7);
    commands.spawn((MaterialMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Circle { radius: 50.0 })),
        material: materials.add(color),
        ..default()
    }, RenderLayers::layer(1)));

    // commands.spawn(NodeBundle {
    //     style: Style {
    //         width: Val::Percent(100.0),
    //         height: Val::Percent(100.0),
    //         justify_content: JustifyContent::SpaceBetween,
    //         ..default()
    //     },
    //     ..default()
    // });
}