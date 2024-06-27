use bevy::app::{App, Plugin, Update};
use bevy::math::Vec3;
use bevy::prelude::{Component, Query, Res, Resource, Time, Transform};
use bevy_inspector_egui::InspectorOptions;

#[derive(Debug)]
pub struct PhysicsPlugin {
    pub gravity: f32,
}

#[derive(Default, Debug, Component, InspectorOptions)]
pub struct Velocity(pub Vec3);

#[derive(Debug, Resource)]
struct PhysicsSettings {
    gravity: f32,
}

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PhysicsSettings {
            gravity: self.gravity
        });
        app.add_systems(Update, apply_velocity);
    }
}

impl Default for PhysicsPlugin {
    fn default() -> Self {
        Self {
            gravity: 9.81
        }
    }
}

fn apply_velocity(
    settings: Res<PhysicsSettings>,
    time: Res<Time>,
    mut transforms: Query<(&mut Transform, &mut Velocity)>) {
    let delta = time.delta().as_millis() as f32 / 1_000.0;

    for (mut transform, mut vel) in transforms.iter_mut() {
        vel.0.x *= 0.6;
        vel.0.z *= 0.6;
        vel.0.y -= settings.gravity * delta * delta;
        transform.translation += vel.0;

        if transform.translation.y < 0.0 {
            transform.translation.y = 0.0;
            vel.0 = Vec3::default();
        }
    }
}